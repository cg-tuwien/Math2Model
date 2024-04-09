import {
  type AbstractMesh,
  GroundMesh,
  MeshBuilder,
  Quaternion,
  ShaderLanguage,
  ShaderMaterial,
  UniformBuffer,
  Vector3,
  Matrix,
  WebGPUDataBuffer,
  ComputeShader,
} from "@babylonjs/core";
import type { ModelDisplayVirtualScene } from "./ModelDisplayVirtualScene";
import { type FilePath } from "@/filesystem/scene-files";
import { DisposableStack } from "../disposable-stack";

export class VirtualModel implements Disposable {
  public key: string = crypto.randomUUID();
  private _disposables = new DisposableStack();
  private _mesh: GroundMesh;

  get position() {
    return this._mesh.position.clone();
  }
  set position(value: Vector3) {
    this._mesh.position = value;
  }
  get rotation() {
    return this._mesh.rotationQuaternion!.clone();
  }
  set rotation(value: Quaternion) {
    this._mesh.rotationQuaternion = value;
  }

  constructor(
    public name: string,
    public code: ShaderCodeRef,
    virtualScene: ModelDisplayVirtualScene
  ) {
    const scene = virtualScene.scene;
    const files = virtualScene.getVertexAndFragmentShaders();

    this._mesh = this._disposables.addDisposable(
      MeshBuilder.CreateGround(
        this.key + "_mesh",
        {
          width: 3.14159265359 * 2,
          height: 3.14159265359 * 2,
          subdivisions: 5,
        },
        scene
      )
    );
    // Needs to be set, otherwise Babylon.js will use a Vector3 property for rotation.
    this._mesh.rotationQuaternion = Quaternion.Identity();
    this._mesh.thinInstanceAddSelf();
    this._mesh.thinInstanceCount = 0.1; // This is a disgusting hack to tell Babylon.js "render zero instances"

    let shaderMaterial = this._disposables.addDisposable(
      new ShaderMaterial(
        "custom",
        scene,
        {
          vertexSource: files.readVertexShader(this.code.vertexFile),
          fragmentSource: files.readFragmentShader(this.code.fragmentFile),
        },
        {
          attributes: ["uv", "position", "normal"],
          uniformBuffers: ["Scene", "Mesh", "instances"],
          shaderLanguage: ShaderLanguage.WGSL,
        }
      )
    );
    shaderMaterial.backFaceCulling = false;
    const myUBO = this._disposables.addDisposable(
      new UniformBuffer(scene.engine)
    );
    myUBO.addUniform("width", 1);
    myUBO.update();
    shaderMaterial.setUniformBuffer("myUBO", myUBO);
    shaderMaterial.onBind = (m: AbstractMesh) => {
      let x = Math.floor(
        255 / Vector3.Distance(scene.camera.position, this.position)
      );
      let nextSquareNum = Math.pow(Math.ceil(Math.sqrt(x)), 2);
      myUBO.updateFloat("width", nextSquareNum);
      myUBO.update();
    };
    shaderMaterial.setUniformBuffer("globalUBO", virtualScene.globalUBO);
    this._mesh.material = shaderMaterial;

    // TODO: Import the vertex shader function into here
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

    const csUniformBuffer = this._disposables.addDisposable(
      new UniformBuffer(scene.engine)
    );
    csUniformBuffer.addUniform("visibleInstances", 100);
    cs.setUniformBuffer("inputBuffer", csUniformBuffer);

    let first = true;
    const renderObserver = scene.onBeforeRenderObservable.add(() => {
      const renderPassId = scene.engine.currentRenderPassId;
      const drawWrapper = this._mesh.subMeshes[0]._getDrawWrapper(
        renderPassId,
        false
      );
      const indirectDrawBuffer: GPUBuffer = (drawWrapper?.drawContext as any)
        ?.indirectDrawBuffer;
      if (!indirectDrawBuffer) {
        return;
      }
      if (first) {
        cs.setStorageBuffer(
          "indirectDrawBuffer",
          new WebGPUDataBuffer(indirectDrawBuffer, 20)
        );
        first = false;
      }

      const x = Math.floor(
        255 / Vector3.Distance(scene.camera.position, this._mesh.position)
      );
      const nextSquareNum = Math.pow(Math.ceil(Math.sqrt(x)), 2);
      csUniformBuffer.updateUInt("visibleInstances", nextSquareNum);
      csUniformBuffer.update();

      cs.dispatch(1, 1, 1);
    });
    this._disposables.addDisposable({
      dispose: () => {
        scene.onBeforeRenderObservable.remove(renderObserver);
      },
    });

    virtualScene.internalAddModel(this);
  }

  [Symbol.dispose]() {
    this._disposables[Symbol.dispose]();
  }

  static serialize(actor: VirtualModel): SerializedActor {
    return {
      key: actor.key,
      name: actor.name,
      code: actor.code,
      position: actor.position.asArray(),
      rotation: actor.rotation.asArray(),
    };
  }

  static deserialize(
    data: SerializedActor,
    virtualScene: ModelDisplayVirtualScene
  ): VirtualModel {
    const actor = new VirtualModel(data.name, data.code, virtualScene);
    actor.key = data.key;
    actor.position = Vector3.FromArray(data.position);
    actor.rotation = Quaternion.FromArray(data.rotation);
    return actor;
  }
}

export interface ShaderCodeRef {
  readonly vertexFile: FilePath;
  readonly fragmentFile: FilePath;
}

export interface SerializedActor {
  key: string;
  name: string;
  code: ShaderCodeRef;
  position: number[];
  rotation: number[];
}
