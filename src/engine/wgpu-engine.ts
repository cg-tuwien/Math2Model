import { watchEffect, type ComputedRef } from "vue";
import init, {
  init_engine,
  update_models,
} from "../../parametric-renderer-core/pkg";
import type { BaseScene, Engine } from "./engine";
import type { VirtualSceneState } from "@/scenes/VirtualScene";

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
  startRenderLoop(
    _value: () => void,
    scene: ComputedRef<VirtualSceneState>
  ): { stop: () => void } {
    watchEffect(() => {
      update_models(
        scene.value.models.map((v) => ({
          label: v.name,
          transform: {
            position: [v.position.x, v.position.y, v.position.z],
            rotation: [v.rotation.x, v.rotation.y, v.rotation.z],
            scale: v.scale,
          },
          material_info: {
            color: [1, 0.5, 0.2],
            emissive: [0, 0, 0],
            roughness: 0.5,
            metallic: 0.5,
          },
          evaluate_image_code:
            // TODO: Implement this
            "fn evaluateImage(input2: vec2f) -> vec3f { return vec3(input2, 0.0); }",
        }))
      );
    });
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
