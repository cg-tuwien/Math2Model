import { Zoom } from "rete-area-plugin";
import type { OnZoom } from "rete-area-plugin/_types/zoom";

/**
 * Zoom source
 */
export type ZoomSource = "wheel" | "touch";

/**
 * Zoom class, used to handle zooming of the area. Can be extended to add custom behavior.
 * @internal
 */
export class CustomZoom extends Zoom {
  protected previous: { cx: number; cy: number; distance: number } | null =
    null;
  protected pointers: PointerEvent[] = [];
  declare protected container: HTMLElement;
  declare protected element: HTMLElement;
  declare protected onzoom: OnZoom;

  constructor(protected intensity: number) {
    super(intensity);
  }

  public override initialize(
    container: HTMLElement,
    element: HTMLElement,
    onzoom: OnZoom
  ) {
    this.container = container;
    this.element = element;
    this.onzoom = onzoom;
    this.container.addEventListener("wheel", this.wheel);
    this.container.addEventListener("pointerdown", this.down);

    window.addEventListener("pointermove", this.move);
    window.addEventListener("pointerup", this.up);
    window.addEventListener("pointercancel", this.up);
    window.addEventListener("contextmenu", this.contextmenu);
  }
}
