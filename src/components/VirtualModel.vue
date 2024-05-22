<template><slot></slot></template>
<script setup lang="ts">
import type {
  HasReactiveFiles,
  ReadonlyFiles,
} from "@/filesystem/reactive-files";
import type { BabylonBaseScene } from "@/scenes/BaseScene";
import type {
  ReadonlyEulerAngles,
  ReadonlyQuaternion,
  ReadonlyVector3,
  VirtualModelState,
} from "@/scenes/VirtualScene";
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
import PbrShader from "../../parametric-renderer-core/shaders/Shader.wgsl?raw";

const props = defineProps<{
  scene: BabylonBaseScene;
  files: ReadonlyFiles & HasReactiveFiles;
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

function replaceAutogen(shader: string, innerCode: string | null) {
  const autogenRegex = /\/\/ AUTOGEN[^]+?\/\/ END OF AUTOGEN/g;

  if (innerCode === null || innerCode === "") {
    innerCode = `
fn evaluateImage(input2: vec2f) -> vec3f {
  return vec3f(input2, 0.0);
}
}
`;
  }

  return shader.replaceAll(autogenRegex, (v) => {
    if (/fn\s+evaluateImage/.test(v)) {
      return innerCode;
    } else {
      return v;
    }
  });
}
function assembleShaders(innerCode: string): {
  vertexShader: string;
  fragmentShader: string;
} {
  const shader = replaceAutogen(PbrShader, innerCode);
  const vertexRegex = /\/\/\/ VERTEX[^]+?\/\/\/ END VERTEX/g;
  const fragmentRegex = /\/\/\/ FRAGMENT[^]+?\/\/\/ END FRAGMENT/g;

  return {
    vertexShader: shader
      .replace(fragmentRegex, "")
      .replace("fn vs_main", "fn main"),
    fragmentShader: shader
      .replace(vertexRegex, "")
      .replace("fn fs_main", "fn main"),
  };
}

function assembleComputeShader(innerCode: string | null) {
  return replaceAutogen(ComputePatches, innerCode);
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
  mesh.value.position = toVector3(props.model.position);
  mesh.value.rotationQuaternion = eulerToQuaternion(props.model.rotation);
  mesh.value.scaling = new Vector3(
    props.model.scale,
    props.model.scale,
    props.model.scale
  );
});

const shaderMaterial = babylonEffectRef<ShaderMaterial | null>(() => {
  // This is neccessary to make it fully reactive (otherwise it would not update when the file changes)
  const vertexSourceId = props.files.fileNames.value.get(props.model.code);
  if (vertexSourceId === undefined) {
    return null;
  }
  const vertexSource = props.files.readFile(props.model.code);
  if (vertexSource === null) {
    return null;
  }

  const shaders = assembleShaders(vertexSource);
  const material = new ShaderMaterial(
    "custom",
    props.scene,
    {
      vertexSource: shaders.vertexShader,
      fragmentSource: shaders.fragmentShader,
    },
    {
      attributes: ["uv", "position", "normal", "camera", "model"],
      uniformBuffers: ["Scene", "Mesh", "instances"],
      shaderLanguage: ShaderLanguage.WGSL,
      storageBuffers: ["lights", "render_buffer"],
    }
  );
  material.backFaceCulling = false;
  // material.wireframe = true;
  return material;
});
const materialUbo = babylonEffectRef<UniformBuffer>(() => {
  const buffer = new UniformBuffer(props.scene.engine);
  buffer.addColor4(
    "color_roughness",
    {
      r: 0,
      g: 0,
      b: 0,
    },
    0
  );
  buffer.addColor4(
    "emissive_metallic",
    {
      r: 0,
      g: 0,
      b: 0,
    },
    0
  );
  return buffer;
});
watchEffect(() => {
  shaderMaterial.value?.setUniformBuffer("global_ubo", props.scene.globalUBO);
});
watchEffect(() => {
  shaderMaterial.value?.setUniformBuffer("material", materialUbo.value);
});
// TODO:
/*
@group(0) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(1) var<storage, read> lights: Lights;
@group(0) @binding(2) var<uniform> model: Model;
@group(0) @binding(4) var<uniform> material: Material;
  */

watchEffect(() => {
  materialUbo.value.updateColor4(
    "color_roughness",
    {
      r: props.model.material.color.x,
      g: props.model.material.color.y,
      b: props.model.material.color.z,
    },
    props.model.material.roughness
  );
});
watchEffect(() => {
  materialUbo.value.updateColor4(
    "emissive_metallic",
    {
      r: props.model.material.emissive.x,
      g: props.model.material.emissive.y,
      b: props.model.material.emissive.z,
    },
    props.model.material.metallic
  );
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
    const vertexSourceId = props.files.fileNames.value.get(props.model.code);
    const vertexSource =
      vertexSourceId === undefined
        ? null
        : props.files.readFile(props.model.code);

    const computeSource = assembleComputeShader(vertexSource);
    // Apparently the compute shader cannot be disposed of.
    const cs0 = new ComputeShader(
      "Compute Patches 0",
      props.scene.engine,
      {
        computeSource,
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
        computeSource,
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
    "render_buffer",
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

function toVector3(v: ReadonlyVector3) {
  return new Vector3(v.x, v.y, v.z);
}
function toQuaternion(v: ReadonlyQuaternion) {
  return new Quaternion(v.x, v.y, v.z, v.w);
}
function eulerToQuaternion(v: ReadonlyEulerAngles) {
  return Quaternion.FromEulerAngles(v.x, v.y, v.z);
}
</script>
