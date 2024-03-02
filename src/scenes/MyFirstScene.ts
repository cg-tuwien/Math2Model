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
} from "@babylonjs/core";

// import {WebGPUEngine} from "@babylonjs/core";

// TODO: Maybe hook into Vite's hot reloading?

export class MyFirstScene extends Scene {
  private ground: GroundMesh;

  public frame: number = 0;
  public time: number = 0;

  constructor(engine: WebGPUEngine) {
    super(engine);

    // This creates and positions a free camera (non-mesh)
    let camera = new FlyCamera("camera1", new Vector3(0, 5, -10), this);

    // This targets the camera to scene origin
    // camera.setTarget(Vector3.Zero());

    // This attaches the camera to the canvas
    camera.attachControl(true);

    // This creates a light, aiming 0,1,0 - to the sky (non-mesh)
    let light = new HemisphericLight("light1", new Vector3(0, 1, 0), this);

    // Default intensity is 1. Let's dim the light a small amount
    light.intensity = 0.7;

    Effect.ShadersStore["customVertexShader"] = `
precision highp float;
attribute vec3 position;
attribute vec2 uv;
uniform mat4 worldViewProjection;

void main() {

    vec3 pos = vec3(uv.x, 0.0, uv.y) * 3.14159265359 * 2.;

    float x = sin(pos.x) * cos(pos.z);
    float y = sin(pos.x) * sin(pos.z);
    float z = cos(pos.x);

    float x2 = sin(pos.x) * (15. * sin(pos.z) - 4. * sin(3. * pos.z));
    float y2 = 8. * cos(pos.x);
    float z2 = sin(pos.x) * (15. * cos(pos.z) - 5. * cos(2. * pos.z) - 2. * cos(3. * pos.z) - cos(2. * pos.z));

    vec3 sphere = vec3(x, y, z);
    vec3 heart = vec3(x2, y2, z2);

    vec4 p = vec4(mix(sphere, heart, 0.) * 1., 1.);
    gl_Position = worldViewProjection * p;
}
    `;

    Effect.ShadersStore["customFragmentShader"] = `
        precision highp float;

        void main() {
            gl_FragColor = vec4(1.,0.,0.,1.);
        }
    `;

    let shaderMaterial = new ShaderMaterial("custom", this, "custom", {
      attributes: ["uv", "position"],
      uniforms: ["iTime", "iTimeDelta", "iFrame"],
    });
    shaderMaterial.allowShaderHotSwapping = true;
    // Create a built-in "box" shape; with 2 segments and a height of 1.
    //this.box = MeshBuilder.CreateBox("box", {size: 2}, this);
    //this.box.material = shaderMaterial;

    this.ground = MeshBuilder.CreateGround(
      "ground",
      { width: 3.14159265359 * 2, height: 3.14159265359 * 2, subdivisions: 10 },
      this
    );
    this.ground.thinInstanceAddSelf();
    this.ground.thinInstanceAdd(
      Matrix.Translation(3.14159265359, 0.0, 3.14159265359)
    );
    this.ground.thinInstanceAdd(
      Matrix.Translation(3.14159265359, 0.0, -3.14159265359)
    );
    // this.ground.material = shaderMaterial;
    this.ground.thinInstanceCount = 1;

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
      const renderPassId = engine.currentRenderPassId;
      const drawWrapper = this.ground.subMeshes[0]._getDrawWrapper(
        renderPassId,
        false
      );

      if (t++ > 30) {
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

          csUniformBuffer.updateUInt(
            "visibleInstances",
            Math.floor(Math.random() * 10) + 1
          );
          csUniformBuffer.update();

          cs.dispatch(1, 1, 1);
        }
      }
    });
  }

  // Für Hot reloading
  resetMaterials(): void {
    this.ground.material?.dispose();
    let shaderMaterial = new ShaderMaterial("custom", this, "custom", {
      attributes: ["uv", "position"],
      uniforms: ["iTime", "iTimeDelta", "iFrame", "worldViewProjection"],
    });
    // this.ground.material = shaderMaterial;
    /*this.ground.material.onBind = (m: any) => {
      shaderMaterial.setFloat("iTime", this.time / 1000);
      shaderMaterial.setFloat("iTimeDelta", this.deltaTime / 1000);
      shaderMaterial.setFloat("iFrame", this.frame);
    };*/
  }
}
