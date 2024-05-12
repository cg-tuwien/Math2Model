import { mat4, quat, vec3, type Mat4, type Quat, type Vec3 } from "wgpu-matrix";
import type { CameraSettings } from "./camera-settings";
import type { CameraController } from "./camera-controller";

// From https://iolite-engine.com/blog_posts/reverse_z_cheatsheet
const reverseZ = mat4.create(
  1,
  0,
  0,
  0, //
  0,
  1,
  0,
  0, //
  0,
  0,
  -1,
  1, //
  0,
  0,
  0,
  1 //
);

// Or maybe I should use https://docs.rs/glam/latest/src/glam/f32/sse2/mat4.rs.html#969-982

export class Camera {
  private _view: Mat4;
  private _projection: Mat4;
  constructor(
    public position: Vec3,
    public orientation: Quat,
    aspect: number,
    settings: CameraSettings
  ) {
    this._projection = mat4.mul(
      mat4.perspective(settings.fov, aspect, settings.zNear, settings.zFar),
      reverseZ
    );
    this._view = calculateViewMatrix(position, orientation);
  }

  update(controller: CameraController) {
    this._view = calculateViewMatrix(
      controller.getPosition(),
      controller.getOrientation()
    );
  }

  updateAspect(aspect: number) {
    // Ported from the Rust code
    this._projection[0] = -this._projection[5] / aspect;
  }

  get view(): Readonly<Mat4> {
    return this._view;
  }

  get projection(): Readonly<Mat4> {
    return this._projection;
  }

  static get forward(): Vec3 {
    return vec3.create(0, 0, -1);
  }
  static get up(): Vec3 {
    return vec3.create(0, 1, 0);
  }
  static get right(): Vec3 {
    return vec3.create(1, 0, 0);
  }
}

function calculateViewMatrix(position: Vec3, orientation: Quat): Mat4 {
  const cameraDirection = vec3.transformQuat(Camera.forward, orientation);
  const target = vec3.add(position, cameraDirection);
  return mat4.lookAt(position, target, Camera.up);
}
