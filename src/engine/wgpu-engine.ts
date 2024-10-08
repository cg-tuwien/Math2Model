import init, {
  init_engine,
  update_models,
} from "../../parametric-renderer-core/pkg";

await init();

export class WgpuEngine {
  private constructor(private engine: void) {}
  static async createEngine(canvasElement: HTMLCanvasElement) {
    const engine = init_engine(canvasElement);
    return new WgpuEngine(engine);
  }
  updateModels(js_models: any) {
    update_models(js_models);
  }
}
