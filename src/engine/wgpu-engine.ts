import { getBuffers, renderEncoder, type LodStageBuffers } from "@/webgpu-hook";
import init, {
  WasmApplication,
  type WasmModelInfo,
  type WasmShaderInfo,
  type WasmCompilationMessage,
  type WasmFrameTime,
} from "../../parametric-renderer-core/pkg";

await init();

/** Wraps the Rust engine in fire-and-forget functions. They will always be execude in-order */
export class WgpuEngine {
  private taskQueue: Promise<void> = Promise.resolve();
  private constructor(private engine: WasmApplication) {}
  static createEngine(canvasElement: HTMLCanvasElement) {
    const engine = new WasmApplication();
    engine.run(canvasElement);
    return new WgpuEngine(engine);
  }
  async updateModels(js_models: WasmModelInfo[]) {
    this.taskQueue = this.taskQueue.then(() =>
      this.engine.update_models(js_models)
    );
    await this.taskQueue;
  }
  async updateShader(shader_info: WasmShaderInfo) {
    this.taskQueue = this.taskQueue.then(() =>
      this.engine.update_shader(shader_info)
    );
    await this.taskQueue;
  }
  async removeShader(id: string) {
    this.taskQueue = this.taskQueue.then(() => this.engine.remove_shader(id));
    await this.taskQueue;
  }
  async updateTexture(texture_info: { id: string; bitmap: ImageBitmap }) {
    this.taskQueue = this.taskQueue.then(() =>
      this.engine.update_texture(texture_info.id, texture_info.bitmap)
    );
    await this.taskQueue;
  }
  async removeTexture(id: string) {
    this.taskQueue = this.taskQueue.then(() => this.engine.remove_texture(id));
    await this.taskQueue;
  }
  async setOnShaderCompiled(
    callback: (shaderId: string, messages: WasmCompilationMessage[]) => void
  ) {
    this.taskQueue = this.taskQueue.then(() =>
      this.engine.set_on_shader_compiled(callback)
    );
    await this.taskQueue;
  }
  async setLodStage(
    callback:
      | null
      | ((
          shaderPath: string,
          buffers: LodStageBuffers,
          commandEncoder: GPUCommandEncoder
        ) => void)
  ) {
    if (callback === null) {
      this.taskQueue = this.taskQueue.then(() => this.engine.set_lod_stage());
    } else {
      this.taskQueue = this.taskQueue.then(() =>
        this.engine.set_lod_stage((shaderPath: string, buffersUUID: string) => {
          if (renderEncoder === null) {
            console.error("renderEncoder is null");
          } else {
            callback(shaderPath, getBuffers(buffersUUID), renderEncoder);
          }
        })
      );
    }
    await this.taskQueue;
  }
  async getFrameTime(): Promise<WasmFrameTime> {
    let { promise, resolve } = Promise.withResolvers<WasmFrameTime>();
    this.taskQueue = this.taskQueue.then(() => {
      resolve(this.engine.get_frame_time());
    });
    await this.taskQueue;
    return promise;
  }
  async setThresholdFactor(factor: number) {
    this.taskQueue = this.taskQueue.then(() =>
      this.engine.set_threshold_factor(factor)
    );
    await this.taskQueue;
  }

  async focusOn(position: [number, number, number]) {
    this.taskQueue = this.taskQueue.then(() => this.engine.focus_on(position));
    await this.taskQueue;
  }
}
