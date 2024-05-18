import init, { init_engine } from "../../parametric-renderer-core/pkg";
import type { BaseScene, Engine } from "./engine";

await init();

export class WgpuEngine implements Engine {
  private constructor(private engine: void) {}
  static async createEngine(canvasElement: HTMLCanvasElement) {
    const engine = init_engine(canvasElement);
    return new WgpuEngine(engine);
  }
  createBaseScene(): BaseScene {
    return new WgpuBaseScene();
  }
  startRenderLoop(_value: () => void): { stop: () => void } {
    return { stop: () => {} };
  }
}

// Dummy implementation
export class WgpuBaseScene implements BaseScene {
  update(): void {}
  render(): void {}
  asBabylon() {
    return null;
  }
  [Symbol.dispose](): void {}
}
