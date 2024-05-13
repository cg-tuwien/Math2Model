import {
  ReactiveFiles,
  FilesWithFilesystem,
  makeFilePath,
} from "./filesystem/reactive-files";
import { BabylonEngine } from "./babylon";

export const sceneFilesPromise = (async () => {
  const fs = await FilesWithFilesystem.create(makeFilePath("some-key"));
  return await ReactiveFiles.create(fs);
})();

export const canvasElement = document.createElement("canvas");
canvasElement.style.width = "100%";
canvasElement.style.height = "100%";
canvasElement.addEventListener(
  "wheel",
  (e) => {
    e.preventDefault();
    e.stopPropagation();
  },
  {
    passive: false,
  }
);

export const enginePromise = BabylonEngine.createEngine(canvasElement);
