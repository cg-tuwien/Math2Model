import { WebGPUEngine } from "@babylonjs/core";
import { markRaw, shallowRef } from "vue";

export const canvasElement = document.createElement("canvas");
canvasElement.style.width = "100%";
canvasElement.style.height = "100%";
export const engine = shallowRef<WebGPUEngine | null>(null);

async function load() {
  const e = new WebGPUEngine(canvasElement, {});
  e.compatibilityMode = false;

  if (import.meta.hot) {
    import.meta.hot.dispose(() => {
      e.dispose();
    });
  }

  e.initAsync().then(() => {
    if (e.isDisposed) {
      return;
    }
    e.getCaps().canUseGLInstanceID = false;
    engine.value = markRaw(e);
  });

  e.onContextRestoredObservable.add(() => {
    if (e.isDisposed) {
      return;
    }
    e.getCaps().canUseGLInstanceID = false;
  });
}
load();

if (import.meta.hot) {
  import.meta.hot.accept((newModule) => {
    engine.value = null;
    if (newModule) {
      newModule.load();
    }
  });
}
