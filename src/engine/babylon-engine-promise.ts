import { takeCanvas } from "@/globals";
import { BabylonEngine } from "./babylon-engine";

let babylonEngine: { engine: BabylonEngine; canvas: HTMLCanvasElement } | null =
  null;

export async function getEngine() {
  if (babylonEngine !== null) {
    return babylonEngine;
  }

  const canvasElement = takeCanvas();
  if (canvasElement === null) {
    window.location.reload();
    throw new Error("Canvas element already used, reloading the site.");
  }
  let engine = await BabylonEngine.createEngine(canvasElement);
  babylonEngine = { engine, canvas: canvasElement };
  return babylonEngine;
}
