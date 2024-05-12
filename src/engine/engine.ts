import { Input } from "./input-manager";

export class Engine {
  input: Input;
  private _startTime: number = 0;
  constructor(public canvas: HTMLCanvasElement) {
    this.input = new Input(canvas);
    this._startTime = performance.now();
  }

  update() {
    const currentTime = performance.now();
    const deltaTime = (currentTime - this._startTime) / 1000;
    this.input.update();
  }
}
