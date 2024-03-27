import { makeHotCache } from "@/stores/hot-cache";

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
} from "@babylonjs/core";
import { GridMaterial } from "@babylonjs/materials/grid/gridMaterial";
import backgroundGround from "@/assets/backgroundGround.png";

let getHotCache = makeHotCache<{
  "camera-target": Vector3;
  "camera-alpha": number;
  "camera-beta": number;
  "camera-radius": number;
}>(import.meta.url);

export type Milliseconds = number;
export type Seconds = number;

export class BaseScene extends Scene {
  private hotCache = getHotCache();
  private gridMesh: GroundMesh;
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

  constructor(public engine: WebGPUEngine) {
    super(engine);

    this.createDefaultEnvironment({
      enableGroundShadow: true,
      enableGroundMirror: true,
      groundYBias: 0,
      groundSize: 200,
      skyboxSize: 600,
      groundColor: new Color3(0.09, 0.59, 0.85),
      skyboxColor: new Color3(0.09, 0.59, 0.85),
    });
    this.gridMesh = makeGridMesh(this);

    let camera = new ArcRotateCamera(
      "camera",
      this.hotCache.getOrInsert("camera-alpha", () => 0),
      this.hotCache.getOrInsert("camera-beta", () => 0),
      this.hotCache.getOrInsert("camera-radius", () => 30),
      this.hotCache.getOrInsert("camera-target", () => new Vector3(0, 10, 0)),
      this
    );
    camera.minZ = 0.01;
    camera.maxZ = 1000;
    camera.attachControl(true);
    camera.lowerBetaLimit = 0.2; // Almost peak
    camera.upperBetaLimit = 1.5; // Almost straight down to xz-plane
    camera.lowerRadiusLimit = 3; // Not tooo close
    this._camera = camera;

    this._startTime = performance.now();
  }

  update() {
    this._currentTime = performance.now();
    this._frame++;

    this.hotCache.set("camera-target", this._camera.target.clone());
    this.hotCache.set("camera-alpha", this._camera.alpha);
    this.hotCache.set("camera-beta", this._camera.beta);
    this.hotCache.set("camera-radius", this._camera.radius);
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
