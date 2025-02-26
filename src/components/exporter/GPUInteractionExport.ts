import {
  makeFilePath,
  type FilePath,
  type ReactiveFilesystem,
} from "@/filesystem/reactive-files";
import LodStageWgsl from "../lod_stage.wgsl?raw";
import PrepVerticesStageWgsl from "../prep_vertices_stage.wgsl?raw";
import OutputVerticesWgsl from "../vertices_stage.wgsl?raw";
import { ref, type Ref } from "vue";
import type { LodStageBuffers } from "@/webgpu-hook";
import {
  simpleB2BSameSize,
  simpleBindGroupLayout,
  createSimpleBindGroup,
  createBufferWith,
  concatArrayBuffers,
  makeShaderFromCodeAndPipeline,
} from "./GPUUtils";
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

// Main function
export async function mainExport(
  triggerDownload: any,
  exportMeshFromPatches: any,
  props: any,
  lodExportParametersRefs: any,
  onFrame: any
) {
  const _adapter = await navigator.gpu.requestAdapter(); // Wait for Rust backend to be in business
  const device = props.gpuDevice;
  const sceneUniformsLayout = simpleBindGroupLayout(
    GPUShaderStage.COMPUTE,
    ["uniform", "uniform", "uniform"],
    device,
    "scene uniforms"
  );
  const layout1 = simpleBindGroupLayout(
    GPUShaderStage.COMPUTE,
    [
      "uniform",
      "storage",
      "storage",
      "storage",
      "storage",
      "storage",
      "uniform",
    ],
    device,
    "layout1"
  );
  const layout2 = simpleBindGroupLayout(
    GPUShaderStage.COMPUTE,
    ["storage", "read-only-storage", "storage", "uniform"],
    device,
    "layout2"
  );

  const prepVerticesBinding0 = simpleBindGroupLayout(
    GPUShaderStage.COMPUTE,
    ["read-only-storage", "storage"],
    device,
    "prep vertices"
  );

  const vertexOutputLayout = simpleBindGroupLayout(
    GPUShaderStage.COMPUTE,
    ["read-only-storage", "storage"],
    device,
    "patch output"
  );

  const pipelineLayout = device.createPipelineLayout({
    bindGroupLayouts: [sceneUniformsLayout, layout1, layout2],
  });

  const prepVerticesStageLayout = device.createPipelineLayout({
    bindGroupLayouts: [prepVerticesBinding0],
  });
  const outputVerticesLayout = device.createPipelineLayout({
    bindGroupLayouts: [sceneUniformsLayout, vertexOutputLayout],
  });

  async function getShaderCode(
    fs: ReactiveFilesystem,
    key: FilePath
  ): Promise<{
    lodStage: string;
    verticesStage: string;
    code: string | null;
  }> {
    // Race condition free, because the fs resolves read requests in order
    const code = await fs.readTextFile(key);

    const result = {
      lodStage: LodStageWgsl,
      verticesStage: OutputVerticesWgsl,
      code: code,
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
    prepVerticesStageLayout,
    device
  );

  let oneVertexEntry = 32;
  let startVertexOffset = 8;
  let padding = 8;
  const vertOutputBufferSize =
    startVertexOffset + padding + oneVertexEntry * 500_000;
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

  props.fs.watchFromStart((change: any) => {
    if (!change.key.endsWith(".wgsl")) return;
    if (change.type === "insert" || change.type === "update") {
      getShaderCode(props.fs, change.key).then(
        ({ lodStage, verticesStage, code }) => {
          compiledShaders.value.set(change.key, {
            lodStage: makeShaderFromCodeAndPipeline(
              lodStage,
              pipelineLayout,
              device
            ),
            verticesStage: makeShaderFromCodeAndPipeline(
              verticesStage,
              outputVerticesLayout,
              device
            ),
          });
        }
      );
    } else {
      compiledShaders.value.delete(change.key);
    }
  });

  const patchesBufferDebug = createBufferWith(
    props,
    // Fill however many slots you want to debug print
    concatArrayBuffers(props, [new Uint32Array([0, 0, 0, 0, 0, 0, 0, 0])]),
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

  const MAX_PATCHES = 500_000;
  const patchesBufferReset = createBufferWith(
    props,
    concatArrayBuffers(props, [new Uint32Array([0, MAX_PATCHES])]),
    GPUBufferUsage.COPY_SRC
  );

  let reset = new Uint32Array(vertOutputBufferSize / 32);
  reset[0] = 0;
  reset[1] = vertOutputBufferSize;
  const vertOutputBufferReset = createBufferWith(
    props,
    concatArrayBuffers(props, [reset]),
    GPUBufferUsage.COPY_SRC
  );

  const indirectComputeBufferReset = createBufferWith(
    props,
    concatArrayBuffers(props, [new Uint32Array([0, 1, 1])]),
    GPUBufferUsage.COPY_SRC
  );

  const forceRenderFalse = createBufferWith(
    props,
    concatArrayBuffers(props, [new Uint32Array([0])]),
    GPUBufferUsage.COPY_SRC
  );

  const forceRenderTrue = createBufferWith(
    props,
    concatArrayBuffers(props, [new Uint32Array([1])]),
    GPUBufferUsage.COPY_SRC
  );

  const dispatchVerticesStage = createBufferWith(
    props,
    concatArrayBuffers(props, [new Uint32Array([1, 1, 1])]),
    GPUBufferUsage.COPY_SRC | GPUBufferUsage.STORAGE | GPUBufferUsage.INDIRECT
  );

  let lodStageParameters = new Float32Array([
    lodExportParametersRefs.minSize.value,
    lodExportParametersRefs.maxCurvature,
    lodExportParametersRefs.acceptablePlanarity.value,
  ]);
  const lodStageParametersBuffer = createBufferWith(
    props,
    concatArrayBuffers(props, [lodStageParameters]),
    GPUBufferUsage.COPY_DST | GPUBufferUsage.UNIFORM
  );

  const lodStageParametersStagingBuffer = createBufferWith(
    props,
    concatArrayBuffers(props, [lodStageParameters]),
    GPUBufferUsage.MAP_WRITE | GPUBufferUsage.COPY_SRC
  );

  async function lodStageCallback(
    shaderPath: string,
    buffers: LodStageBuffers,
    commandEncoder: GPUCommandEncoder
  ) {
    let name = shaderPath;
    let uuid = buffers.computePatchesInput.label.split(" ")[0];
    var re = /\.wgsl$/;
    name = name.replace(re, "");
    let models: Ref<any[]> = lodExportParametersRefs.models;
    let registered = false;
    models.value.forEach((model) => {
      if (model.path == shaderPath) {
        registered = true;
      }
    });
    if (!registered) {
      models.value.push({
        path: shaderPath,
        name: name,
        uuid: uuid,
      });
    }
    const compiledShader = compiledShaders.value.get(makeFilePath(shaderPath));
    if (!compiledShader) {
      console.error("Shader not found: ", shaderPath);
      return;
    }
    const pipeline = compiledShader.lodStage.pipeline;

    // We inefficiently recreate the bind groups every time.
    const userInputBindGroup = createSimpleBindGroup(
      [buffers.time, buffers.screen, buffers.mouse],
      sceneUniformsLayout,
      device
    );

    const toRenderPatchesBindGroup = createSimpleBindGroup(
      [
        buffers.computePatchesInput,
        buffers.finalPatches2,
        buffers.finalPatches4,
        buffers.finalPatches8,
        buffers.finalPatches16,
        buffers.finalPatches32,
        lodStageParametersBuffer,
      ],
      layout1,
      device
    );

    const pingPongPatchesBindGroup = [
      createSimpleBindGroup(
        [
          buffers.indirectDispatch1,
          buffers.patches0,
          buffers.patches1,
          buffers.forceRender,
        ],
        layout2,
        device
      ),
      createSimpleBindGroup(
        [
          buffers.indirectDispatch0,
          buffers.patches1,
          buffers.patches0,
          buffers.forceRender,
        ],
        layout2,
        device
      ),
    ];

    let bindGroupVertPrep = createSimpleBindGroup(
      [buffers.finalPatches2, dispatchVerticesStage],
      prepVerticesBinding0,
      device
    );

    let outputVerticesBindGroup = createSimpleBindGroup(
      [buffers.finalPatches2, vertOutputBuffer],
      vertexOutputLayout,
      device
    );

    lodStageParameters = new Float32Array([
      lodExportParametersRefs.minSize.value,
      lodExportParametersRefs.maxCurvature.value,
      lodExportParametersRefs.acceptablePlanarity.value,
    ]);
    device.queue.writeBuffer(
      lodStageParametersBuffer,
      0,
      lodStageParameters,
      0,
      lodStageParameters.length
    );
    const doubleNumberOfRounds = 5;
    // loop entire process, duplicate entire commandEncoder procedure "doubleNumberOfRounds" times to get more subdivision levels
    for (let i = 0; i < doubleNumberOfRounds; i++) {
      const isLastRound = i === doubleNumberOfRounds - 1;
      // Ping
      simpleB2BSameSize(patchesBufferReset, buffers.patches1, commandEncoder);
      simpleB2BSameSize(
        indirectComputeBufferReset,
        buffers.indirectDispatch1,
        commandEncoder
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
        simpleB2BSameSize(forceRenderTrue, buffers.forceRender, commandEncoder);
      }
      simpleB2BSameSize(patchesBufferReset, buffers.patches0, commandEncoder);
      simpleB2BSameSize(
        indirectComputeBufferReset,
        buffers.indirectDispatch0,
        commandEncoder
      );

      let computePassPong = commandEncoder.beginComputePass({ label: "Pong" });
      computePassPong.setPipeline(pipeline);
      computePassPong.setBindGroup(0, userInputBindGroup);
      computePassPong.setBindGroup(1, toRenderPatchesBindGroup);
      computePassPong.setBindGroup(2, pingPongPatchesBindGroup[1]);
      computePassPong.dispatchWorkgroupsIndirect(buffers.indirectDispatch1, 0);
      computePassPong.end();

      if (isLastRound) {
        simpleB2BSameSize(
          forceRenderFalse,
          buffers.forceRender,
          commandEncoder
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

    simpleB2BSameSize(vertOutputBufferReset, vertOutputBuffer, commandEncoder);

    // Run vertex output shader
    let computePassVertices = commandEncoder.beginComputePass({
      label: "vertexOutputStage",
    });
    computePassVertices.setPipeline(compiledShader.verticesStage.pipeline);
    computePassVertices.setBindGroup(0, userInputBindGroup);
    computePassVertices.setBindGroup(1, outputVerticesBindGroup);
    computePassVertices.dispatchWorkgroupsIndirect(dispatchVerticesStage, 0);
    computePassVertices.end();

    //      console.log("SHADER OF THIS OBJECT IS: " + shaderPath);
    //console.log("Hi from " + buffers.computePatchesInput.label);

    let downloadTarget = lodExportParametersRefs.downloadTarget.value;
    let downloadAll = downloadTarget == "";
    let isRightDownloadTarget = downloadAll || downloadTarget == shaderPath;
    if (triggerDownload.value) {
      //        debugger;
    }
    onFrame();
    if (
      triggerDownload.value &&
      isRightDownloadTarget &&
      !lodExportParametersRefs.bufferedNames.value.includes(name)
    ) {
      console.log("Exporting " + name);
      triggerDownload.value = false;
      if (!downloadAll) triggerDownload.value = false;
      simpleB2BSameSize(vertOutputBuffer, vertReadableBuffer, commandEncoder);
      setTimeout(() => {
        vertReadableBuffer
          .mapAsync(GPUMapMode.READ, 0, vertReadableBuffer.size)
          .then(() => {
            console.log("Memory mapped the buffer for " + name);
            const arrayBuffer = vertReadableBuffer
              .getMappedRange(0, vertReadableBuffer.size)
              .slice(0);
            const vertexStream = new Float32Array(arrayBuffer);
            vertReadableBuffer.unmap();
            if (downloadAll) triggerDownload.value = true;
            exportMeshFromPatches(
              vertexStream,
              lodExportParametersRefs.includeUVs.value,
              name,
              downloadAll
            );
          });
      }, 1);
    }
  }
  props.engine.setLodStage(lodStageCallback);
}
