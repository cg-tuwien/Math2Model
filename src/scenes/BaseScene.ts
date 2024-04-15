import {
  Scene,
  type WebGPUEngine,
  Color3,
  CreateGround,
  Texture,
  Vector3,
  type GroundMesh,
  ArcRotateCamera,
  type Camera,
  type Mesh,
} from "@babylonjs/core";
import { GridMaterial } from "@babylonjs/materials/grid/gridMaterial";
import backgroundGround from "@/assets/backgroundGround.png";
import {
  CacheFileSchemaUrl,
  readCacheFile,
  writeCacheFile,
} from "@/filesystem/cache-file";
import type { WritableFiles } from "@/filesystem/reactive-files";
import { mapOptional } from "@/option";

export type Milliseconds = number;
export type Seconds = number;

export class BaseScene extends Scene {
  private _gridMesh: GroundMesh;
  private _camera: ArcRotateCamera;
  private _frame: number = 0;
  private _startTime: Milliseconds = 0;
  private _currentTime: Milliseconds = 0;

  get camera(): Camera {
    return this._camera;
  }

  get frame() {
    return this._frame;
  }

  get time(): Milliseconds {
    return this._currentTime - this._startTime;
  }

  get gridMesh(): Mesh {
    return this._gridMesh;
  }

  constructor(public engine: WebGPUEngine) {
    super(engine);
    const cacheFile = readCacheFile();

    this.createDefaultEnvironment({
      enableGroundShadow: true,
      enableGroundMirror: true,
      groundYBias: 0,
      groundSize: 200,
      skyboxSize: 600,
      groundColor: new Color3(0.09, 0.59, 0.85),
      skyboxColor: new Color3(0.09, 0.59, 0.85),
    });
    this._gridMesh = makeGridMesh(this);

    let camera = new ArcRotateCamera(
      "camera",
      cacheFile?.camera?.alpha ?? 0,
      cacheFile?.camera?.beta ?? 0,
      cacheFile?.camera?.radius ?? 30,
      mapOptional(
        cacheFile?.camera?.target,
        ([a, b, c]) => new Vector3(a, b, c)
      ) ?? new Vector3(0, 10, 0)
    );
    camera.minZ = 0.01;
    camera.maxZ = 1000;
    camera.attachControl(true);
    camera.lowerBetaLimit = 0.2; // Almost peak
    camera.upperBetaLimit = 1.5; // Almost straight down to xz-plane
    camera.lowerRadiusLimit = 0.1;
    this._camera = camera;

    this._startTime = performance.now();

    const writeCache = async () => {
      writeCacheFile({
        $schema: CacheFileSchemaUrl,
        camera: {
          type: "arc-rotate-camera",
          alpha: camera.alpha,
          beta: camera.beta,
          radius: camera.radius,
          target: [camera.target.x, camera.target.y, camera.target.z],
        },
      });
    };
    this.onDisposeObservable.add(() => writeCache());
    globalThis.addEventListener("beforeunload", () => writeCache());
  }

  update() {
    this._currentTime = performance.now();
    this._frame++;
  }
}

function makeGridMesh(scene: Scene): GroundMesh {
  const gridMesh = CreateGround(
    "grid",
    {
      width: 1.0,
      height: 1.0,
      subdivisions: 1,
    },
    scene
  );
  if (!gridMesh.reservedDataStore) {
    gridMesh.reservedDataStore = {};
  }
  gridMesh.scaling.x = 150;
  gridMesh.scaling.z = 150;
  gridMesh.reservedDataStore.isInspectorGrid = true;
  gridMesh.isPickable = false;

  const groundMaterial = new GridMaterial("GridMaterial", scene);
  groundMaterial.majorUnitFrequency = 10;
  groundMaterial.minorUnitVisibility = 0.3;
  groundMaterial.gridRatio = 0.01;
  groundMaterial.backFaceCulling = false;
  groundMaterial.mainColor = new Color3(1, 1, 1);
  groundMaterial.lineColor = new Color3(1.0, 1.0, 1.0);
  groundMaterial.opacity = 0.2;
  groundMaterial.zOffset = 1.0;
  groundMaterial.opacityTexture = new Texture(backgroundGround, scene);
  gridMesh.material = groundMaterial;

  return gridMesh;
}
