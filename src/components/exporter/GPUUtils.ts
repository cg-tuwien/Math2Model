import { createHelpers } from "../webgpu-helpers";
export function concatArrayBuffers(
  props: any,
  views: ArrayBufferView[]
): Uint8Array {
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

export function createBufferWith(
  props: any,
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
  return buffer
}

/**
 * simpleBindGroup
 */
export function createBindGroupLayout(
  shaderStage: GPUShaderStageFlags,
  types: string[],
  device: any,
  name: string
) {
  let entries: any[] = [];
  let i = 0;
  types.forEach((e) => {
    entries.push({
      binding: i,
      visibility: shaderStage,
      buffer: {
        type: e,
      },
    });
    i++;
  });
  return device.createBindGroupLayout(
    {
      entries: entries,
      label: name
    },
    name
  );
}
let bindGroupCache: Map<string, Map<string, GPUBindGroup>> = new Map();

export function createSimpleBindGroup(
  buffers: GPUBuffer[],
  bindgroupLayout: GPUBindGroupLayout,
  device: GPUDevice,
  guid: string | null
) {
  

  let entries: GPUBindGroupEntry[] = buffers.map((buffer, i) => ({
    binding: i,
    resource: { buffer: buffer },
  }));

  let bindGroup = device.createBindGroup({
    layout: bindgroupLayout,
    entries: entries,
  });
  if(guid)
    bindGroup.label = guid;

  
  return bindGroup;
}


export function simpleB2BSameSize(
  bufferSource: GPUBuffer,
  bufferTarget: GPUBuffer,
  commandEncoder: GPUCommandEncoder
) {
  commandEncoder.copyBufferToBuffer(
    bufferSource,
    0,
    bufferTarget,
    0,
    bufferSource.size
  );
}

export function createShaderWithPipeline(
  code: string,
  shaderPipelineLayout: GPUPipelineLayout,
  device: GPUDevice
): {
  pipeline: GPUComputePipeline;
  shader: GPUShaderModule;
} {
  const shader = device.createShaderModule({
    code,
  });
  shader.getCompilationInfo().then((info: any) => {
    console.log("Shader compilation resulted in ", info);
  });
  const pipeline = device.createComputePipeline({
    layout: shaderPipelineLayout,
    compute: {
      module: shader,
    },
  });
  return { shader, pipeline };
}
