import {
  ReactiveFiles,
  FilesWithFilesystem,
  makeFilePath,
} from "./filesystem/reactive-files";

export const sceneFilesPromise = (async () => {
  const fs = await FilesWithFilesystem.create(makeFilePath("some-key"));
  return await ReactiveFiles.create(fs);
})();

const canvasElement = document.createElement("canvas");
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
let isCanvasTaken = false;

export const takeCanvas = () => {
  if (isCanvasTaken) {
    return null;
  }
  isCanvasTaken = true;
  return canvasElement;
};
