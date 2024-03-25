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
  ShaderLanguage,
} from "@babylonjs/core";
import { assembleFullVertexShader } from "@/shaders/shader-processor";
import vertexShader from "./MyFirstScene.vert.wgsl?raw";
import { readOrCreateFile, type SceneFiles } from "@/filesystem/scene-files";
import type { BaseScene } from "./BaseScene";

export class MyFirstScene {
  private plane: GroundMesh;
  private _disposables: Disposable[] = [];

  public readonly key = MyFirstScene.name;
  constructor(scene: BaseScene, public files: SceneFiles) {
    let light = this.addDisposable(
      new HemisphericLight("light1", new Vector3(0, 1, 0), scene)
    );
    light.intensity = 0.7;

    this.plane = this.addDisposable(
      MeshBuilder.CreateGround(
        "ground",
        {
          width: 3.14159265359 * 2,
          height: 3.14159265359 * 2,
          subdivisions: 5,
        },
        scene
      )
    );
    this.plane.position.y = 10.1;
    this.plane.thinInstanceAddSelf();
    this.plane.thinInstanceAdd(
      Matrix.Translation(3.14159265359, 0.0, 3.14159265359)
    );
    this.plane.thinInstanceCount = 1;

    const customFragmentShader = this.readOrCreateFile(
      "customFragmentShader",
      () => `
    varying vNormal : vec3<f32>;
    varying vUV : vec2<f32>;
    @fragment
    fn main(input : FragmentInputs) -> FragmentOutputs {
        fragmentOutputs.color = vec4<f32>(input.vUV,1.0, 1.0);
    }
`
    );
    const customVertexShader = this.readOrCreateFile(
      "customVertexShader",
      () => vertexShader,
      assembleFullVertexShader
    );

    let shaderMaterial = this.addDisposable(
      new ShaderMaterial(
        "custom",
        scene,
        {
          vertexSource: customVertexShader,
          fragmentSource: customFragmentShader,
        },
        {
          attributes: ["uv", "position", "normal"],
          uniformBuffers: ["Scene", "Mesh", "instances"],
          // uniforms: ["iTime", "iTimeDelta", "iFrame", "worldViewProjection"],
          shaderLanguage: ShaderLanguage.WGSL,
        }
      )
    );
    shaderMaterial.backFaceCulling = false;
    shaderMaterial.wireframe = false;
    const myUBO = this.addDisposable(new UniformBuffer(scene.engine));
    myUBO.addUniform("iTime", 1);
    myUBO.addUniform("iTimeDelta", 1);
    myUBO.addUniform("iFrame", 1);
    myUBO.addUniform("width", 1);
    myUBO.update();
    shaderMaterial.setUniformBuffer("myUBO", myUBO);
    shaderMaterial.onBind = (m: any) => {
      let x = Math.floor(
        255 / Vector3.Distance(scene.camera.position, this.plane.position)
      );
      let nextSquareNum = Math.pow(Math.ceil(Math.sqrt(x)), 2);
      myUBO.updateFloat("iTime", scene.time / 1000);
      myUBO.updateFloat("iTimeDelta", scene.deltaTime / 1000);
      myUBO.updateFloat("iFrame", scene.frame);
      myUBO.updateFloat("width", nextSquareNum);
      myUBO.update();
    };
    this.plane.material = shaderMaterial;

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
    /*
    this.addDisposable({
      dispose: () => {
        console.log("disposing compute shader");
        cs._effect?.dispose();
      },
    });*/
    const csUniformBuffer = this.addDisposable(new UniformBuffer(scene.engine));
    csUniformBuffer.addUniform("visibleInstances", 1);
    cs.setUniformBuffer("inputBuffer", csUniformBuffer);

    let t = 0;
    let first = true;

    const renderObserver = scene.onBeforeRenderObservable.add(() => {
      // TODO: Correctly handle resetMaterials
      const renderPassId = scene.engine.currentRenderPassId;
      const drawWrapper = this.plane.subMeshes[0]._getDrawWrapper(
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
            255 / Vector3.Distance(scene.camera.position, this.plane.position)
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

  dispose() {
    this._disposables.forEach((d) => d[Symbol.dispose]());
  }

  private addDisposable<T extends { dispose: () => void }>(disposable: T): T {
    this._disposables.push({
      [Symbol.dispose]: () => disposable.dispose(),
    });
    return disposable;
  }

  private shaderKey(name: string) {
    return this.key + "-" + name;
  }

  private readOrCreateFile(
    name: string,
    defaultContent: () => string,
    transformer: (content: string) => string = (content) => content
  ): string {
    const content = readOrCreateFile(this.files, name, defaultContent);
    // ShaderStore.ShadersStoreWGSL[this.shaderKey(name)] =
    return transformer(content);
  }
}
