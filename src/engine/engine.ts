export interface Engine {
  createBaseScene(): BaseScene;
  startRenderLoop(value: () => void): {
    stop: () => void;
  };
}
export interface BaseScene {
  update(): void;
  render(): void;
  [Symbol.dispose](): void;

  asBabylon(): any | null;
  asWgpu(): any | null;
}
