import {
  makeFilePath,
  type FilePath,
  type ReactiveFilesystem,
} from "@/filesystem/reactive-files";
import LodStageWgsl from "../lod_stage.wgsl?raw";
import PrepVerticesStageWgsl from "../prep_vertices_stage.wgsl?raw";
import OutputVerticesWgsl from "../vertices_stage.wgsl?raw";
import IndirectDispatchRebalanceSource from "../indirect_dispatch_rebalance.wgsl?raw";
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
// Base structure Taken from https://webgpu.github.io/webgpu-samples/?sample=rotatingCube#main.ts

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

  const instanceSelectionLayout = simpleBindGroupLayout(
    GPUShaderStage.COMPUTE,
    ["uniform"],
    device,
    "target instance id"
  );

  
  const indirectDispatchTransformerBindGroupLayout = simpleBindGroupLayout(
    GPUShaderStage.COMPUTE,
    ["storage"],
    device,
    "indirect dispatch transformer bind group layout"
  );

  const pipelineLayout = device.createPipelineLayout({
    bindGroupLayouts: [sceneUniformsLayout, layout1, layout2],
  });

  const prepVerticesStageLayout = device.createPipelineLayout({
    bindGroupLayouts: [prepVerticesBinding0],
  });
  const outputVerticesLayout = device.createPipelineLayout({
    bindGroupLayouts: [sceneUniformsLayout, vertexOutputLayout, instanceSelectionLayout],
  });

  const indirectDispatchTransformerLayout: GPUPipelineLayout = device.createPipelineLayout({
    bindGroupLayouts: [indirectDispatchTransformerBindGroupLayout]
  });

  let v = makeShaderFromCodeAndPipeline(IndirectDispatchRebalanceSource,indirectDispatchTransformerLayout, device);
  const indirectDispatchRebalancePipeline = v.pipeline;
  const indirectDispatchRebalanceShader = v.shader;

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
//  alert("Actual size: " + vertOutputBuffer.size)
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

  let numberBuffersArray: GPUBuffer[] = [];
  for(let i = 0; i < 30; i++)
  {
    numberBuffersArray.push(
      createBufferWith(
        props,
        concatArrayBuffers(props, [new Uint32Array([i])]),
        GPUBufferUsage.COPY_SRC | GPUBufferUsage.UNIFORM
      )
    );
  }

  let target_instance_id_buffer: GPUBuffer = 
    createBufferWith(
      props,
      concatArrayBuffers(props, [new Uint32Array([0])]),
      GPUBufferUsage.COPY_DST | GPUBufferUsage.UNIFORM
    );


  let userInputBindGroup: any = null;

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

    const compiledShader = compiledShaders.value.get(makeFilePath(shaderPath));
    if (!compiledShader) {
      // The shader probably didn't load yet
      // console.error("Shader not found: ", shaderPath);
      return;
    }
    const pipeline = compiledShader.lodStage.pipeline;

    // We inefficiently recreate the bind groups every time.
    if(userInputBindGroup == null)
      userInputBindGroup = createSimpleBindGroup(
        [buffers.time, buffers.screen, buffers.mouse],
        sceneUniformsLayout,
        device,
        uuid
      );

//      console.log(buffers);

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
      device,
      uuid
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
        device,
        null
      ),
      createSimpleBindGroup(
        [
          buffers.indirectDispatch0,
          buffers.patches1,
          buffers.patches0,
          buffers.forceRender,
        ],
        layout2,
        device,
        null
      ),
    ];

    const indirectRebalanceBindGroups = [
      createSimpleBindGroup(
        [
          buffers.indirectDispatch1
        ],
        indirectDispatchTransformerBindGroupLayout,
        device,
        null
      ),
      createSimpleBindGroup(
        [
          buffers.indirectDispatch0,
        ],
        indirectDispatchTransformerBindGroupLayout,
        device,
        null
      ),
    ];




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
      
      let rebalancePass = commandEncoder.beginComputePass();
      rebalancePass.setPipeline(indirectDispatchRebalancePipeline);
      rebalancePass.setBindGroup(0,indirectRebalanceBindGroups[0]);
      rebalancePass.dispatchWorkgroups(1,1,1);
      rebalancePass.end();

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
      
      let rebalancePass2 = commandEncoder.beginComputePass();
      rebalancePass2.setPipeline(indirectDispatchRebalancePipeline);
      rebalancePass2.setBindGroup(0,indirectRebalanceBindGroups[1] );
      rebalancePass2.dispatchWorkgroups(1,1,1);
      rebalancePass2.end();
    }

    if (!triggerDownload.value) {
      return;
    }
    let downloadTarget = lodExportParametersRefs.downloadTarget.value;
    let downloadAll = downloadTarget == "";
    let isRightDownloadTarget = downloadAll || downloadTarget == uuid;
    let toDownload = lodExportParametersRefs.toDownload.value;
    let toDownloadModel = null;
    let toDownloadIndex = 0;
    for(let i = 0; i < toDownload.length; i++)
    {
      if(toDownload[i].name == uuid && toDownload[i].currentInstance <= instanceCount)
      {
        toDownloadModel = toDownload[i];
        toDownloadIndex = i;
      }
    }
    if (
      isRightDownloadTarget && toDownloadModel != null
      )
    {
      console.log("Exporting " + name);
      triggerDownload.value = false;
      let instanceToDownload = toDownloadModel.currentInstance++;
      // Prepare vertices output
      
      if(instanceToDownload >= instanceCount)
      {
        toDownload.splice(toDownloadIndex,1);
        console.log("Removed " + toDownload + " because it was done");
      }

      let bindGroupVertPrep = createSimpleBindGroup(
        [buffers.finalPatches2, dispatchVerticesStage, ],
        prepVerticesBinding0,
        device,
        uuid
      );
  
      let outputVerticesBindGroup = createSimpleBindGroup(
        [buffers.finalPatches2, vertOutputBuffer],
        vertexOutputLayout,
        device,
        uuid
      );

      let instanceSelectionBindGroup = createSimpleBindGroup(
        [target_instance_id_buffer],
        instanceSelectionLayout,
        device,
        null
      )
      let computePassPrep = commandEncoder.beginComputePass({
        label: "prepareVertexOutput",
      });
      computePassPrep.setPipeline(prepVerticesStageShaderAndPipeline.pipeline);
      computePassPrep.setBindGroup(0, bindGroupVertPrep);
      computePassPrep.dispatchWorkgroups(1);
      computePassPrep.end();

      simpleB2BSameSize(
        vertOutputBufferReset,
        vertOutputBuffer,
        commandEncoder
      );

      
      simpleB2BSameSize(numberBuffersArray[instanceToDownload],target_instance_id_buffer,commandEncoder);
      // Run vertex output shader
      let computePassVertices = commandEncoder.beginComputePass({
        label: "vertexOutputStage",
      });
      computePassVertices.setPipeline(compiledShader.verticesStage.pipeline);
      computePassVertices.setBindGroup(0, userInputBindGroup);
      computePassVertices.setBindGroup(1, outputVerticesBindGroup);
      computePassVertices.setBindGroup(2, instanceSelectionBindGroup);
      computePassVertices.dispatchWorkgroupsIndirect(dispatchVerticesStage, 0);
      computePassVertices.end();

      simpleB2BSameSize(vertOutputBuffer, vertReadableBuffer, commandEncoder);
      setTimeout(() => {
        vertReadableBuffer
          .mapAsync(GPUMapMode.READ, 0, vertReadableBuffer.size)
          .then(() => {

            console.log("Memory mapped the buffer for " + name + " instance " + instanceToDownload);
              const arrayBuffer = vertReadableBuffer
                .getMappedRange(0, vertReadableBuffer.size)
                .slice(0);
              let size = arrayBuffer[0];
              const vertexStream = new Float32Array(arrayBuffer);
              vertReadableBuffer.unmap();
              if (downloadAll) {
                triggerDownload.value = true;
              }
//              let l = lodExportParametersRefs.toDownload.value;
//              l.splice(l.indexOf(uuid), 1);
              
              console.log("exportMeshFromPatches called");
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
}
