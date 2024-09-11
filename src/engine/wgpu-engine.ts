import init, {
  init_engine,
  remove_shader,
  update_models,
  update_shader,
  type WasmModelInfo,
  type WasmShaderInfo,
} from "../../parametric-renderer-core/pkg";

await init();

export class WgpuEngine {
  private constructor(private engine: void) {}
  static async createEngine(canvasElement: HTMLCanvasElement) {
    const engine = init_engine(canvasElement);
    return new WgpuEngine(engine);
  }
  updateModels(js_models: WasmModelInfo[]) {
    update_models(js_models);
  }
  updateShader(shader_info: WasmShaderInfo) {
    update_shader(shader_info);
  }
  removeShader(id: string) {
    remove_shader(id);
  }
}
