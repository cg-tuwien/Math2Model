import {
  ReadonlyQuaternion,
  ReadonlyVector3,
  type ShaderCodeRef,
  type VirtualModelState,
} from "@/scenes/VirtualScene";
import { computed, type ComputedRef } from "vue";
import { Quaternion, Vector3 } from "@babylonjs/core";

export interface WriteableModelState {
  id: string;
  name: string;
  code: ShaderCodeRef;
  posX: number;
  posY: number;
  posZ: number;
  rotX: number;
  rotY: number;
  rotZ: number;
  rotW: number;
  scale: number;
}

export function toWriteableModelState(
  model: VirtualModelState | undefined,
): ComputedRef<WriteableModelState | null> {
  if (model === undefined) return computed(() => null);
  const pos = model.position.toVector3();
  const rot = model.rotation.toQuaternion();
  return computed(() => {
    return {
      id: model.id.valueOf(),
      name: model.name.valueOf(),
      code: model.code,
      posX: pos.x.valueOf(),
      posY: pos.y.valueOf(),
      posZ: pos.z.valueOf(),
      rotX: rot.x.valueOf(),
      rotY: rot.y.valueOf(),
      rotZ: rot.z.valueOf(),
      rotW: rot.w.valueOf(),
      scale: model.scale.valueOf(),
    };
  });
}

export function fromWriteableModelState(
  model: WriteableModelState,
): VirtualModelState {
  return {
    id: model.id,
    name: model.name,
    code: model.code,
    position: ReadonlyVector3.fromVector3(
      new Vector3(model.posX, model.posY, model.posZ),
    ),
    rotation: ReadonlyQuaternion.fromQuaternion(
      new Quaternion(model.rotX, model.rotY, model.rotZ, model.rotW),
    ),
    scale: model.scale,
  };
}
