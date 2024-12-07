import {
  getFileExtension,
  type FilePath,
  type ReactiveFilesystem,
} from "@/filesystem/reactive-files";
import type { WgpuEngine } from "./wgpu-engine";
import DefaultShaderCode from "@/../parametric-renderer-core/shaders/DefaultParametric.wgsl?raw";

/**
 * Sync the filesystem with the Rust backend
 */
export function syncFilesystem(fs: ReactiveFilesystem, engine: WgpuEngine) {
  const opsSignals = new Map<FilePath, AbortController>();
  const addSignal = (path: FilePath): AbortSignal => {
    const controller = new AbortController();
    opsSignals.set(path, controller);
    return controller.signal;
  };
  const stopPending = (path: FilePath): void => {
    opsSignals.get(path)?.abort();
    opsSignals.delete(path);
  };

  const imageFileTypes = new Map<string, string>([
    ["png", "image/png"],
    ["avif", "image/avif"],
    ["bmp", "image/bmp"],
    ["gif", "image/gif"],
    ["jpg", "image/jpeg"],
    ["jpeg", "image/jpeg"],
    ["jpe", "image/jpeg"],
    ["jif", "image/jpeg"],
    ["jfif", "image/jpeg"],
    ["tif", "image/tiff"],
    ["tiff", "image/tiff"],
    ["webp", "image/webp"],
  ]);

  fs.watchFromStart((change) => {
    const extension = getFileExtension(change.key);
    if (extension === null) return;
    if (extension === "wgsl") {
      stopPending(change.key);
      if (change.type === "insert") {
        engine.updateShader({
          id: change.key,
          label: change.key,
          code: DefaultShaderCode,
        });
      }

      if (change.type === "insert" || change.type === "update") {
        const file = change.key;
        const signal = addSignal(file);
        fs.readTextFile(file, { signal })?.then((code) => {
          if (signal.aborted) return;
          engine.updateShader({
            id: file,
            label: file,
            code: code as string,
          });
        });
      } else if (change.type === "remove") {
        engine.removeShader(change.key);
      }
    } else if (imageFileTypes.has(extension)) {
      stopPending(change.key);
      if (change.type === "insert" || change.type === "update") {
        const file = change.key;
        const signal = addSignal(file);
        fs.readFile(file, { signal })?.then(async (blob) => {
          if (signal.aborted) return;
          const image = await globalThis.createImageBitmap(blob, {
            premultiplyAlpha: "premultiply",
            colorSpaceConversion: "none",
          });
          if (signal.aborted) return;
          engine.updateTexture({
            id: file,
            bitmap: image,
          });
        });
      } else if (change.type === "remove") {
        engine.removeTexture(change.key);
      }
    }
  });

  return {};
}
