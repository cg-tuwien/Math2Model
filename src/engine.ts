export interface Engine {
  createBaseScene(): BaseScene;
  resize(): void;
  startRenderLoop(value: () => void): {
    stop: () => void;
  };
}
export interface BaseScene {
  update(): void;
  render(): void;
  [Symbol.dispose](): void;

  // TODO: Temporary hack
  asBabylon(): any | null;
}
