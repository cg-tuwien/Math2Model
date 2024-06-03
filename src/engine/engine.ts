import type { VirtualSceneState } from "@/scenes/VirtualScene";
import type { ComputedRef } from "vue";

export interface Engine {
  createBaseScene(): BaseScene;
  startRenderLoop(
    value: () => void,
    scene: ComputedRef<VirtualSceneState>
  ): {
    stop: () => void;
  };
}
export interface BaseScene {
  update(): void;
  render(): void;
  [Symbol.dispose](): void;

  asBabylon(): any | null;
}
