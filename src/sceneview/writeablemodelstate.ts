import {
  ReadonlyQuaternion,
  ReadonlyVector3,
  type ShaderCodeRef,
  type VirtualModelState,
} from "@/scenes/VirtualScene";
import VirtualModel from "@/components/VirtualModel.vue";
import { Angle, Quaternion, Vector3 } from "@babylonjs/core";

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
  model: VirtualModelState,
): WriteableModelState {
  const pos = model.position.toVector3();
  const rot = model.rotation.toQuaternion().toEulerAngles();
  return {
    name: model.name,
    code: model.code,
    posX: pos.x,
    posY: pos.y,
    posZ: pos.z,
    rotX: parseFloat(new Angle(rot.x).degrees().toFixed(2)),
    rotY: parseFloat(new Angle(rot.y).degrees().toFixed(2)),
    rotZ: parseFloat(new Angle(rot.z).degrees().toFixed(2)),
    scale: model.scale,
  };
}

export function commonWriteableModelState(
  models: VirtualModelState[],
): WriteableModelState {
  const canvas: VirtualModelState = {
    id: "",
    name: "",
    code: models[0].code,
    position: ReadonlyVector3.zero,
    rotation: ReadonlyQuaternion.identity,
    scale: 1,
  };

  const first = models[0];

  // For some reason Vue.js doesn't let me do that
  /**for (let property in first) {
    if (checkSamePropertyValues(models, property)) {
      canvas[property] = first[property];
    }
  }*/

  if (checkSamePropertyValues(models, "name")) {
    canvas.name = first.name;
  }
  if (checkSamePropertyValues(models, "scale")) {
    canvas.scale = first.scale;
  }

  positionEquals(models, canvas);
  rotationEquals(models, canvas);

  return toWriteableModelState(canvas);
}

function checkSamePropertyValues(
  models: VirtualModelState[],
  property: keyof VirtualModelState,
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
  canvas: VirtualModelState,
) {
  if (models.length === 0) {
    return; // No objects to compare
  }

  const first = models[0];

  canvas.position = first.position;

  for (let i = 1; i < models.length; i++) {
    if (models[i].position.x != first.position.x) {
      canvas.position = ReadonlyVector3.fromVector3(
        new Vector3(0, canvas.position.y, canvas.position.z),
      );
    }
    if (models[i].position.y != first.position.y) {
      canvas.position = ReadonlyVector3.fromVector3(
        new Vector3(canvas.position.x, 0, canvas.position.z),
      );
    }
    if (models[i].position.z != first.position.z) {
      canvas.position = ReadonlyVector3.fromVector3(
        new Vector3(canvas.position.x, canvas.position.y, 0),
      );
    }
  }
}

function rotationEquals(
  models: VirtualModelState[],
  canvas: VirtualModelState,
) {
  if (models.length === 0) {
    return; // No objects to compare
  }

  const first = models[0];

  canvas.rotation = first.rotation;

  for (let i = 1; i < models.length; i++) {
    if (models[i].rotation.x != first.rotation.x) {
      canvas.rotation = ReadonlyQuaternion.fromQuaternion(
        new Quaternion(
          0,
          canvas.rotation.y,
          canvas.rotation.z,
          canvas.rotation.w,
        ),
      );
    }
    if (models[i].rotation.y != first.rotation.y) {
      canvas.rotation = ReadonlyQuaternion.fromQuaternion(
        new Quaternion(
          canvas.rotation.x,
          0,
          canvas.rotation.z,
          canvas.rotation.w,
        ),
      );
    }
    if (models[i].rotation.z != first.rotation.z) {
      canvas.rotation = ReadonlyQuaternion.fromQuaternion(
        new Quaternion(
          canvas.rotation.x,
          canvas.rotation.y,
          0,
          canvas.rotation.w,
        ),
      );
    }
    if (models[i].rotation.w != first.rotation.w) {
      canvas.rotation = ReadonlyQuaternion.fromQuaternion(
        new Quaternion(
          canvas.rotation.x,
          canvas.rotation.y,
          canvas.rotation.z,
          0,
        ),
      );
    }
  }
}
