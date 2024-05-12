import { quat, vec3, type Quat, type Vec2, type Vec3 } from "wgpu-matrix";
import { Camera } from "./camera";
import type { CameraController } from "./camera-controller";
import { PointerButton, type Input } from "../input-manager";

export class FreecamController implements CameraController {
  public position: Vec3 = vec3.create();
  public pitch: number = 0;
  public yaw: number = 0;
  constructor(
    public speed: number,
    public sensitivity: number
  ) {}

  getPosition(): Vec3 {
    return this.position;
  }
  getOrientation(): Quat {
    return quat.fromEuler(this.pitch, this.yaw, 0, "xyz");
  }

  update(input: Input, deltaTime: number) {
    if (input.isMouseDown(PointerButton.Right)) {
      this.updateOrientation(input.getMouseDelta());
    }
    this.updatePosition(inputToDirection(input), deltaTime);
  }

  updateOrientation(mouseDelta: Vec2) {
    this.setPitchYaw(
      this.pitch + mouseDelta[1] * this.sensitivity,
      this.yaw - mouseDelta[0] * this.sensitivity
    );
  }

  setPitchYaw(newPitch: number, newYaw: number) {
    const maxPitch = 88;
    this.pitch = clamp(newPitch, -maxPitch, maxPitch);
    this.yaw = newYaw % (Math.PI * 2);
  }

  updatePosition(direction: Vec3, deltaTime: number) {
    const horizontalMovement = normalizeOrZero(
      vec3.fromValues(direction[0], 0, direction[2])
    );
    const verticalMovement = vec3.mulScalar(Camera.up, direction[1]);
    const horizontalMovementRotated = vec3.rotateY(
      horizontalMovement,
      vec3.create(),
      this.yaw
    );
    this.position = vec3.add(
      this.position,
      vec3.scale(
        vec3.add(horizontalMovementRotated, verticalMovement),
        this.speed * deltaTime
      )
    );
  }
}

function normalizeOrZero(v: Vec3): Vec3 {
  const length = vec3.length(v);
  if (length <= 0.0001) {
    return vec3.create();
  }
  return vec3.scale(v, 1 / length);
}

function clamp(value: number, min: number, max: number) {
  return Math.min(Math.max(value, min), max);
}

function inputToDirection(input: Input) {
  let direction = vec3.create();
  if (input.isPhysicalKeyDown("KeyW")) {
    vec3.add(direction, Camera.forward);
  }
  if (input.isPhysicalKeyDown("KeyS")) {
    vec3.sub(direction, Camera.forward);
  }
  if (input.isPhysicalKeyDown("KeyD")) {
    vec3.sub(direction, Camera.right);
  }
  if (input.isPhysicalKeyDown("KeyA")) {
    vec3.add(direction, Camera.right);
  }
  if (input.isPhysicalKeyDown("Space")) {
    vec3.add(direction, Camera.up);
  }
  if (
    input.isPhysicalKeyDown("ShiftLeft") ||
    input.isPhysicalKeyDown("ShiftRight")
  ) {
    vec3.sub(direction, Camera.up);
  }
  return direction;
}
