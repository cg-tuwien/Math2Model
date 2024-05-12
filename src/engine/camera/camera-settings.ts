import { degToRad } from "wgpu-matrix/dist/2.x/utils";

export class CameraSettings {
  constructor(
    public readonly zNear: number,
    public readonly zFar: number,
    /** In radians */
    public readonly fov: number
  ) {}

  static default(): CameraSettings {
    return new CameraSettings(0.1, 100.0, degToRad(60));
  }
}
