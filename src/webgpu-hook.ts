import { assert } from "@stefnotch/typestef/assert";

const gpuDevicePromise = Promise.withResolvers<GPUDevice>();
let hasDevice = false;

type BufferLabel = string;
type WgpuBuffers<
  T extends {
    [key: string]: BufferLabel;
  },
> = {
  [Key in keyof T]: GPUBuffer[] | (GPUBuffer | null);
};
function makeRelevantBuffers<
  T extends {
    [key: string]: BufferLabel;
  },
>(
  values: T
): { buffers: WgpuBuffers<T>; lookupBuffer: Map<BufferLabel, keyof T> } {
  const lookupBuffer = new Map();
  for (const [key, label] of Object.entries(values)) {
    lookupBuffer.set(label, key);
  }
  const buffers = Object.fromEntries(
    Object.entries(values).map(([key, _]) => [key, null])
  ) as any;
  return { buffers, lookupBuffer };
}

const { buffers, lookupBuffer } = makeRelevantBuffers({
  time: "Time Buffer",
  screen: "Screen Buffer",
  mouse: "Mouse Buffer",
  computePatchesInput: "Compute Patches Input Buffer",
  finalPatches2: "Render Buffer 2",
  finalPatches4: "Render Buffer 4",
  finalPatches8: "Render Buffer 8",
  finalPatches16: "Render Buffer 16",
  finalPatches32: "Render Buffer 32",
  indirectDispatch0: "Indirect Compute Dispatch Buffer 0",
  indirectDispatch1: "Indirect Compute Dispatch Buffer 1",
  patches0: "Patches Buffer 0",
  patches1: "Patches Buffer 1",
  forceRender: "Force Render Uniform",
});

export const wgpuBuffers = buffers;
console.log(wgpuBuffers);

if (globalThis.navigator.gpu !== undefined) {
  hookFunction(globalThis.navigator.gpu, "requestAdapter", async (fn, args) => {
    const adapter = await fn(...args);
    if (adapter) {
      hookFunction(adapter, "requestDevice", async (fn, args) => {
        const device = await fn(...args);
        hookFunction(device, "createBuffer", (fn, args) => {
          const descriptor = args[0];
          const buffer = fn(...args);
          if (descriptor.label !== undefined) {
            const IdRegex = /ID(\d+) (.+)/;
            const match = descriptor.label.match(IdRegex);
            if (match) {
              const id = parseInt(match[1], 10);
              const key = lookupBuffer.get(match[2]);
              if (key !== undefined) {
                if (buffers[key] === null) {
                  buffers[key] = [];
                }
                assert(Array.isArray(buffers[key]));
                buffers[key][id] = buffer;
              }
            } else {
              const key = lookupBuffer.get(descriptor.label);
              if (key !== undefined) {
                buffers[key] = buffer;
              }
            }
          }
          return buffer;
        });
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
      });
    }
    return adapter;
  });
}

function hookFunction<T extends Record<string, any>, FnName extends keyof T>(
  obj: T,
  functionName: FnName,
  hookFunction: (
    original: T[FnName],
    args: Parameters<T[FnName]>
  ) => ReturnType<T[FnName]>
) {
  const original = obj[functionName].bind(obj);
  obj[functionName] = function (...args: Parameters<T[FnName]>) {
    return hookFunction(original as any, args);
  } as any;
}

export const GpuDevicePromise: Promise<GPUDevice> = gpuDevicePromise.promise;
