import {
    Vector3,
    MeshBuilder,
    HemisphericLight,
    ShaderMaterial,
    type GroundMesh,
    Matrix,
    ComputeShader,
    UniformBuffer,
    WebGPUDataBuffer,
    ShaderLanguage, GizmoManager, Mesh, Quaternion, WebGPUEngine, Observable, type Nullable,
} from "@babylonjs/core";
import { assembleFullVertexShader } from "@/shaders/shader-processor";
import vertexShader from "./ModelDisplayVirtualScene.vert.wgsl?raw";
import { readOrCreateFile, type SceneFiles } from "@/filesystem/scene-files";
import type { BaseScene } from "./BaseScene";
import type {ObjectData} from "@/scenes/ObjectData";
import { makeHotCache } from "@/stores/hot-cache";
import {RuntimeMesh} from "@/scenes/RuntimeMesh";
import {shallowRef} from "vue";

let getHotCache = makeHotCache<{
    "camera-target": Vector3;
    "camera-alpha": number;
    "camera-beta": number;
    "camera-radius": number;
    "scene-objects": Array<ObjectData>;
}>(import.meta.url);


export class ModelDisplayVirtualScene {
  private _disposables: Disposable[] = [];

  private models : Map<GroundMesh, ObjectData> = new Map<GroundMesh, ObjectData>();
  private runtimeMeshes: Array<RuntimeMesh> = [];

  public readonly key = ModelDisplayVirtualScene.name;
  private hotCache = getHotCache();
  public selectedObject = shallowRef<string | null>("");
  onSelectedObject : Observable<Nullable<string>> = new Observable<Nullable<string>>();


  constructor(scene: BaseScene, public files: SceneFiles) {

    let light = this.addDisposable(
      new HemisphericLight("light1", new Vector3(0, 1, 0), scene)
    );
    light.intensity = 0.7;

    const gizmoManager = new GizmoManager(scene);
    gizmoManager.positionGizmoEnabled = true;
    gizmoManager.rotationGizmoEnabled = true;
    gizmoManager.onAttachedToMeshObservable.add(
        (mesh) =>
        {
            if(mesh)
            {
                this.selectedObject.value = mesh.name;
                this.onSelectedObject.notifyObservers(mesh.name);
                alert("SELECTED " + mesh.name);
            }
            else
                alert("SELECTED " + mesh);
        }
    )
      //gizmoManager.attachableMeshes = [this.mathmodel];


    let objects = this.hotCache.getOrInsert("scene-objects",
        () => [
            {
                position: new Vector3(0,0,0),
                code: "",
                rotation: Quaternion.Identity(),
                name: "Math2Modelmesh1"
            }
        ]);
    gizmoManager.attachableMeshes = [];
    for (const obj of objects) {
        let mesh = this.addMesh(scene, files, obj);
        gizmoManager.attachableMeshes.push(mesh.mathmodel);
    }
  }

  public createNewMeshFromScratch(name: string)
  {
      this.hotCache.get("scene-objects")?.push(
          {
              position: Vector3.One(),
              rotation: Quaternion.Identity(),
              code: "",
              name: name
          }
      );
  }

    private addMesh(scene: BaseScene, files: SceneFiles, obj: ObjectData | {
        code: string;
        rotation: Quaternion;
        name: string;
        position: Vector3
    }): RuntimeMesh {
        let cs = this.createComputeShader(scene);
        let mesh = new RuntimeMesh(scene, this, files, obj);
        const csUniformBuffer = this.addDisposable(new UniformBuffer(scene.engine));
        csUniformBuffer.addUniform("visibleInstances", 1);
        cs.setUniformBuffer("inputBuffer", csUniformBuffer);
        this.registerMeshForUBOUpdates(mesh.mathmodel,scene,cs,csUniformBuffer);
        return mesh;
    }

  dispose() {
    this._disposables.forEach((d) => d[Symbol.dispose]());
  }

  public addDisposable<T extends { dispose: () => void }>(disposable: T): T {
    this._disposables.push({
      [Symbol.dispose]: () => disposable.dispose(),
    });
    return disposable;
  }

  private shaderKey(name: string) {
    return this.key + "-" + name;
  }

  readOrCreateFile(
    name: string,
    defaultContent: () => string,
    transformer: (content: string) => string = (content) => content
  ): string {
    const content = readOrCreateFile(this.files, name, defaultContent);
    // ShaderStore.ShadersStoreWGSL[this.shaderKey(name)] =
    return transformer(content);
  }

    private createComputeShader(scene: BaseScene) {
        let cs = new ComputeShader(
            "mycs",
            scene.engine,
            {
                computeSource: `
        struct IndirectDrawBuffer {
            indexOrVertexCount: u32,
            instanceCount: u32,
            firstIndexOrVertex: u32,
            tmp1: u32,
            tmp2: u32,
        };

        struct InputBuffer {
            visibleInstances: u32,
        };

        @group(0) @binding(0) var<storage,read_write> indirectDrawBuffer : IndirectDrawBuffer;
        @group(0) @binding(1) var<uniform> inputBuffer : InputBuffer;

        @compute @workgroup_size(1, 1, 1)
        fn main(@builtin(global_invocation_id) global_id : vec3<u32>) {
            indirectDrawBuffer.instanceCount = inputBuffer.visibleInstances;
        }
    `,
                },
                {
                    bindingsMapping: {
                        indirectDrawBuffer: { group: 0, binding: 0 },
                        inputBuffer: { group: 0, binding: 1 },
                    },
                }
            );
        return cs;
    }

    private registerMeshForUBOUpdates(mesh: Mesh, scene: BaseScene, cs: ComputeShader, csUniformBuffer: UniformBuffer) {
        let t = 0;
        let first = true;
        const renderObserver = scene.onBeforeRenderObservable.add(() => {
            // TODO: Correctly handle resetMaterials
            const renderPassId = scene.engine.currentRenderPassId;
            const drawWrapper = mesh.subMeshes[0]._getDrawWrapper(
                renderPassId,
                false
            );
            let refresh = 0;
            if (t++ > refresh) {
                t = 0;
                let indirectDrawBuffer: GPUBuffer = (drawWrapper?.drawContext as any)
                    ?.indirectDrawBuffer;
                if (indirectDrawBuffer) {
                    if (first) {
                        cs.setStorageBuffer(
                            "indirectDrawBuffer",
                            new WebGPUDataBuffer(indirectDrawBuffer, 20)
                        );
                        first = false;
                    }

                    var x = Math.floor(
                        255 / Vector3.Distance(scene.camera.position, mesh.position)
                    );
                    var nextSquareNum = Math.pow(Math.ceil(Math.sqrt(x)), 2);
                    csUniformBuffer.updateUInt("visibleInstances", nextSquareNum);
                    csUniformBuffer.update();

                    cs.dispatch(1, 1, 1);
                }
            }
        });

        this.addDisposable({
            dispose: () => {
                scene.onBeforeRenderObservable.remove(renderObserver);
            },
        });
  }

}
