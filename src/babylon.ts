import { Tools, WebGPUEngine } from "@babylonjs/core";
import type { Engine } from "./engine";
import { BabylonBaseScene } from "@/scenes/BaseScene";

WebGPUEngine.IsSupportedAsync.then((supported) => {
  if (!supported) {
    alert("WebGPU not supported");
  }
});

export class BabylonEngine implements Engine {
  private constructor(private engine: WebGPUEngine) {}
  static async createEngine(canvasElement: HTMLCanvasElement) {
    // GDPR compliance https://forum.babylonjs.com/t/offer-alternative-to-babylon-js-cdn/48982
    Tools.ScriptBaseUrl = "/babylon";

    const engine = new WebGPUEngine(canvasElement, {});
    engine.compatibilityMode = false;
    engine.onContextRestoredObservable.add(() => {
      engine.getCaps().canUseGLInstanceID = false;
    });

    await engine.initAsync();
    engine.getCaps().canUseGLInstanceID = false;
    return new BabylonEngine(engine);
  }

  resize(): void {
    this.engine.resize();
  }

  createBaseScene() {
    return new BabylonBaseScene(this.engine);
  }

  startRenderLoop(value: () => void): { stop: () => void } {
    this.engine.runRenderLoop(value);
    return {
      stop: () => {
        this.engine.stopRenderLoop(value);
      },
    };
  }
}
