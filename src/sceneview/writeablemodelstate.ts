import {
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
  rotW: number;
  scale: number;
}

export function toWriteableModelState(
  model: VirtualModelState
): WriteableModelState {
  const pos = model.position.toVector3();
  const rot = model.rotation.toQuaternion();
  return {
    name: model.name,
    code: model.code,
    posX: pos.x,
    posY: pos.y,
    posZ: pos.z,
    rotX: rot.x,
    rotY: rot.y,
    rotZ: rot.z,
    rotW: rot.w,
    scale: model.scale,
  };
}
