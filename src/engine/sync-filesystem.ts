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
        fs
          .readBinaryFile(file, { signal })
          ?.then((data) => {
            if (signal.aborted) return;
            // Ugh, Typescript hasn't caught up
            const imageDecoder = new ImageDecoder({
              data,
              type: imageFileTypes.get(extension)!,
            });
            return imageDecoder.decode();
          })
          .then(async (result) => {
            if (signal.aborted) return;
            if (result === undefined) return;
            const image = result.image;
            const imageRect =
              image.visibleRect ??
              new DOMRectReadOnly(0, 0, image.codedWidth, image.codedHeight);
            const options = {
              rect: imageRect,
              // MDN says that this property is a thing
              format: "RGBA",
            };
            const buffer = new ArrayBuffer(image.allocationSize(options));
            await image.copyTo(buffer, options);

            const data = new Uint8Array(buffer);
            // TODO: Image decoding
            engine.updateTexture({
              id: file,
              width: imageRect.width,
              // Wow, inefficient
              data: Array.from(data),
            });
          });
      } else if (change.type === "remove") {
        engine.removeTexture(change.key);
      }
    }
  });

  return {};
}
