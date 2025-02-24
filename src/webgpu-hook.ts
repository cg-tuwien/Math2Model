import { assert } from "@stefnotch/typestef/assert";

const gpuDevicePromise = Promise.withResolvers<GPUDevice>();
let hasDevice = false;

export type LodStageBuffers = Record<BufferId, GPUBuffer>;

type BufferId =
  | "time"
  | "screen"
  | "mouse"
  | "computePatchesInput"
  | "finalPatches2"
  | "finalPatches4"
  | "finalPatches8"
  | "finalPatches16"
  | "finalPatches32"
  | "indirectDispatch0"
  | "indirectDispatch1"
  | "patches0"
  | "patches1"
  | "forceRender";

const bufferNameToId: Record<string, BufferId> = {
  "Time Buffer": "time",
  "Screen Buffer": "screen",
  "Mouse Buffer": "mouse",
  "Compute Patches Input Buffer": "computePatchesInput",
  "Render Buffer 2": "finalPatches2",
  "Render Buffer 4": "finalPatches4",
  "Render Buffer 8": "finalPatches8",
  "Render Buffer 16": "finalPatches16",
  "Render Buffer 32": "finalPatches32",
  "Indirect Compute Dispatch Buffer 0": "indirectDispatch0",
  "Indirect Compute Dispatch Buffer 1": "indirectDispatch1",
  "Patches Buffer 0": "patches0",
  "Patches Buffer 1": "patches1",
  "Force Render Uniform": "forceRender",
};

type BufferUUID = string;
const createdBuffers = new Map<BufferUUID, Partial<LodStageBuffers>>();
const createdBuffersWithoutUuid: Partial<LodStageBuffers> = {};

export function getBuffers(buffersUUID: BufferUUID): LodStageBuffers {
  const buffers = createdBuffers.get(buffersUUID);
  assert(buffers !== undefined);
  return { ...buffers, ...createdBuffersWithoutUuid } as any as LodStageBuffers;
}

export let renderEncoder: GPUCommandEncoder | null = null;

if (globalThis.navigator.gpu !== undefined) {
  const originalCreateBuffer = GPUDevice.prototype.createBuffer;
  GPUDevice.prototype.createBuffer = function (
    descriptor: GPUBufferDescriptor
  ) {
    const buffer = originalCreateBuffer.apply(this, [descriptor]);
    if (descriptor.label !== undefined) {
      const IdRegex = /(?<id>([a-z0-9]+-)+[a-z0-9]+) (?<name>.+)/;
      const match = descriptor.label.match(IdRegex);
      if (match) {
        const bufferUUID = match.groups!.id;
        const bufferName = match.groups!.name;

        let buffers = createdBuffers.get(bufferUUID);
        if (buffers === undefined) {
          buffers = {} as any as LodStageBuffers;
          createdBuffers.set(bufferUUID, buffers as any);
        }

        buffers[bufferNameToId[bufferName]] = buffer;
      } else {
        // the buffer only had a name
        const bufferName = descriptor.label;
        createdBuffersWithoutUuid[bufferNameToId[bufferName]] = buffer;
      }
    }
    return buffer;
  };

  const originalRequestDevice = GPUAdapter.prototype.requestDevice;
  GPUAdapter.prototype.requestDevice = async function (
    args: GPUDeviceDescriptor | undefined
  ) {
    const device = await originalRequestDevice.apply(this, [args]);
    gpuDevicePromise.resolve(device);
    if (!hasDevice) {
      hasDevice = true;
    } else {
      console.error(
        "New GPU device obtained, this should never happen",
        device
      );
    }
    return device;
  };

  const originalCreateCommandEncoder = GPUDevice.prototype.createCommandEncoder;
  GPUDevice.prototype.createCommandEncoder = function (
    descriptor?: GPUCommandEncoderDescriptor
  ) {
    const commandEncoder = originalCreateCommandEncoder.apply(this, [
      descriptor,
    ]);
    if (descriptor?.label === "Render Encoder") {
      renderEncoder = commandEncoder;
    }
    return commandEncoder;
  };
}

export const GpuDevicePromise: Promise<GPUDevice> = gpuDevicePromise.promise;
