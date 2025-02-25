import {
    makeFilePath,
    type FilePath,
    type ReactiveFilesystem,
  } from "@/filesystem/reactive-files";
import LodStageWgsl from "../lod_stage.wgsl?raw";
import PrepVerticesStageWgsl from "../prep_vertices_stage.wgsl?raw";
import OutputVerticesWgsl from "../vertices_stage.wgsl?raw";
import { ref } from "vue";
import { createHelpers } from "../webgpu-helpers";
import type { LodStageBuffers } from "@/webgpu-hook";

// Unchanging props! No need to watch them.

const compiledShaders = ref<
  Map<
    FilePath,
    {
      lodStage: {
        shader: GPUShaderModule;
        pipeline: GPUComputePipeline;
      };
      verticesStage: {
        shader: GPUShaderModule;
        pipeline: GPUComputePipeline;
      };
    }
  >
>(new Map());

function concatArrayBuffers(    props: any,views: ArrayBufferView[]): Uint8Array {
    let length = 0;
    for (const v of views) length += v.byteLength;
  
    let buf = new Uint8Array(length);
    let offset = 0;
    for (const v of views) {
      const uint8view = new Uint8Array(v.buffer, v.byteOffset, v.byteLength);
      buf.set(uint8view, offset);
      offset += uint8view.byteLength;
    }
    const { concatArrayBuffers, createBufferWith } = createHelpers(
      props.gpuDevice
    );
  
    return buf;
  }
  
  function createBufferWith(
    props: any,
    contents: Uint8Array,
    usage: GPUBufferUsageFlags,
    size?: number,
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


// Main function
export async function mainExport(triggerDownload: any, exportMeshFromPatches: any, props: any) {
    const _adapter = await navigator.gpu.requestAdapter(); // Wait for Rust backend to be in business
    const device = props.gpuDevice;
  
    const sceneUniformsLayout = device.createBindGroupLayout({
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
  
    const prepVerticesBinding0 = device.createBindGroupLayout({
      entries: [
        {
          binding: 0,
          visibility: GPUShaderStage.COMPUTE,
          buffer: {
            type: "read-only-storage",
          },
        },
        {
          binding: 1,
          visibility: GPUShaderStage.COMPUTE,
          buffer: {
            type: "storage",
          },
        },
      ],
    });
  
    const vertexOutputLayout = device.createBindGroupLayout({
      entries: [
        //{
        //  binding: 0,
        //  visibility: GPUShaderStage.COMPUTE,
        //  buffer: {
        //    type: "uniform",
        //  },
        //},
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
      ],
    });
  
    const pipelineLayout = device.createPipelineLayout({
      bindGroupLayouts: [sceneUniformsLayout, layout1, layout2],
    });
  
    const prepVerticesStageLayout = device.createPipelineLayout({
      bindGroupLayouts: [prepVerticesBinding0],
    });
    const outputVerticesLayout = device.createPipelineLayout({
      bindGroupLayouts: [sceneUniformsLayout, vertexOutputLayout],
    });
  
    function makeShaderFromCodeAndPipeline(
      code: string,
      shaderPipelineLayout: GPUPipelineLayout
    ): {
      pipeline: GPUComputePipeline;
      shader: GPUShaderModule;
    } {
      const shader = props.gpuDevice.createShaderModule({
        code,
      });
      shader.getCompilationInfo().then((info : any) => {
        console.log("Shader compilation resulted in ", info);
      });
      const pipeline = props.gpuDevice.createComputePipeline({
        layout: shaderPipelineLayout,
        compute: {
          module: shader,
        },
      });
      return { shader, pipeline };
    }
  
    async function getShaderCode(
      fs: ReactiveFilesystem,
      key: FilePath
    ): Promise<{
      lodStage: string;
      verticesStage: string;
    }> {
      // Race condition free, because the fs resolves read requests in order
      const code = await fs.readTextFile(key);
  
      const result = {
        lodStage: LodStageWgsl,
        verticesStage: OutputVerticesWgsl,
      };
      if (code !== null) {
        const replaceWithCode = (v: string) =>
          v.replace(
            /\/\/\/\/ START sampleObject([^]+?)\/\/\/\/ END sampleObject/,
            code
          );
        result.lodStage = replaceWithCode(result.lodStage);
        result.verticesStage = replaceWithCode(result.verticesStage);
        ///console.log("LOD STAGE: " + result.lodStage);
        //console.log("VERTICES STAGE: " + result.verticesStage);
      }
      return result;
    }
  
    let prepVerticesStageShaderAndPipeline = makeShaderFromCodeAndPipeline(
      PrepVerticesStageWgsl,
      prepVerticesStageLayout
    );
  
    let oneVertexEntry = 32;
    let startVertexOffset = 8;
    let padding = 8;
    const vertOutputBufferSize =
      startVertexOffset + padding + oneVertexEntry * 200_000;
    let vertOutputBuffer = device.createBuffer({
      label: "VertOutputBuffer",
      size: vertOutputBufferSize,
      usage:
        GPUBufferUsage.COPY_SRC |
        GPUBufferUsage.COPY_DST |
        GPUBufferUsage.STORAGE,
    });
    let vertReadableBuffer = device.createBuffer({
      label: "VertReadableBuffer",
      size: vertOutputBufferSize,
      usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
    });
  
    props.fs.watchFromStart((change : any) => {
      if (!change.key.endsWith(".wgsl")) return;
      if (change.type === "insert" || change.type === "update") {
        getShaderCode(props.fs, change.key).then(
          ({ lodStage, verticesStage }) => {
            compiledShaders.value.set(change.key, {
              lodStage: makeShaderFromCodeAndPipeline(lodStage, pipelineLayout),
              verticesStage: makeShaderFromCodeAndPipeline(
                verticesStage,
                outputVerticesLayout
              ),
            });
          }
        );
      } else {
        compiledShaders.value.delete(change.key);
      }
    });
  
    const patchesBufferDebug = createBufferWith(props,
      // Fill however many slots you want to debug print
      concatArrayBuffers(props,[new Uint32Array([0, 0, 0, 0, 0, 0, 0, 0])]),
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
  
    const MAX_PATCHES = 100_000;
    const patchesBufferReset = createBufferWith(props,
      concatArrayBuffers(props,[new Uint32Array([0, MAX_PATCHES])]),
      GPUBufferUsage.COPY_SRC
    );
  
    const vertOutputBufferReset = createBufferWith(props,
      concatArrayBuffers(props,[new Uint32Array([0, vertOutputBufferSize])]),
      GPUBufferUsage.COPY_SRC
    );
  
    const indirectComputeBufferReset = createBufferWith(props,
      concatArrayBuffers(props,[new Uint32Array([0, 1, 1])]),
      GPUBufferUsage.COPY_SRC
    );
  
    const forceRenderFalse = createBufferWith(props,
      concatArrayBuffers(props,[new Uint32Array([0])]),
      GPUBufferUsage.COPY_SRC
    );
  
    const forceRenderTrue = createBufferWith(props,
      concatArrayBuffers(props,[new Uint32Array([1])]),
      GPUBufferUsage.COPY_SRC
    );
  
    const dispatchVerticesStage = createBufferWith(props,
      concatArrayBuffers(props,[new Uint32Array([1, 1, 1])]),
      GPUBufferUsage.COPY_SRC | GPUBufferUsage.STORAGE | GPUBufferUsage.INDIRECT
    );
  
    function lodStageCallback(
      shaderPath: string,
      buffers: LodStageBuffers,
      commandEncoder: GPUCommandEncoder
    ) {
      const compiledShader = compiledShaders.value.get(makeFilePath(shaderPath));
      if (!compiledShader) {
        console.error("Shader not found: ", shaderPath);
        return;
      }
      const pipeline = compiledShader.lodStage.pipeline;
  
      // We inefficiently recreate the bind groups every time.
      const userInputBindGroup = device.createBindGroup({
        layout: sceneUniformsLayout,
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
  
      let bindGroupVertPrep = device.createBindGroup({
        layout: prepVerticesBinding0,
        entries: [
          {
            binding: 0,
            resource: { buffer: buffers.finalPatches2 },
          },
          {
            binding: 1,
            resource: { buffer: dispatchVerticesStage },
          },
        ],
      });
  
      // @group(1) @binding(0) var<uniform> model: Model;
      // @group(1) @binding(1) var<storage, read> render_buffer: RenderBufferRead;
      // @group(1) @binding(2) var<storage, read_write> output_buffer: OutputBuffer;
  
      let outputVerticesBindGroup = device.createBindGroup({
        layout: vertexOutputLayout,
        entries: [
          {
            binding: 1,
            resource: { buffer: buffers.finalPatches2 },
          },
          {
            binding: 2,
            resource: { buffer: vertOutputBuffer },
          },
        ],
      });
  
      const doubleNumberOfRounds = 3;
      // loop entire process, duplicate entire commandEncoder procedure "doubleNumberOfRounds" times to get more subdivision levels
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
  
      // Prepare vertices output
      let computePassPrep = commandEncoder.beginComputePass({
        label: "prepareVertexOutput",
      });
      computePassPrep.setPipeline(prepVerticesStageShaderAndPipeline.pipeline);
      computePassPrep.setBindGroup(0, bindGroupVertPrep);
      computePassPrep.dispatchWorkgroups(1);
      computePassPrep.end();
  
      commandEncoder.copyBufferToBuffer(
        vertOutputBufferReset,
        0,
        vertOutputBuffer,
        0,
        vertOutputBufferReset.size
      );
  
      // Run vertex output shader
      let computePassVertices = commandEncoder.beginComputePass({
        label: "vertexOutputStage",
      });
      computePassVertices.setPipeline(compiledShader.verticesStage.pipeline);
      computePassVertices.setBindGroup(0, userInputBindGroup);
      computePassVertices.setBindGroup(1, outputVerticesBindGroup);
      computePassVertices.dispatchWorkgroupsIndirect(dispatchVerticesStage, 0);
      computePassVertices.end();
  
      if (triggerDownload.value) {
        triggerDownload.value = false;
        commandEncoder.copyBufferToBuffer(
          vertOutputBuffer,
          0,
          vertReadableBuffer,
          0,
          vertOutputBuffer.size
        );
        setTimeout(() => {
          vertReadableBuffer
            .mapAsync(GPUMapMode.READ, 0, vertReadableBuffer.size)
            .then(() => {
              const arrayBuffer = vertReadableBuffer
                .getMappedRange(0, vertReadableBuffer.size)
                .slice(0);
              const vertexStream = new Float32Array(arrayBuffer);
              vertReadableBuffer.unmap();
              exportMeshFromPatches(vertexStream);
            });
        }, 10);
      }
    }
  
    props.engine.setLodStage(lodStageCallback);
  }