import {
  makeFilePath,
  type FilePath,
  type ReactiveFilesystem,
} from "@/filesystem/reactive-files";
import LodStageWgsl from "../lod_stage.wgsl?raw";
import PrepVerticesStageWgsl from "../prep_vertices_stage.wgsl?raw";
import OutputVerticesWgsl from "../vertices_stage.wgsl?raw";
import FilterInstancesWgsl from "../filter-instance.wgsl?raw";
import IndirectDispatchRebalanceSource from "../indirect_dispatch_rebalance.wgsl?raw";
import { ref, type Ref } from "vue";
import type { LodStageBuffers } from "@/webgpu-hook";
import {
  simpleB2BSameSize,
  createBindGroupLayout,
  createSimpleBindGroup,
  createBufferWith,
  concatArrayBuffers,
  createShaderWithPipeline,
} from "./GPUUtils";
// Unchanging props! No need to watch them.
// Base structure Taken from https://webgpu.github.io/webgpu-samples/?sample=rotatingCube#main.ts

const shaderPipelinesMap = ref<
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

function parseOutDebugInfo(buf: ArrayBuffer)
{
  const floats = new Float32Array(buf);
  const ints = new Int32Array(buf);
  let index = ints[0];
  let count = ints[1];
  debugger;
  let i = 4;
  while(i < floats.length)
  {
    var minX = floats[i++];
    var minY = floats[i++];
    var maxX = floats[i++];
    var maxY = floats[i++];
    var instance = ints[i++];
    i++;
    var curvature = floats[i++];
    var planarity = floats[i++];
    var size = floats[i++];
    i+=3;
    var vec1x = floats[i++];
    var vec1y = floats[i++];
    var vec1z = floats[i++];
    var vec1w = floats[i++];
    
    var vec2x = floats[i++];
    var vec2y = floats[i++];
    var vec2z = floats[i++];
    var vec2w = floats[i++];

    var structure = {
      min: [minX, minY],
      max: [maxX, maxY],
      instance: instance,
      criteria: {
        planarity: planarity,
        size: size,
        curvature: curvature
      },
      v1: {
        x: vec1x,
        y: vec1y,
        z: vec1z,
        z2: vec1w,
      },
      v2: {
        x: vec2x,
        y: vec2y,
        z: vec2z,
        z2: vec2w
      }
    }
    console.log("DEBUG STRUCT: ", structure);
    if(size == 0)
    {
      return;
    }
  }
}

// Main function
export async function mainExport(
  triggerDownload: any,
  exportMeshFromPatches: any,
  props: any,
  lodExportParametersRefs: any,
  onFrame: any
): Promise<any> {
  const _adapter = await navigator.gpu.requestAdapter(); // Wait for Rust backend to be in business
  const device = props.gpuDevice;
  const sceneUniformsLayout = createBindGroupLayout(
    GPUShaderStage.COMPUTE,
    ["uniform", "uniform", "uniform"],
    device,
    "scene uniforms"
  );
  const renderBuffersBindGroupLayout = createBindGroupLayout(
    GPUShaderStage.COMPUTE,
    ["uniform", "storage", "uniform"],
    device,
    "renderBuffersBindGroupLayout"
  );
  const layout2 = createBindGroupLayout(
    GPUShaderStage.COMPUTE,
    ["storage", "read-only-storage", "storage", "uniform", "storage"],
    device,
    "layout2"
  );

  const prepVerticesBinding0 = createBindGroupLayout(
    GPUShaderStage.COMPUTE,
    ["read-only-storage", "storage"],
    device,
    "prep vertices"
  );

  const vertexOutputLayout = createBindGroupLayout(
    GPUShaderStage.COMPUTE,
    ["read-only-storage", "storage"],
    device,
    "patch output"
  );

  const instanceSelectionLayout = createBindGroupLayout(
    GPUShaderStage.COMPUTE,
    ["uniform"],
    device,
    "target instance id"
  );

  const indirectDispatchTransformerBindGroupLayout = createBindGroupLayout(
    GPUShaderStage.COMPUTE,
    ["storage"],
    device,
    "indirect dispatch transformer bind group layout"
  );

  const pipelineLayout = device.createPipelineLayout({
    bindGroupLayouts: [
      sceneUniformsLayout,
      renderBuffersBindGroupLayout,
      layout2,
    ],
  });

  const prepVerticesStageLayout = device.createPipelineLayout({
    bindGroupLayouts: [prepVerticesBinding0],
  });
  const outputVerticesLayout = device.createPipelineLayout({
    bindGroupLayouts: [
      sceneUniformsLayout,
      vertexOutputLayout,
      instanceSelectionLayout,
    ],
  });

  const filterInstancesStageLayout = device.createPipelineLayout({
    bindGroupLayouts: [layout2, instanceSelectionLayout],
  });

  const indirectDispatchTransformerLayout: GPUPipelineLayout =
    device.createPipelineLayout({
      bindGroupLayouts: [indirectDispatchTransformerBindGroupLayout],
    });

  let v = createShaderWithPipeline(
    IndirectDispatchRebalanceSource,
    indirectDispatchTransformerLayout,
    device
  );
  const indirectDispatchRebalancePipeline = v.pipeline;

  async function fetchAndInjectShaderCode(
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
    }
    return result;
  }

  let prepVerticesShaderWithPipeline = createShaderWithPipeline(
    PrepVerticesStageWgsl,
    prepVerticesStageLayout,
    device
  );

  let filterInstanceIdPipeline = createShaderWithPipeline(
    FilterInstancesWgsl,
    filterInstancesStageLayout,
    device
  );

  const MAX_PATCHES = 2_000_000;
  let oneVertexEntry = 32;
  let startVertexOffset = 8;
  let padding = 8;
  const vertOutputBufferSize: number =
    startVertexOffset + padding + oneVertexEntry * MAX_PATCHES;

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
      fetchAndInjectShaderCode(props.fs, change.key).then(
        ({ lodStage, verticesStage, code }) => {
          shaderPipelinesMap.value.set(change.key, {
            lodStage: createShaderWithPipeline(
              lodStage,
              pipelineLayout,
              device
            ),
            verticesStage: createShaderWithPipeline(
              verticesStage,
              outputVerticesLayout,
              device
            ),
          });
        }
      );
    } else {
      shaderPipelinesMap.value.delete(change.key);
    }
  });

  const patchesBufferReset = createBufferWith(
    props,
    concatArrayBuffers(props, [new Uint32Array([0, MAX_PATCHES])]),
    GPUBufferUsage.COPY_SRC
  );

  let reset = new Uint32Array(vertOutputBuffer.size / 32);
  reset[0] = 0;
  reset[1] = vertOutputBufferSize / 32;
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

  const forceRenderFalseBuffer = createBufferWith(
    props,
    concatArrayBuffers(props, [new Uint32Array([0])]),
    GPUBufferUsage.COPY_SRC
  );

  const forceRenderTrueBuffer = createBufferWith(
    props,
    concatArrayBuffers(props, [new Uint32Array([1])]),
    GPUBufferUsage.COPY_SRC
  );

  const dispatchVerticesStage = createBufferWith(
    props,
    concatArrayBuffers(props, [new Uint32Array([1, 1, 1])]),
    GPUBufferUsage.COPY_SRC |
      GPUBufferUsage.COPY_DST |
      GPUBufferUsage.STORAGE |
      GPUBufferUsage.INDIRECT
  );

  const dispatchVerticesStageResetBuffer = createBufferWith(
    props,
    concatArrayBuffers(props, [new Uint32Array([1, 1, 1])]),
    GPUBufferUsage.COPY_SRC | GPUBufferUsage.STORAGE | GPUBufferUsage.INDIRECT
  );

  let lodStageParameters = new Float32Array([
    lodExportParametersRefs.minSize.value,
    lodExportParametersRefs.maxCurvature,
    lodExportParametersRefs.acceptablePlanarity.value,
    0,0,0
  ]);
  const lodStageParametersBuffer = createBufferWith(
    props,
    concatArrayBuffers(props, [lodStageParameters]),
    GPUBufferUsage.COPY_DST | GPUBufferUsage.UNIFORM
  );
  const debugBufferSize = 64 * 1024; // 64KB (adjust as needed)

  const debugInfoBuffer = device.createBuffer({
    label: "Debug info buffer",
    size: debugBufferSize,
    usage:
      GPUBufferUsage.COPY_DST |
      GPUBufferUsage.COPY_SRC |
      GPUBufferUsage.STORAGE,
  });

  const debugOutBuffer = device.createBuffer({
    label: "Debug out buffer",
    size: debugBufferSize,
    usage:
      GPUBufferUsage.COPY_DST |
      GPUBufferUsage.MAP_READ
  });

  let numberBuffersArray: GPUBuffer[] = [];
  for (let i = 0; i < 100; i++) {
    numberBuffersArray.push(
      createBufferWith(
        props,
        concatArrayBuffers(props, [new Uint32Array([i])]),
        GPUBufferUsage.COPY_SRC | GPUBufferUsage.UNIFORM
      )
    );
  }

  let targetInstanceIdBuffer: GPUBuffer = createBufferWith(
    props,
    concatArrayBuffers(props, [new Uint32Array([0])]),
    GPUBufferUsage.COPY_DST | GPUBufferUsage.UNIFORM
  );

  let sceneUniformsBindGroup: any = null;

  function lodStageCallback(
    shaderPath: string,
    buffers: LodStageBuffers,
    commandEncoder: GPUCommandEncoder
  ) {
    const doubleNumberOfRounds = lodExportParametersRefs.subdivisionSteps.value;
    if (!buffers.computePatchesInput) {
      console.log("Missing buffer. Buffers: " + buffers);
    }
    let uuid = buffers.computePatchesInput.label.split(" ")[0];

    let name = uuid;
    var re = /\.wgsl$/;
    name = name.replace(re, "");
    let models: Ref<any[]> = lodExportParametersRefs.models;
    let registered = false;
    let instanceCount = 0;
    models.value.forEach((model) => {
      if (model.uuid == uuid) {
        name = model.name;
        instanceCount = model.instanceCount;
      }
    });

    let downloadTarget = lodExportParametersRefs.downloadTarget.value;
    let downloadAll = downloadTarget == "";
    let isRightDownloadTarget = downloadAll || downloadTarget == uuid;
    let toDownload = lodExportParametersRefs.toDownload.value;
    let toDownloadModel = null;
    let toDownloadIndex = 0;
    for (let i = 0; i < toDownload.length; i++) {
      if (
        toDownload[i].name == uuid &&
        toDownload[i].currentInstance <= instanceCount
      ) {
        toDownloadModel = toDownload[i];
        toDownloadIndex = i;
        break;
      }
    }
    if (
      toDownload.length != 0 &&
      (!isRightDownloadTarget ||
        toDownloadModel == null ||
        triggerDownload.value == false)
    ) {
      return;
    }
    let instanceToDownload =
      toDownloadModel == null ? 0 : toDownloadModel.currentInstance;

    if (!isRightDownloadTarget) return;
    if (toDownload.length != 0) {
      if (instanceToDownload >= instanceCount) {
        toDownload.splice(toDownloadIndex, 1);
        console.log("Removed " + toDownload + " because it was done");
        return;
      }
    }
    const compiledShader = shaderPipelinesMap.value.get(
      makeFilePath(shaderPath)
    );
    if (!compiledShader) {
      // The shader probably didn't load yet
      return;
    }
    const pipeline = compiledShader.lodStage.pipeline;

    // We inefficiently recreate the bind groups every time.
    if (sceneUniformsBindGroup == null)
      sceneUniformsBindGroup = createSimpleBindGroup(
        [buffers.time, buffers.screen, buffers.mouse],
        sceneUniformsLayout,
        device,
        uuid
      );

    const toRenderPatchesBindGroup = createSimpleBindGroup(
      [
        buffers.computePatchesInput,
        buffers.finalPatches2,
        // buffers.finalPatches4,
        // buffers.finalPatches8,
        // buffers.finalPatches16,
        // buffers.finalPatches32,
        lodStageParametersBuffer,
      ],
      renderBuffersBindGroupLayout,
      device,
      uuid
    );

    toRenderPatchesBindGroup.label = "to render patches bind group";

    const pingPongPatchesBindGroup = [
      createSimpleBindGroup(
        [
          buffers.indirectDispatch1,
          buffers.patches0,
          buffers.patches1,
          buffers.forceRender,
          debugInfoBuffer,
        ],
        layout2,
        device,
        "Ping bind group"
      ),
      createSimpleBindGroup(
        [
          buffers.indirectDispatch0,
          buffers.patches1,
          buffers.patches0,
          buffers.forceRender,
          debugInfoBuffer,
        ],
        layout2,
        device,
        "Pong bind group"
      ),
    ];

    const indirectRebalanceBindGroups = [
      createSimpleBindGroup(
        [buffers.indirectDispatch1],
        indirectDispatchTransformerBindGroupLayout,
        device,
        null
      ),
      createSimpleBindGroup(
        [buffers.indirectDispatch0],
        indirectDispatchTransformerBindGroupLayout,
        device,
        null
      ),
    ];

    lodStageParameters = new Float32Array([
      lodExportParametersRefs.minSize.value,
      lodExportParametersRefs.maxCurvature.value,
      lodExportParametersRefs.acceptablePlanarity.value * 0.05 + 0.95,
      0,
      0,
      0
    ]);

    let boolaccess = new Uint32Array(lodStageParameters.buffer);
    boolaccess[3] = lodExportParametersRefs.ignoreMinSize.value ? 1 : 0;
    boolaccess[4] = lodExportParametersRefs.ignoreCurvature.value ? 1 : 0;
    boolaccess[5] = lodExportParametersRefs.ignorePlanarity.value ? 1 : 0;

    device.queue.writeBuffer(
      lodStageParametersBuffer,
      0,
      lodStageParameters,
      0,
      lodStageParameters.length
    );

    simpleB2BSameSize(
      numberBuffersArray[instanceToDownload],
      targetInstanceIdBuffer,
      commandEncoder
    );

    let instanceSelectionBindGroup = createSimpleBindGroup(
      [targetInstanceIdBuffer],
      instanceSelectionLayout,
      device,
      "instance selection bg"
    );

    if (toDownload.length != 0) {
      console.log("Filtering!");
      let computePassFilterInstance = commandEncoder.beginComputePass();
      computePassFilterInstance.label = "filter instance pass";
      computePassFilterInstance.setPipeline(filterInstanceIdPipeline.pipeline);
      computePassFilterInstance.setBindGroup(0, pingPongPatchesBindGroup[0]);
      computePassFilterInstance.setBindGroup(1, instanceSelectionBindGroup);
      computePassFilterInstance.dispatchWorkgroupsIndirect(
        buffers.indirectDispatch0,
        0
      );
      computePassFilterInstance.end();
      console.log("Filtering done");
      simpleB2BSameSize(buffers.patches1, buffers.patches0, commandEncoder);
    }
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
      computePassPing.setBindGroup(0, sceneUniformsBindGroup);
      computePassPing.setBindGroup(1, toRenderPatchesBindGroup);
      computePassPing.setBindGroup(2, pingPongPatchesBindGroup[0]);
      computePassPing.dispatchWorkgroupsIndirect(buffers.indirectDispatch0, 0);
      computePassPing.end();

      // Pong
      if (isLastRound) {
        simpleB2BSameSize(
          forceRenderTrueBuffer,
          buffers.forceRender,
          commandEncoder
        );
      }

      simpleB2BSameSize(patchesBufferReset, buffers.patches0, commandEncoder);
      simpleB2BSameSize(
        indirectComputeBufferReset,
        buffers.indirectDispatch0,
        commandEncoder
      );

      let rebalancePass = commandEncoder.beginComputePass();
      rebalancePass.label = "rebalance 1";
      rebalancePass.setPipeline(indirectDispatchRebalancePipeline);
      rebalancePass.setBindGroup(0, indirectRebalanceBindGroups[0]);
      rebalancePass.dispatchWorkgroups(1, 1, 1);

      rebalancePass.end();

      let computePassPong = commandEncoder.beginComputePass({ label: "Pong" });
      computePassPong.setPipeline(pipeline);
      computePassPong.setBindGroup(0, sceneUniformsBindGroup);
      computePassPong.setBindGroup(1, toRenderPatchesBindGroup);
      computePassPong.setBindGroup(2, pingPongPatchesBindGroup[1]);
      computePassPong.dispatchWorkgroupsIndirect(buffers.indirectDispatch1, 0);
      computePassPong.end();

      if (isLastRound) {
        simpleB2BSameSize(
          forceRenderFalseBuffer,
          buffers.forceRender,
          commandEncoder
        );
      }

      let rebalancePass2 = commandEncoder.beginComputePass();
      rebalancePass.label = "rebalance pass";
      rebalancePass2.setPipeline(indirectDispatchRebalancePipeline);
      rebalancePass2.setBindGroup(0, indirectRebalanceBindGroups[1]);
      rebalancePass2.dispatchWorkgroups(1, 1, 1);
      rebalancePass2.end();
    }

    if (!triggerDownload.value) {
      return;
    }
    if (isRightDownloadTarget && toDownloadModel != null) {
      triggerDownload.value = false;
      // Prepare vertices output

      let bindGroupVertPrep = createSimpleBindGroup(
        [buffers.finalPatches2, dispatchVerticesStage],
        prepVerticesBinding0,
        device,
        uuid
      );
      bindGroupVertPrep.label = "bind group vert prep";

      let outputVerticesBindGroup = createSimpleBindGroup(
        [buffers.finalPatches2, vertOutputBuffer],
        vertexOutputLayout,
        device,
        uuid
      );
      outputVerticesBindGroup.label = "output vertices bind group";
      let computePassPrep = commandEncoder.beginComputePass({
        label: "prepareVertexOutput",
      });
      computePassPrep.setPipeline(prepVerticesShaderWithPipeline.pipeline);
      computePassPrep.setBindGroup(0, bindGroupVertPrep);
      computePassPrep.dispatchWorkgroups(1);
      computePassPrep.end();

      simpleB2BSameSize(
        vertOutputBufferReset,
        vertOutputBuffer,
        commandEncoder
      );

      // Run vertex output shader
      let computePassVertices = commandEncoder.beginComputePass({
        label: "vertexOutputStage",
      });

      computePassVertices.setPipeline(compiledShader.verticesStage.pipeline);
      computePassVertices.setBindGroup(0, sceneUniformsBindGroup);
      computePassVertices.setBindGroup(1, outputVerticesBindGroup);
      computePassVertices.setBindGroup(2, instanceSelectionBindGroup);
      computePassVertices.dispatchWorkgroupsIndirect(dispatchVerticesStage, 0);
      computePassVertices.end();
      lodExportParametersRefs.currentItem.value = name + instanceToDownload;
      simpleB2BSameSize(vertOutputBuffer, vertReadableBuffer, commandEncoder);
      simpleB2BSameSize(
        vertOutputBufferReset,
        vertOutputBuffer,
        commandEncoder
      );

      simpleB2BSameSize(
        dispatchVerticesStageResetBuffer,
        dispatchVerticesStage,
        commandEncoder
      );

      simpleB2BSameSize(debugInfoBuffer,debugOutBuffer,commandEncoder);
      setTimeout(() => {
        debugOutBuffer.mapAsync(
          GPUMapMode.READ, 0, debugOutBuffer.size
        ).then(
          () => 
          {
            var data: any = debugOutBuffer.getMappedRange().slice();
            debugOutBuffer.unmap();
            parseOutDebugInfo(data);
          }
        );
        vertReadableBuffer
          .mapAsync(GPUMapMode.READ, 0, vertReadableBuffer.size)
          .then(() => {
            toDownloadModel.currentInstance++;
            console.log(
              "Memory mapped the buffer for " +
                name +
                " instance " +
                instanceToDownload
            );
            if (instanceToDownload >= instanceCount - 1) {
              toDownload.splice(toDownloadIndex, 1);
              //console.log("Removed " + toDownload + " because it was done");
            }
            const arrayBuffer = vertReadableBuffer
              .getMappedRange(0, vertReadableBuffer.size)
              .slice();
            const vertexStream = new Float32Array(arrayBuffer);
            console.log(vertexStream.slice(0, 1000));
            vertReadableBuffer.unmap();
            if (downloadAll) {
              triggerDownload.value = true;
            }

            exportMeshFromPatches(
              vertexStream,
              lodExportParametersRefs.includeUVs.value,
              uuid,
              downloadAll
            );
          });
      }, 1);
    }
  }
  await props.engine.setLodStage(lodStageCallback);
  return lodStageCallback;
}
