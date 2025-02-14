import { getBuffers, renderEncoder, type LodStageBuffers } from "@/webgpu-hook";
import init, {
  WasmApplication,
  type WasmModelInfo,
  type WasmShaderInfo,
} from "../../parametric-renderer-core/pkg";

await init();

export class WgpuEngine {
  private constructor(private engine: WasmApplication) {}
  static async createEngine(canvasElement: HTMLCanvasElement) {
    const engine = new WasmApplication();
    await engine.run(canvasElement);
    return new WgpuEngine(engine);
  }
  updateModels(js_models: WasmModelInfo[]) {
    this.engine.update_models(js_models);
  }
  updateShader(shader_info: WasmShaderInfo) {
    this.engine.update_shader(shader_info);
  }
  removeShader(id: string) {
    this.engine.remove_shader(id);
  }
  setLodStage(
    callback:
      | null
      | ((
          shaderPath: string,
          buffers: LodStageBuffers,
          commandEncoder: GPUCommandEncoder
        ) => void)
  ) {
    if (callback === null) {
      this.engine.set_lod_stage();
    } else {
      this.engine.set_lod_stage((shaderPath: string, buffersIndex: number) => {

        if (renderEncoder === null) {
          console.error("renderEncoder is null");
        } else {
          callback(shaderPath, getBuffers(+buffersIndex), renderEncoder);
        }
      });
    }
  }
}
