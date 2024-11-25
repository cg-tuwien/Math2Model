<script setup lang="ts">
// Taken from https://webgpu.github.io/webgpu-samples/?sample=rotatingCube#main.ts
// TODO: Either add attribution, or remove this file.
import type { WgpuEngine } from "@/engine/wgpu-engine";
import {
  makeFilePath,
  type FilePath,
  type ReactiveFilesystem,
} from "@/filesystem/reactive-files";
import type { LodStageBuffers } from "@/webgpu-hook";
import { onUnmounted, ref } from "vue";
import { mat4, vec3 } from "webgpu-matrix";
import LodStageWgsl from "./lod_stage.wgsl?raw";

// Unchanging props! No need to watch them.
const props = defineProps<{
  gpuDevice: GPUDevice;
  engine: WgpuEngine;
  fs: ReactiveFilesystem;
}>();

const compiledShaders = ref<
  Map<
    FilePath,
    {
      shader: GPUShaderModule;
      pipeline: GPUComputePipeline;
    }
  >
>(new Map());

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

  const layout0 = device.createBindGroupLayout({
    entries: [
      {
        binding: 0,
        visibility: GPUShaderStage.COMPUTE,
        buffer: {
          type: "uniform",
        },
      },
      {
        binding: 1,
        visibility: GPUShaderStage.COMPUTE,
        buffer: {
          type: "uniform",
        },
      },
      {
        binding: 2,
        visibility: GPUShaderStage.COMPUTE,
        buffer: {
          type: "uniform",
        },
      },
    ],
  });
  const layout1 = device.createBindGroupLayout({
    entries: [
      {
        binding: 0,
        visibility: GPUShaderStage.COMPUTE,
        buffer: {
          type: "uniform",
        },
      },
      {
        binding: 1,
        visibility: GPUShaderStage.COMPUTE,
        buffer: {
          type: "storage",
        },
      },
      {
        binding: 2,
        visibility: GPUShaderStage.COMPUTE,
        buffer: {
          type: "storage",
        },
      },
      {
        binding: 3,
        visibility: GPUShaderStage.COMPUTE,
        buffer: {
          type: "storage",
        },
      },
      {
        binding: 4,
        visibility: GPUShaderStage.COMPUTE,
        buffer: {
          type: "storage",
        },
      },
      {
        binding: 5,
        visibility: GPUShaderStage.COMPUTE,
        buffer: {
          type: "storage",
        },
      },
    ],
  });
  const layout2 = device.createBindGroupLayout({
    entries: [
      {
        binding: 0,
        visibility: GPUShaderStage.COMPUTE,
        buffer: {
          type: "storage",
        },
      },
      {
        binding: 1,
        visibility: GPUShaderStage.COMPUTE,
        buffer: {
          type: "read-only-storage",
        },
      },
      {
        binding: 2,
        visibility: GPUShaderStage.COMPUTE,
        buffer: {
          type: "storage",
        },
      },
      {
        binding: 3,
        visibility: GPUShaderStage.COMPUTE,
        buffer: {
          type: "uniform",
        },
      },
    ],
  });
  const pipelineLayout = device.createPipelineLayout({
    bindGroupLayouts: [layout0, layout1, layout2],
  });
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
    const pipeline = props.gpuDevice.createComputePipeline({
      layout: pipelineLayout,
      compute: {
        module: shader,
        entryPoint: "main",
      },
    });
    return { shader, pipeline };
  }
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

  const patchesBufferDebug = createBufferWith(
    // Fill however many slots you want to debug print
    concatArrayBuffers([new Uint32Array([0, 0, 0, 0, 0, 0, 0, 0])]),
    GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ
  );
  let hasDebugInfo = false;
  const debugPrintBuffer = (
    buffer: GPUBuffer,
    commandEncoder: GPUCommandEncoder
  ) => {
    if (!hasDebugInfo) {
      hasDebugInfo = true;
      const size = Math.min(patchesBufferDebug.size, buffer.size);
      commandEncoder.copyBufferToBuffer(buffer, 0, patchesBufferDebug, 0, size);
      // Need to wait until *after* submit has happened to be able to call mapAsync
      setTimeout(() => {
        patchesBufferDebug.mapAsync(GPUMapMode.READ, 0, size).then(() => {
          const arrayBuffer = patchesBufferDebug
            .getMappedRange(0, size)
            .slice(0);
          const uint32Array = new Uint32Array(arrayBuffer);
          console.log("patchesBufferDebug", uint32Array);
          patchesBufferDebug.unmap();
          hasDebugInfo = false;
        });
      }, 0);
    }
  };

  const MAX_PATCHES = 10_000;
  const patchesBufferReset = createBufferWith(
    concatArrayBuffers([new Uint32Array([0, MAX_PATCHES])]),
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
    debugger;
    const compiledShader = compiledShaders.value.get(makeFilePath(shaderPath));
    if (!compiledShader) {
      console.error("Shader not found: ", shaderPath);
      return;
    }
    const pipeline = compiledShader.pipeline;

    // We inefficiently recreate the bind groups every time.
    const userInputBindGroup = device.createBindGroup({
      layout: layout0,
      entries: [
        {
          binding: 0,
          resource: { buffer: buffers.time },
        },
        {
          binding: 1,
          resource: { buffer: buffers.screen },
        },
        {
          binding: 2,
          resource: { buffer: buffers.mouse },
        },
      ],
    });

    const toRenderPatchesBindGroup = device.createBindGroup({
      layout: layout1,
      entries: [
        {
          binding: 0,
          resource: { buffer: buffers.computePatchesInput },
        },
        {
          binding: 1,
          resource: { buffer: buffers.finalPatches2 },
        },
        {
          binding: 2,
          resource: { buffer: buffers.finalPatches4 },
        },
        {
          binding: 3,
          resource: { buffer: buffers.finalPatches8 },
        },
        {
          binding: 4,
          resource: { buffer: buffers.finalPatches16 },
        },
        {
          binding: 5,
          resource: { buffer: buffers.finalPatches32 },
        },
      ],
    });
    const pingPongPatchesBindGroup = [
      device.createBindGroup({
        layout: layout2,
        entries: [
          {
            binding: 0,
            resource: { buffer: buffers.indirectDispatch1 },
          },
          {
            binding: 1,
            resource: { buffer: buffers.patches0 },
          },
          {
            binding: 2,
            resource: { buffer: buffers.patches1 },
          },
          {
            binding: 3,
            resource: { buffer: buffers.forceRender },
          },
        ],
      }),
      device.createBindGroup({
        layout: layout2,
        entries: [
          {
            binding: 0,
            resource: { buffer: buffers.indirectDispatch0 },
          },
          {
            binding: 1,
            resource: { buffer: buffers.patches1 },
          },
          {
            binding: 2,
            resource: { buffer: buffers.patches0 },
          },
          {
            binding: 3,
            resource: { buffer: buffers.forceRender },
          },
        ],
      }),
    ];

    const doubleNumberOfRounds = 2;
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
      computePassPing.setPipeline(pipeline);
      computePassPing.setBindGroup(0, userInputBindGroup);
      computePassPing.setBindGroup(1, toRenderPatchesBindGroup);
      computePassPing.setBindGroup(2, pingPongPatchesBindGroup[0]);
      computePassPing.dispatchWorkgroupsIndirect(buffers.indirectDispatch0, 0);
      computePassPing.end();

      // debugPrintBuffer(buffers.patches1, commandEncoder);

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

      let computePassPong = commandEncoder.beginComputePass({ label: "Pong" });
      computePassPong.setPipeline(pipeline);
      computePassPong.setBindGroup(0, userInputBindGroup);
      computePassPong.setBindGroup(1, toRenderPatchesBindGroup);
      computePassPong.setBindGroup(2, pingPongPatchesBindGroup[1]);
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
