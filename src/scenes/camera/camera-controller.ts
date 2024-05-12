import type { Quat, Vec3 } from "wgpu-matrix";

export interface CameraController {
  getPosition(): Vec3;
  getOrientation(): Quat;
}
