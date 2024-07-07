const gpuDevicePromise = Promise.withResolvers<GPUDevice>();
let hasDevice = false;

if (globalThis.navigator.gpu !== undefined) {
  hookFunction(globalThis.navigator.gpu, "requestAdapter", async (fn, args) => {
    const adapter = await fn(...args);
    if (adapter !== undefined) {
      hookFunction(adapter, "requestDevice", async (fn, args) => {
        const device = await fn(...args);
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
