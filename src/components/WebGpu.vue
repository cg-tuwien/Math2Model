<script setup lang="ts">
// Taken from https://webgpu.github.io/webgpu-samples/?sample=rotatingCube#main.ts
// TODO: Either add attribution, or remove this file.
import type { WgpuEngine } from "@/engine/wgpu-engine";
import type { FilePath, ReactiveFilesystem } from "@/filesystem/reactive-files";
import type { LodStageBuffers } from "@/webgpu-hook";
import { ref } from "vue";
import { mat4, vec3 } from "webgpu-matrix";
import LodStageWgsl from "./lod_stage.wgsl?raw";

// Unchanging props! No need to watch them.
const props = defineProps<{
  gpuDevice: GPUDevice;
  engine: WgpuEngine;
  fs: ReactiveFilesystem;
}>();

const compiledShaders = ref<Map<FilePath, GPUShaderModule>>(new Map());
async function makeShader(fs: ReactiveFilesystem, key: FilePath) {
  // Race condition free, because the fs resolves read requests in order
  let code = await fs.readTextFile(key);
  if (code === null) {
    code = LodStageWgsl; // Fallback to the default shader
  } else {
    code = LodStageWgsl.replace(
      /\/\/\/\/ START sampleObject([^]+?)\/\/\/\/ END sampleObject/,
      code
    );
  }
  const shader = props.gpuDevice.createShaderModule({
    code,
  });
  shader.getCompilationInfo().then((info) => {
    console.log("Shader compilation resulted in ", info);
  });
  return shader;
}

function concatArrayBuffers(views: ArrayBufferView[]): Uint8Array {
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

function createBufferWith(
  contents: Uint8Array,
  usage: GPUBufferUsageFlags,
  size?: number
) {
  const buffer = props.gpuDevice.createBuffer({
    size: size ?? contents.byteLength,
    usage,
    mappedAtCreation: true,
  });
  new Uint8Array(buffer.getMappedRange()).set(contents);
  buffer.unmap();
  return buffer;
}

async function main() {
  const _adapter = await navigator.gpu.requestAdapter(); // Wait for Rust backend to be in business
  const device = props.gpuDevice;
  props.fs.watchFromStart((change) => {
    if (!change.key.endsWith(".wgsl")) return;
    if (change.type === "insert" || change.type === "update") {
      makeShader(props.fs, change.key).then((shader) => {
        compiledShaders.value.set(change.key, shader);
      });
    } else {
      compiledShaders.value.delete(change.key);
    }
  });

  const patchesBufferReset = createBufferWith(
    concatArrayBuffers([new Uint32Array([0, 0])]),
    GPUBufferUsage.COPY_SRC
  );

  const indirectComputeBufferReset = createBufferWith(
    concatArrayBuffers([new Uint32Array([0, 1, 1])]),
    GPUBufferUsage.COPY_SRC
  );

  const forceRenderFalse = createBufferWith(
    concatArrayBuffers([new Uint32Array([0])]),
    GPUBufferUsage.COPY_SRC
  );

  const forceRenderTrue = createBufferWith(
    concatArrayBuffers([new Uint32Array([1])]),
    GPUBufferUsage.COPY_SRC
  );

  function lodStageCallback(
    shaderPath: string,
    buffers: LodStageBuffers,
    commandEncoder: GPUCommandEncoder
  ) {
    const doubleNumberOfRounds = 4;
    for (let i = 0; i < doubleNumberOfRounds; i++) {
      const isLastRound = i === doubleNumberOfRounds - 1;
      // Ping
      commandEncoder.copyBufferToBuffer(
        patchesBufferReset,
        0,
        buffers.patches1,
        0,
        patchesBufferReset.size
      );
      commandEncoder.copyBufferToBuffer(
        indirectComputeBufferReset,
        0,
        buffers.indirectDispatch1,
        0,
        indirectComputeBufferReset.size
      );

      let computePassPing = commandEncoder.beginComputePass();
      // TODO: Pipeline
      // computePass.setBindGroup(0, TODO:)
      computePassPing.dispatchWorkgroupsIndirect(buffers.indirectDispatch0, 0);
      computePassPing.end();

      // Pong
      if (isLastRound) {
        commandEncoder.copyBufferToBuffer(
          forceRenderTrue,
          0,
          buffers.forceRender,
          0,
          forceRenderTrue.size
        );
      }

      commandEncoder.copyBufferToBuffer(
        patchesBufferReset,
        0,
        buffers.patches0,
        0,
        patchesBufferReset.size
      );
      commandEncoder.copyBufferToBuffer(
        indirectComputeBufferReset,
        0,
        buffers.indirectDispatch0,
        0,
        indirectComputeBufferReset.size
      );

      let computePassPong = commandEncoder.beginComputePass();
      // TODO: Pipeline
      // computePass.setBindGroup(0, TODO:)
      computePassPong.dispatchWorkgroupsIndirect(buffers.indirectDispatch1, 0);
      computePassPong.end();

      if (isLastRound) {
        commandEncoder.copyBufferToBuffer(
          forceRenderFalse,
          0,
          buffers.forceRender,
          0,
          forceRenderFalse.size
        );
      }
    }
  }

  props.engine.setLodStage(lodStageCallback);
}
main();
</script>
<template>
  <div></div>
</template>
