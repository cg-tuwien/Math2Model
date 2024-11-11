import { getBuffers, renderEncoder, type LodStageBuffers } from "@/webgpu-hook";
import init, {
  WasmApplication,
  type WasmModelInfo,
  type WasmShaderInfo,
  type WasmCompilationMessage,
} from "../../parametric-renderer-core/pkg";

await init();

export class WgpuEngine {
  private constructor(private engine: WasmApplication) {}
  static createEngine(canvasElement: HTMLCanvasElement) {
    const engine = new WasmApplication();
    engine.run(canvasElement);
    return new WgpuEngine(engine);
  }
  async updateModels(js_models: WasmModelInfo[]) {
    await this.engine.update_models(js_models);
  }
  async updateShader(shader_info: WasmShaderInfo) {
    await this.engine.update_shader(shader_info);
  }
  async removeShader(id: string) {
    await this.engine.remove_shader(id);
  }
  setOnShaderCompiled(
    callback: (shaderId: string, messages: WasmCompilationMessage[]) => void
  ) {
    setTimeout(() => this.engine.set_on_shader_compiled(callback), 0);
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
