import { vec2, type Vec2 } from "wgpu-matrix";

export enum PointerButton {
  Left = 0,
  Middle = 1,
  Right = 2,
  Back = 3,
  Forward = 4,
}
export class Input {
  private pressedPhysicalKeys = new Set<string>();
  private pressedLogicalKeys = new Set<string>();
  private pressedPointers = new Set<PointerButton>();
  private mouseCurrent: Vec2 = vec2.create();
  private mouseDelta: Vec2 = vec2.create();
  private frameEvents: (() => void)[] = [];
  constructor(public element: HTMLElement) {
    this.element = element;
    this.element.addEventListener("keydown", this.onKeyDown.bind(this));
    this.element.addEventListener("keyup", this.onKeyUp.bind(this));
    this.element.addEventListener("pointerdown", this.onPointerDown.bind(this));
    this.element.addEventListener("pointerup", this.onPointerUp.bind(this));
    this.element.addEventListener("pointermove", this.onPointerMove.bind(this));
  }

  update() {
    const mouseStart = this.mouseCurrent.slice();
    this.frameEvents.forEach((event) => event());
    this.mouseDelta = vec2.sub(this.mouseCurrent, mouseStart);
  }

  private onKeyDown(event: KeyboardEvent) {
    this.frameEvents.push(() => {
      this.pressedPhysicalKeys.add(event.code);
      this.pressedLogicalKeys.add(event.key);
    });
  }

  private onKeyUp(event: KeyboardEvent) {
    this.frameEvents.push(() => {
      this.pressedPhysicalKeys.delete(event.code);
      this.pressedLogicalKeys.delete(event.key);
    });
  }

  private onPointerDown(event: PointerEvent) {
    this.frameEvents.push(() => {
      this.pressedPointers.add(event.button);
    });
  }

  private onPointerUp(event: PointerEvent) {
    this.frameEvents.push(() => {
      this.pressedPointers.delete(event.button);
    });
  }

  private onPointerMove(event: PointerEvent) {
    this.frameEvents.push(() => {
      this.mouseCurrent = vec2.create(event.offsetX, event.offsetY);
    });
  }

  isPhysicalKeyDown(key: string) {
    return this.pressedPhysicalKeys.has(key);
  }

  isLogicalKeyDown(key: string) {
    return this.pressedLogicalKeys.has(key);
  }

  isMouseDown(button: PointerButton) {
    return this.pressedPointers.has(button);
  }

  getMouseDelta() {
    return this.mouseDelta;
  }

  [Symbol.dispose]() {
    this.element.removeEventListener("keydown", this.onKeyDown);
    this.element.removeEventListener("keyup", this.onKeyUp);
    this.element.removeEventListener("pointerdown", this.onPointerDown);
    this.element.removeEventListener("pointerup", this.onPointerUp);
    this.element.removeEventListener("pointermove", this.onPointerMove);
  }
}
