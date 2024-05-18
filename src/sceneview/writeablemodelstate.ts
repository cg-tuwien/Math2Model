import { makeFilePath } from "@/filesystem/reactive-files";
import {
  ReadonlyEulerAngles,
  ReadonlyVector3,
  type ShaderCodeRef,
  type VirtualModelState,
} from "@/scenes/VirtualScene";

export interface WriteableModelState {
  name: string;
  code: ShaderCodeRef;
  posX: number;
  posY: number;
  posZ: number;
  rotX: number;
  rotY: number;
  rotZ: number;
  scale: number;
}

export function toWriteableModelState(
  model: VirtualModelState
): WriteableModelState {
  const pos = model.position;
  const rot = model.rotation;
  return {
    name: model.name,
    code: model.code,
    posX: pos.x,
    posY: pos.y,
    posZ: pos.z,
    rotX: rot.x,
    rotY: rot.y,
    rotZ: rot.z,
    scale: model.scale,
  };
}

export function commonWriteableModelState(
  models: VirtualModelState[]
): WriteableModelState {
  const output: VirtualModelState = {
    id: "",
    name: "",
    code: { vertexFile: makeFilePath(""), fragmentFile: makeFilePath("") },
    position: ReadonlyVector3.zero,
    rotation: ReadonlyEulerAngles.identity,
    scale: 1,
  };
  if (models.length === 0) {
    return toWriteableModelState(output);
  }

  const first = models[0];
  if (checkSamePropertyValues(models, "name")) {
    output.name = first.name;
  }
  if (checkSamePropertyValues(models, "scale")) {
    output.scale = first.scale;
  }
  positionEquals(models, output);
  rotationEquals(models, output);
  return toWriteableModelState(output);
}

function checkSamePropertyValues(
  models: VirtualModelState[],
  property: keyof VirtualModelState
): boolean {
  if (models.length === 0) {
    return false; // No objects to compare
  }

  const first = models[0];

  for (let i = 1; i < models.length; i++) {
    if (models[i][property] !== first[property]) {
      return false; // Different value found
    }
  }

  return true; // All values are the same
}

function positionEquals(
  models: VirtualModelState[],
  canvas: VirtualModelState
) {
  if (models.length === 0) {
    return; // No objects to compare
  }

  const first = models[0];

  canvas.position = first.position;

  for (let i = 1; i < models.length; i++) {
    if (models[i].position.x != first.position.x) {
      canvas.position = new ReadonlyVector3(
        0,
        canvas.position.y,
        canvas.position.z
      );
    }
    if (models[i].position.y != first.position.y) {
      canvas.position = new ReadonlyVector3(
        canvas.position.x,
        0,
        canvas.position.z
      );
    }
    if (models[i].position.z != first.position.z) {
      canvas.position = new ReadonlyVector3(
        canvas.position.x,
        canvas.position.y,
        0
      );
    }
  }
}

function rotationEquals(
  models: VirtualModelState[],
  canvas: VirtualModelState
) {
  if (models.length === 0) {
    return; // No objects to compare
  }

  const first = models[0];

  canvas.rotation = first.rotation;

  for (let i = 1; i < models.length; i++) {
    if (models[i].rotation.x != first.rotation.x) {
      canvas.rotation = new ReadonlyEulerAngles(
        0,
        canvas.rotation.y,
        canvas.rotation.z
      );
    }
    if (models[i].rotation.y != first.rotation.y) {
      canvas.rotation = new ReadonlyEulerAngles(
        canvas.rotation.x,
        0,
        canvas.rotation.z
      );
    }
    if (models[i].rotation.z != first.rotation.z) {
      canvas.rotation = new ReadonlyEulerAngles(
        canvas.rotation.x,
        canvas.rotation.y,
        0
      );
    }
  }
}
