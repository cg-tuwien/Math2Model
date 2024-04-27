<template><slot></slot></template>
<script setup lang="ts">
import type {
  HasReactiveFiles,
  ReadonlyFiles,
} from "@/filesystem/reactive-files";
import type { BaseScene } from "@/scenes/BaseScene";
import type { VirtualModelState } from "@/scenes/VirtualScene";
import {
  ComputeShader,
  Constants,
  Matrix,
  MeshBuilder,
  Quaternion,
  ShaderLanguage,
  ShaderMaterial,
  StorageBuffer,
  UniformBuffer,
  Vector3,
  WebGPUDataBuffer,
  WebGPUDrawContext,
  WebGPUEngine,
  type GroundMesh,
} from "@babylonjs/core";
import { assert } from "@stefnotch/typestef/assert";
import {
  onUnmounted,
  shallowRef,
  watch,
  watchEffect,
  type DeepReadonly,
  type ShallowRef,
} from "vue";
import ComputePatches from "../../parametric-renderer-core/shaders/ComputePatches.wgsl?raw";
import CopyPatches from "../../parametric-renderer-core/shaders/CopyPatches.wgsl?raw";

const props = defineProps<{
  scene: BaseScene;
  files: ReadonlyFiles & HasReactiveFiles;
  globalUBO: UniformBuffer;
  model: DeepReadonly<VirtualModelState>;
}>();

const dummyValue = Symbol("dummyValue");
function shallowEffectRef<T>(
  effect: (oldValue: NoInfer<T> | null) => T
): ShallowRef<T> {
  const ref = shallowRef<T>(dummyValue as any);
  watchEffect(() => {
    const oldValue = ref.value;
    ref.value = effect(oldValue === dummyValue ? null : oldValue);
  });
  assert(ref.value !== dummyValue);
  return ref;
}

const onRenderCallbacks: (() => void)[] = [];
function onRender(callback: () => void) {
  onRenderCallbacks.push(callback);
}
const renderObserver = shallowEffectRef<{ dispose: () => void }>((oldValue) => {
  oldValue?.dispose();
  const observer = props.scene.onBeforeRenderObservable.add(() => {
    onRenderCallbacks.forEach((callback) => callback());
  });
  return {
    observer,
    dispose: () => {
      observer.remove();
    },
  };
});
onUnmounted(() => {
  renderObserver.value.dispose();
});

// From https://stackoverflow.com/a/70005195/3492994
function concatArrayBuffers(views: ArrayBufferView[]) {
  let length = 0;
  for (const v of views) length += v.byteLength;

  let buf = new Uint8Array(length);
  let offset = 0;
  for (const v of views) {
    const uint8view = new Uint8Array(v.buffer, v.byteOffset, v.byteLength);
    buf.set(uint8view, offset);
    offset += uint8view.byteLength;
  }

  return buf;
}

class SmartStorageBuffer {
  buffer: StorageBuffer;
  initialBuffer: StorageBuffer;
  initialByteSize: number;
  constructor(
    engine: WebGPUEngine,
    initialData: {
      buffers: ArrayBufferView[];
      runtimeArray: number;
    },
    creationFlags: number,
    label: string
  ) {
    const data = concatArrayBuffers(initialData.buffers);

    this.initialByteSize = data.byteLength;
    this.buffer = new StorageBuffer(
      engine,
      data.byteLength + initialData.runtimeArray,
      creationFlags | Constants.BUFFER_CREATIONFLAG_WRITE,
      label
    );
    this.buffer.update(data);
    this.initialBuffer = new StorageBuffer(
      engine,
      data.byteLength,
      Constants.BUFFER_CREATIONFLAG_READ | Constants.BUFFER_CREATIONFLAG_WRITE,
      label
    );
    this.initialBuffer.update(data);
  }

  reset(engine: WebGPUEngine) {
    engine._renderEncoder.copyBufferToBuffer(
      this.initialBuffer.getBuffer().underlyingResource,
      0,
      this.buffer.getBuffer().underlyingResource,
      0,
      this.initialByteSize
    );
  }

  dispose() {
    this.buffer.dispose();
    this.initialBuffer.dispose();
  }
}

// TODO: Introduce a shader caching in the layer above
// (Which keeps track of what shaders are used in the scene, and uses the fact that it can just read the required shaders from the models array)
const mesh = babylonEffectRef<GroundMesh>(() => {
  const mesh = MeshBuilder.CreateGround(
    props.model.id + "_mesh",
    {
      width: 3.14159265359 * 2,
      height: 3.14159265359 * 2,
      subdivisions: 5,
    },
    props.scene
  );
  // Needs to be set, otherwise Babylon.js will use a Vector3 property for rotation.
  mesh.rotationQuaternion = Quaternion.Identity();
  mesh.alwaysSelectAsActiveMesh = true;
  mesh.thinInstanceAddSelf();
  mesh.thinInstanceCount = 0.1; // This is a disgusting hack to tell Babylon.js "render zero instances"
  return mesh;
});
watchEffect(() => {
  mesh.value.position = props.model.position.toVector3();
  mesh.value.rotationQuaternion = props.model.rotation.toQuaternion();
  mesh.value.scaling = new Vector3(
    props.model.scale,
    props.model.scale,
    props.model.scale
  );
});

const shaderMaterial = babylonEffectRef<ShaderMaterial | null>(() => {
  // This is neccessary to make it fully reactive (otherwise it would not update when the file changes)
  const vertexSourceId = props.files.fileNames.value.get(
    props.model.code.vertexFile
  );
  if (vertexSourceId === undefined) {
    return null;
  }
  const vertexSource = props.files.readFile(props.model.code.vertexFile);
  if (vertexSource === null) {
    return null;
  }
  const fragmentSourceId = props.files.fileNames.value.get(
    props.model.code.fragmentFile
  );
  if (fragmentSourceId === undefined) {
    return null;
  }
  const fragmentSource = props.files.readFile(props.model.code.fragmentFile);
  if (fragmentSource === null) {
    return null;
  }
  const material = new ShaderMaterial(
    "custom",
    props.scene,
    {
      vertexSource: assembleFullVertexShader(vertexSource),
      fragmentSource: fragmentSource,
    },
    {
      attributes: ["uv", "position", "normal"],
      uniformBuffers: ["Scene", "Mesh", "instances"],
      shaderLanguage: ShaderLanguage.WGSL,
      storageBuffers: ["renderBuffer"],
    }
  );
  material.backFaceCulling = false;
  material.wireframe = true;
  return material;
});
watchEffect(() => {
  shaderMaterial.value?.setUniformBuffer("globalUBO", props.globalUBO);
});
watchEffect(() => {
  mesh.value.material = shaderMaterial.value;
});

const patchByteSize = 16; // 4 bytes per float, 4 floats per patch
const maxPatchCount = 100000;
const renderBuffer = babylonEffectRef<SmartStorageBuffer>(() => {
  return new SmartStorageBuffer(
    props.scene.engine,
    {
      buffers: [
        new Uint32Array([
          0, // patches_length,
          maxPatchCount, // patches_capacity
        ]),
      ],
      runtimeArray: maxPatchCount * patchByteSize,
    },
    Constants.BUFFER_CREATIONFLAG_READWRITE,
    "Render Buffer"
  );
});

const patchesInputBuffer = babylonEffectRef<UniformBuffer>(() => {
  const buffer = new UniformBuffer(props.scene.engine);
  buffer.addMatrix("model_view_projection", Matrix.Identity());
  return buffer;
});
onRender(() => {
  patchesInputBuffer.value.updateMatrix(
    "model_view_projection",
    mesh.value.getWorldMatrix().multiply(props.scene.getTransformMatrix())
  );
  patchesInputBuffer.value.update();
});

const patchesBufferReset = babylonEffectRef<StorageBuffer>(() => {
  const data = concatArrayBuffers([
    new Uint32Array([
      1, //patches_length
      maxPatchCount, //patches_capacity
    ]),
    new Float32Array([
      0.0,
      0.0,
      1.0,
      1.0, // patch 0
    ]),
  ]);
  const buffer = new StorageBuffer(
    props.scene.engine,
    data.byteLength,
    Constants.BUFFER_CREATIONFLAG_READWRITE,
    "Patches Buffer Initial Data"
  );
  buffer.update(data);
  return buffer;
});

const patchesBuffer = shallowEffectRef<
  [SmartStorageBuffer, SmartStorageBuffer]
>((oldValue) => {
  oldValue?.[0]?.dispose();
  oldValue?.[1]?.dispose();
  const buffer0 = new SmartStorageBuffer(
    props.scene.engine,
    {
      buffers: [
        new Uint32Array([
          0, //patches_length
          maxPatchCount, //patches_capacity
        ]),
      ],
      runtimeArray: maxPatchCount * patchByteSize,
    },
    Constants.BUFFER_CREATIONFLAG_READWRITE,
    "Patches Buffer 0"
  );
  const buffer1 = new SmartStorageBuffer(
    props.scene.engine,
    {
      buffers: [
        new Uint32Array([
          0, //patches_length
          maxPatchCount, //patches_capacity
        ]),
      ],
      runtimeArray: maxPatchCount * patchByteSize,
    },
    Constants.BUFFER_CREATIONFLAG_READWRITE,
    "Patches Buffer 1"
  );
  return [buffer0, buffer1];
});
onUnmounted(() => {
  patchesBuffer.value[0].dispose();
  patchesBuffer.value[1].dispose();
});

const indirectComputeBuffer = shallowEffectRef<
  [SmartStorageBuffer, SmartStorageBuffer]
>((oldValue) => {
  oldValue?.[0]?.dispose();
  oldValue?.[1]?.dispose();
  const buffer0 = new SmartStorageBuffer(
    props.scene.engine,
    {
      buffers: [
        new Uint32Array([
          1, // x
          1, // y
          1, // z
        ]),
      ],
      runtimeArray: 0,
    },
    Constants.BUFFER_CREATIONFLAG_READWRITE |
      Constants.BUFFER_CREATIONFLAG_INDIRECT,
    "Indirect Compute Buffer 0"
  );
  const buffer1 = new SmartStorageBuffer(
    props.scene.engine,
    {
      buffers: [
        new Uint32Array([
          1, // x
          1, // y
          1, // z
        ]),
      ],
      runtimeArray: 0,
    },
    Constants.BUFFER_CREATIONFLAG_READWRITE |
      Constants.BUFFER_CREATIONFLAG_INDIRECT,
    "Indirect Compute Buffer 1"
  );
  return [buffer0, buffer1];
});
onUnmounted(() => {
  indirectComputeBuffer.value[0].dispose();
  indirectComputeBuffer.value[1].dispose();
});

const computePatchesShader = shallowEffectRef<[ComputeShader, ComputeShader]>(
  () => {
    // Apparently the compute shader cannot be disposed of.
    const cs0 = new ComputeShader(
      "Compute Patches 0",
      props.scene.engine,
      {
        computeSource: ComputePatches,
      },
      {
        bindingsMapping: {
          input_buffer: { group: 0, binding: 0 },
          patches_from_buffer: { group: 0, binding: 1 },
          patches_to_buffer: { group: 0, binding: 2 },
          render_buffer: { group: 0, binding: 3 },
          dispatch_next: { group: 0, binding: 4 },
        },
      }
    );
    const cs1 = new ComputeShader(
      "Compute Patches 1",
      props.scene.engine,
      {
        computeSource: ComputePatches,
      },
      {
        bindingsMapping: {
          input_buffer: { group: 0, binding: 0 },
          patches_from_buffer: { group: 0, binding: 1 },
          patches_to_buffer: { group: 0, binding: 2 },
          render_buffer: { group: 0, binding: 3 },
          dispatch_next: { group: 0, binding: 4 },
        },
      }
    );
    return [cs0, cs1];
  }
);
watchEffect(() => {
  computePatchesShader.value[0].setUniformBuffer(
    "input_buffer",
    patchesInputBuffer.value
  );
  computePatchesShader.value[1].setUniformBuffer(
    "input_buffer",
    patchesInputBuffer.value
  );
  computePatchesShader.value[0].setStorageBuffer(
    "render_buffer",
    renderBuffer.value.buffer
  );
  computePatchesShader.value[1].setStorageBuffer(
    "render_buffer",
    renderBuffer.value.buffer
  );
});

watchEffect(() => {
  computePatchesShader.value[0].setStorageBuffer(
    "patches_from_buffer",
    patchesBuffer.value[0].buffer
  );
  computePatchesShader.value[0].setStorageBuffer(
    "patches_to_buffer",
    patchesBuffer.value[1].buffer
  );
  computePatchesShader.value[0].setStorageBuffer(
    "dispatch_next",
    indirectComputeBuffer.value[1].buffer
  );
  //
  computePatchesShader.value[1].setStorageBuffer(
    "patches_from_buffer",
    patchesBuffer.value[1].buffer
  );
  computePatchesShader.value[1].setStorageBuffer(
    "patches_to_buffer",
    patchesBuffer.value[0].buffer
  );
  computePatchesShader.value[1].setStorageBuffer(
    "dispatch_next",
    indirectComputeBuffer.value[0].buffer
  );
});

const copyPatchesShader = shallowEffectRef<ComputeShader>(() => {
  const cs = new ComputeShader(
    "Copy Patches",
    props.scene.engine,
    {
      computeSource: CopyPatches,
    },
    {
      bindingsMapping: {
        indirect_draw_buffer: { group: 0, binding: 0 },
        patches_from_buffer: { group: 0, binding: 1 },
        render_buffer: { group: 0, binding: 2 },
      },
    }
  );
  return cs;
});
watchEffect(() => {
  copyPatchesShader.value.setStorageBuffer(
    "patches_from_buffer",
    patchesBuffer.value[0].buffer
  );
  copyPatchesShader.value.setStorageBuffer(
    "render_buffer",
    renderBuffer.value.buffer
  );
});

watchEffect(() => {
  shaderMaterial.value?.setStorageBuffer(
    "renderBuffer",
    renderBuffer.value.buffer
  );
});

let lastIndirectDrawBuffer: GPUBuffer | null = null;
watch(copyPatchesShader, () => {
  lastIndirectDrawBuffer = null;
});

onRender(() => {
  const engine = props.scene.engine;

  // Record commands only
  indirectComputeBuffer.value[0].reset(engine);
  indirectComputeBuffer.value[1].reset(engine);
  renderBuffer.value.reset(engine);

  engine._renderEncoder.copyBufferToBuffer(
    patchesBufferReset.value.getBuffer().underlyingResource,
    0,
    patchesBuffer.value[0].buffer.getBuffer().underlyingResource,
    0,
    patchesBufferReset.value.getBuffer().capacity
  );

  // Subdivide the patches
  for (let i = 0; i < 4; i++) {
    {
      patchesBuffer.value[1].reset(engine);
      computePatchesShader.value[0].dispatchIndirect(
        indirectComputeBuffer.value[0].buffer
      );
    }
    {
      patchesBuffer.value[0].reset(engine);
      computePatchesShader.value[1].dispatchIndirect(
        indirectComputeBuffer.value[1].buffer
      );
    }
  }

  // Copy the patches, and trigger the rendering
  {
    // We're hooking into Babylon.js internals here for indirect drawing.
    const renderPassId = props.scene.engine.currentRenderPassId;
    const drawWrapper = mesh.value.subMeshes[0]._getDrawWrapper(
      renderPassId,
      false
    );
    const indirectDrawBuffer = (
      drawWrapper?.drawContext as any as WebGPUDrawContext
    )?.indirectDrawBuffer;
    if (!indirectDrawBuffer) {
      return;
    }
    if (lastIndirectDrawBuffer !== indirectDrawBuffer) {
      copyPatchesShader.value.setStorageBuffer(
        "indirect_draw_buffer",
        new WebGPUDataBuffer(indirectDrawBuffer, 20)
      );
      lastIndirectDrawBuffer = indirectDrawBuffer;
    }
  }

  copyPatchesShader.value.dispatchIndirect(
    indirectComputeBuffer.value[0].buffer
  );

  // And Babylon.js will submit the commands
});

type HasDispose = { dispose: () => void };
function babylonEffectRef<T extends HasDispose | null>(
  effect: () => T
): ShallowRef<T> {
  const ref = shallowEffectRef<T>((oldValue) => {
    oldValue?.dispose();
    return effect();
  });
  onUnmounted(() => {
    ref.value?.dispose();
  });
  return ref;
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

struct RenderBufferRead {
  patches_length: u32, // Same size as atomic<u32>
  patches_capacity: u32,
  patches: array<Patch>,
};

var<uniform> globalUBO: GlobalUBO;
var<storage, read> renderBuffer: RenderBufferRead;

${innerCode}

@vertex
fn main(input: VertexInputs) -> FragmentInputs {
  let quad = renderBuffer.patches[input.instanceIndex];

  var uv = vec2<f32>(quad.min.x, quad.min.y);
  if (input.vertexIndex == 0) {
      uv = vec2<f32>(quad.min.x, quad.min.y);
  } else if (input.vertexIndex == 1) {
      uv = vec2<f32>(quad.max.x, quad.min.y);
  } else if (input.vertexIndex == 2) {
      uv = vec2<f32>(quad.max.x, quad.max.y);
  } else if (input.vertexIndex == 3) {
      uv = vec2<f32>(quad.min.x, quad.max.y);
  }

  vertexInputs.uv = uv;

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
</script>
