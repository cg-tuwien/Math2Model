import {
  AbstractMesh,
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
  StorageBuffer,
  Constants,
} from "@babylonjs/core";
import { type ModelDisplayVirtualScene } from "./ModelDisplayVirtualScene";
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
    this._mesh.alwaysSelectAsActiveMesh = true;

    this._mesh.thinInstanceAddSelf();
    this._mesh.thinInstanceCount = 0.1; // This is a disgusting hack to tell Babylon.js "render zero instances"

    let vertexSource = files.readVertexShader(this.code.vertexFile);
    let shaderMaterial = this._disposables.addDisposable(
      new ShaderMaterial(
        "custom",
        scene,
        {
          vertexSource: assembleFullVertexShader(vertexSource),
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
    shaderMaterial.setUniformBuffer("globalUBO", virtualScene.globalUBO);
    shaderMaterial.wireframe = true;
    this._mesh.material = shaderMaterial;

    // Storage buffer with the patches (stored as 4 floats per patch, aka min-x, min-y, max-x, max-y)
    // TODO: Use a smarter encoding
    const readCounterBytes = 4 + 4; // A read start and a read end counter
    const atomicCounterBytes = 4; // A write counter
    const patchSize = 4 * 4; // 4 bytes per float, 4 floats per patch
    const maxPatchCount = 10_000;
    const patchesBytes = 4 + patchSize * maxPatchCount; // 4 bytes for the length, 10_00 patches
    const patchesBuffer = this._disposables.addDisposable(
      new StorageBuffer(
        scene.engine,
        readCounterBytes + atomicCounterBytes + patchesBytes,
        Constants.BUFFER_CREATIONFLAG_READWRITE
      )
    );
    const patchesBufferBytes = new Float32Array([
      0,
      0,
      0,
      0,
      0, // patch 0
      0, // patch 0
      1, // patch 0
      1, // patch 0
    ]);
    new Uint32Array(patchesBufferBytes.buffer).set([
      0, // read start
      1, // read end (exclusive)
      1, // write
      maxPatchCount, // patchesLength
    ]);

    const renderBuffer = this._disposables.addDisposable(
      new StorageBuffer(
        scene.engine,
        atomicCounterBytes + patchSize * maxPatchCount,
        Constants.BUFFER_CREATIONFLAG_READWRITE
        // Constants.BUFFER_CREATIONFLAG_VERTEX
      )
    );
    const renderBufferBytes = new Uint32Array([
      0, // instanceCount
      maxPatchCount, // patchesLength
    ]);
    const structPatches = `
    struct Patch {
      min: vec2<f32>,
      max: vec2<f32>,
  };

  struct Patches {
    readStart: u32,
    readEnd: u32,
    write: atomic<u32>,
    patchesLength: u32,
    patches : array<Patch>,
  };
    `;
    const structRenderBuffer = `
    struct RenderBuffer {
      instanceCount: atomic<u32>,
      patchesLength: u32,
      patches: array<Patch>,
    };
  `;

    let cs = new ComputeShader(
      "subdivide-cs",
      scene.engine,
      {
        computeSource: `
            struct InputBuffer {
                modelViewProjection: mat4x4<f32>,
            };

            ${structPatches}
            ${structRenderBuffer}

            @group(0) @binding(1) var<uniform> inputBuffer : InputBuffer;
            @group(0) @binding(2) var<storage, read_write> patchesBuffer : Patches;
            @group(0) @binding(3) var<storage, read_write> renderBuffer : RenderBuffer;

            ${vertexSource}

            fn triangleArea(a: vec3f, b: vec3f, c: vec3f) -> f32 {
              return 0.5 * length(cross(b - a, c - a));
            }

            // assume a single work group
            @compute @workgroup_size(64, 1, 1)
            fn main(@builtin(global_invocation_id) global_id : vec3<u32>) {
              // TODO: Do loops in compute shaders make sense?
              for (var i = 0u; i < 8u; i = i + 1u) {

              let patchIndex = global_id.x + patchesBuffer.readStart;
              if (patchIndex < patchesBuffer.readEnd) {
                let quad = patchesBuffer.patches[patchIndex];

                let corners = array<vec3f, 4>(
                  evaluateImage(vec2f(quad.min.x, quad.min.y)),
                  evaluateImage(vec2f(quad.max.x, quad.min.y)),
                  evaluateImage(vec2f(quad.max.x, quad.max.y)),
                  evaluateImage(vec2f(quad.min.x, quad.max.y))
                );
                
                let cornersClipSpace = array<vec4f, 4>(
                  (inputBuffer.modelViewProjection * vec4f(corners[0], 1.0)),
                  (inputBuffer.modelViewProjection * vec4f(corners[1], 1.0)),
                  (inputBuffer.modelViewProjection * vec4f(corners[2], 1.0)),
                  (inputBuffer.modelViewProjection * vec4f(corners[3], 1.0))
                );
                // TODO: Clipping (aka discard if outside of the frustum)
                let cornersScreenSpace = array<vec2f, 4>(
                  cornersClipSpace[0].xy / cornersClipSpace[0].w,
                  cornersClipSpace[1].xy / cornersClipSpace[1].w,
                  cornersClipSpace[2].xy / cornersClipSpace[2].w,
                  cornersClipSpace[3].xy / cornersClipSpace[3].w
                );
                let cornersForArea = array<vec3f, 4>(
                  vec3f(cornersScreenSpace[0], 0.0),
                  vec3f(cornersScreenSpace[1], 0.0),
                  vec3f(cornersScreenSpace[2], 0.0),
                  vec3f(cornersScreenSpace[3], 0.0)
                );
                var area = triangleArea(cornersForArea[0], cornersForArea[1], cornersForArea[2]) + 
                triangleArea(cornersForArea[0], cornersForArea[2], cornersForArea[3]);
                
                let sizeThreshold = 0.05; // TODO: Will depend on the screen resolution (and we need different thresholds for X and Y)

                // Bonus check for degenerate cases
                let centerSpot = inputBuffer.modelViewProjection * vec4f(evaluateImage(mix(quad.min, quad.max, 0.5)), 1.0);
                if (distance(centerSpot.xyz / centerSpot.w, cornersClipSpace[0].xyz / cornersClipSpace[0].w) > 0.1) { // In screen space + depth
                  area = 2 * sizeThreshold;
                }

                if (area < sizeThreshold * sizeThreshold) {
                  // TODO: Should we do instancing, or should we directly generate vertices here?
                  // Done, please render
                  let writeIndex = min(atomicAdd(&renderBuffer.instanceCount, 1), renderBuffer.patchesLength - 1);
                  renderBuffer.patches[writeIndex] = quad;
                } else {
                  // Split in four
                  // TODO: Properly handle overflow
                  let writeIndex = min(atomicAdd(&patchesBuffer.write, 4), patchesBuffer.patchesLength - 4);

                  let center = mix(quad.min, quad.max, 0.5);
                  patchesBuffer.patches[writeIndex + 0] = Patch(quad.min, center);
                  patchesBuffer.patches[writeIndex + 1] = Patch(vec2f(center.x, quad.min.y), vec2f(quad.max.x, center.y));
                  patchesBuffer.patches[writeIndex + 2] = Patch(center, quad.max);
                  patchesBuffer.patches[writeIndex + 3] = Patch(vec2f(quad.min.x, center.y), vec2f(center.x, quad.max.y));
                }
              }
              // TODO: What's the most efficient approach (multiple buffers? start-end? etc.)
              storageBarrier(); // Wait for all threads to finish reading "readStart" and "readEnd"
              if(global_id.x == 0u && global_id.y == 0u && global_id.z == 0u) {
                patchesBuffer.readStart = patchesBuffer.readEnd;
                patchesBuffer.readEnd = min(patchesBuffer.readEnd + 64, atomicLoad(&patchesBuffer.write));
              }
              storageBarrier(); // Wait until the writes are done before continuing

              } // end of for loop
            }
        `,
      },
      {
        bindingsMapping: {
          inputBuffer: { group: 0, binding: 1 },
          patchesBuffer: { group: 0, binding: 2 },
          renderBuffer: { group: 0, binding: 3 },
        },
      }
    );
    cs.setStorageBuffer("patchesBuffer", patchesBuffer);
    cs.setStorageBuffer("renderBuffer", renderBuffer);

    const csUniformBuffer = this._disposables.addDisposable(
      new UniformBuffer(scene.engine)
    );
    csUniformBuffer.addMatrix("modelViewProjection", Matrix.Identity());
    cs.setUniformBuffer("inputBuffer", csUniformBuffer);

    const csUniformBufferUpdate = scene.onBeforeRenderObservable.add(() => {
      csUniformBuffer.updateMatrix(
        "modelViewProjection",
        this._mesh.getWorldMatrix().multiply(scene.getTransformMatrix())
      );
      csUniformBuffer.update();
    });
    this._disposables.addDisposable({
      dispose: () => {
        scene.onBeforeRenderObservable.remove(csUniformBufferUpdate);
      },
    });

    let renderCs = new ComputeShader(
      "render-cs",
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

            ${structPatches}
            ${structRenderBuffer}

            @group(0) @binding(0) var<storage, read_write> indirectDrawBuffer : IndirectDrawBuffer;
            @group(0) @binding(2) var<storage, read_write> patchesBuffer : Patches;
            @group(0) @binding(3) var<storage, read_write> renderBuffer : RenderBuffer;

            fn ceilDiv(a: u32, b: u32) -> u32 {
              return (a + b - 1u) / b;
            }

            @compute @workgroup_size(64, 1, 1)
            fn main(@builtin(global_invocation_id) global_id : vec3<u32>) {
              let renderBufferStart = atomicLoad(&renderBuffer.instanceCount);

              let start = patchesBuffer.readStart;
              var end = atomicLoad(&patchesBuffer.write);
              end = min(end, min(end - start, renderBuffer.patchesLength) + start);

              // Split into 64 segments (workgroup size)
              let segmentSize = ceilDiv(end - start, 64);
              let segmentStart = start + segmentSize * global_id.x;
              let segmentEnd = min(segmentStart + segmentSize, end);

              // Copy patches to render buffer
              // Notice how we know where to put everything, so no need for synchronization with atomics
              for (var i = segmentStart; i < segmentEnd; i = i + 1u) {
                let quad = patchesBuffer.patches[i];
                renderBuffer.patches[renderBufferStart + (i - start)] = quad;
              }

              if(global_id.x == 0u && global_id.y == 0u && global_id.z == 0u) {
                indirectDrawBuffer.instanceCount = renderBufferStart + (end - start);
              }
            }
        `,
      },
      {
        bindingsMapping: {
          indirectDrawBuffer: { group: 0, binding: 0 },
          patchesBuffer: { group: 0, binding: 2 },
          renderBuffer: { group: 0, binding: 3 },
        },
      }
    );
    renderCs.setStorageBuffer("patchesBuffer", patchesBuffer);
    renderCs.setStorageBuffer("renderBuffer", renderBuffer);

    shaderMaterial.setStorageBuffer("renderBuffer", renderBuffer);

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
        renderCs.setStorageBuffer(
          "indirectDrawBuffer",
          new WebGPUDataBuffer(indirectDrawBuffer, 20)
        );
        first = false;
      }
      patchesBuffer.update(patchesBufferBytes);
      renderBuffer.update(renderBufferBytes);

      cs.dispatch(1, 1, 1);
      // cs.dispatch(1, 1, 1);
      // cs.dispatch(1, 1, 1);
      renderCs.dispatch(1, 1, 1);
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

function assembleFullVertexShader(innerCode: string) {
  return `
#include<sceneUboDeclaration>
#include<meshUboDeclaration>

attribute position : vec3<f32>;
attribute uv: vec2<f32>;
attribute normal : vec3<f32>;

varying vUV : vec2<f32>;
varying vNormal : vec3<f32>;

struct GlobalUBO {
  iTime: f32,
  iTimeDelta: f32,
  iFrame: f32,
};

struct Patch {
  min: vec2<f32>,
  max: vec2<f32>,
};

struct RenderBuffer {
  _skipInstanceCount: u32, // Same size as the atomic<u32>, see https://www.w3.org/TR/WGSL/#alignment-and-size
  patchesLength: u32,
  patches : array<Patch>,
};

var<uniform> globalUBO: GlobalUBO;
var<storage, read> renderBuffer: RenderBuffer;

${innerCode}

@vertex
fn main(input: VertexInputs) -> FragmentInputs {
  let quad = renderBuffer.patches[input.instanceIndex];

  vertexInputs.uv = array<vec2<f32>, 4>(
    vec2<f32>(quad.min.x, quad.min.y),
    vec2<f32>(quad.max.x, quad.min.y),
    vec2<f32>(quad.max.x, quad.max.y),
    vec2<f32>(quad.min.x, quad.max.y)
  )[input.vertexIndex];

    ${
      innerCode !== ""
        ? "let pos = evaluateImage(vertexInputs.uv);"
        : "let pos = vec3f(vertexInputs.uv, 0.0);"
    }
    vertexOutputs.position = scene.projection * scene.view * mesh.world * vec4f(pos,1.);
    vertexOutputs.vUV = vertexInputs.uv;
    vertexOutputs.vNormal = vertexInputs.normal;
}`;
}
