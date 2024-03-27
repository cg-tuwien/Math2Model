import { WebGPUEngine } from "@babylonjs/core";
import {
  ReactiveSceneFiles,
  SceneFilesWithFilesystem,
} from "./filesystem/scene-files";

export const sceneFilesPromise = (async () => {
  const fs = await SceneFilesWithFilesystem.create("some-key");
  return await ReactiveSceneFiles.create(fs);
})();

export const canvasElement = document.createElement("canvas");
canvasElement.style.width = "100%";
canvasElement.style.height = "100%";

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
