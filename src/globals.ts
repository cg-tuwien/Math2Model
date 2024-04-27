import { Tools, WebGPUEngine } from "@babylonjs/core";
import {
  ReactiveFiles,
  FilesWithFilesystem,
  makeFilePath,
} from "./filesystem/reactive-files";

// GDPR compliance https://forum.babylonjs.com/t/offer-alternative-to-babylon-js-cdn/48982
Tools.ScriptBaseUrl = "/babylon";

export const sceneFilesPromise = (async () => {
  const fs = await FilesWithFilesystem.create(makeFilePath("some-key"));
  return await ReactiveFiles.create(fs);
})();

export const canvasElement = document.createElement("canvas");
canvasElement.style.width = "100%";
canvasElement.style.height = "100%";
canvasElement.addEventListener('wheel', e => { e.preventDefault(); e.stopPropagation() });

export const enginePromise = (async () => {
  const engine = new WebGPUEngine(canvasElement, {});
  engine.compatibilityMode = false;
  engine.onContextRestoredObservable.add(() => {
    engine.getCaps().canUseGLInstanceID = false;
  });

  await engine.initAsync();
  engine.getCaps().canUseGLInstanceID = false;
  return engine;
})();
