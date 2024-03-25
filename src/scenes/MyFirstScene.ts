import { HmrCache, makeHotCache, useHotCacheStore } from "@/stores/hot-cache";
import {
  Engine,
  Scene,
  Vector3,
  MeshBuilder,
  HemisphericLight,
  ShaderMaterial,
  FlyCamera,
  WebGPUEngine,
  Effect,
  type GroundMesh,
  Matrix,
  ComputeShader,
  UniformBuffer,
  WebGPUDataBuffer,
  Buffer,
  StorageBuffer,
  ShaderLanguage,
} from "@babylonjs/core";

let getHotCache = makeHotCache<{
  "camera-position": Vector3;
  "camera-rotation": Vector3;
}>(import.meta.url);

export class MyFirstScene extends Scene {
  private ground: GroundMesh;
  private hotCache = getHotCache();

  public frame: number = 0;
  public time: number = 0;

  constructor(engine: WebGPUEngine) {
    super(engine);
    // This creates and positions a free camera (non-mesh)
    let camera = new FlyCamera(
      "camera1",
      this.hotCache.getOrInsert(
        "camera-position",
        () => new Vector3(0, 5, -10)
      ),
      this
    );
    camera.rotation = this.hotCache.getOrInsert("camera-rotation", () =>
      Vector3.Forward()
    );
    camera.minZ = 0.01;
    camera.maxZ = 1000;
    // This targets the camera to scene origin
    // camera.setTarget(Vector3.Zero());

    // This attaches the camera to the canvas
    camera.attachControl(true);

    // This creates a light, aiming 0,1,0 - to the sky (non-mesh)
    let light = new HemisphericLight("light1", new Vector3(0, 1, 0), this);

    // Default intensity is 1. Let's dim the light a small amount
    light.intensity = 0.7;

    // Create a built-in "box" shape; with 2 segments and a height of 1.
    //this.box = MeshBuilder.CreateBox("box", {size: 2}, this);
    //this.box.material = shaderMaterial;

    this.ground = MeshBuilder.CreateGround(
      "ground",
      { width: 3.14159265359 * 2, height: 3.14159265359 * 2, subdivisions: 5 },
      this
    );
    this.ground.thinInstanceAddSelf();
    this.ground.thinInstanceAdd(
      Matrix.Translation(3.14159265359, 0.0, 3.14159265359)
    );
    this.ground.thinInstanceCount = 1;

    let shaderMaterial = new ShaderMaterial("custom", this, "custom", {
      attributes: ["uv", "position", "normal"],
      uniformBuffers: ["Scene", "Mesh", "instances"],
      // uniforms: ["iTime", "iTimeDelta", "iFrame", "worldViewProjection"],
      shaderLanguage: ShaderLanguage.WGSL,
    });
    shaderMaterial.backFaceCulling = false;
    shaderMaterial.wireframe = false;
    const myUBO = new UniformBuffer(this.getEngine());
    myUBO.addUniform("iTime", 1);
    myUBO.addUniform("iTimeDelta", 1);
    myUBO.addUniform("iFrame", 1);
    myUBO.addUniform("width", 1);
    myUBO.update();
    shaderMaterial.setUniformBuffer("myUBO", myUBO);

    ``;
    //console.log(nextPowOf2);
    shaderMaterial.onBind = (m: any) => {
      var x = Math.floor(
        255 / Vector3.Distance(camera.position, this.ground.position)
      );
      var nextSquareNum = Math.pow(Math.ceil(Math.sqrt(x)), 2);
      myUBO.updateFloat("iTime", this.time / 1000);
      myUBO.updateFloat("iTimeDelta", this.deltaTime / 1000);
      myUBO.updateFloat("iFrame", this.frame);
      myUBO.updateFloat("width", nextSquareNum);
      myUBO.update();
    };
    this.ground.material = shaderMaterial;

    let cs = new ComputeShader(
      "mycs",
      engine,
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
    const csUniformBuffer = new UniformBuffer(engine);
    csUniformBuffer.addUniform("visibleInstances", 1);
    cs.setUniformBuffer("inputBuffer", csUniformBuffer);

    let t = 0;
    let first = true;

    this.onBeforeRenderObservable.add(() => {
      this.hotCache.set("camera-position", camera.position.clone());
      this.hotCache.set("camera-rotation", camera.rotation.clone());
    });

    this.onBeforeRenderObservable.add(() => {
      // TODO: Correctly handle resetMaterials
      const renderPassId = engine.currentRenderPassId;
      const drawWrapper = this.ground.subMeshes[0]._getDrawWrapper(
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
            const buffer = new Buffer(
              engine,
              new WebGPUDataBuffer(indirectDrawBuffer, 20),
              true
            );
            // TODO: Ask babylon js peeps to add a proper API for this
            cs.setStorageBuffer("indirectDrawBuffer", buffer as any);
            first = false;
          }

          var x = Math.floor(
            255 / Vector3.Distance(camera.position, this.ground.position)
          );
          var nextSquareNum = Math.pow(Math.ceil(Math.sqrt(x)), 2);
          csUniformBuffer.updateUInt("visibleInstances", nextSquareNum);
          csUniformBuffer.update();

          cs.dispatch(1, 1, 1);
        }
      }
    });
  }
}
