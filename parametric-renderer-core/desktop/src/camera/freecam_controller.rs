use glam::Quat;
use glamour::{Angle, Point3, Vector2, Vector3};
use winit::{event::MouseButton, keyboard::KeyCode};
use winit_input_helper::WinitInputHelper;

use super::{camera_controller::CameraController, Camera};

pub struct FreecamController {
    pub position: Point3,
    pub pitch: Angle,
    pub yaw: Angle,
    pub speed: f32,
    pub sensitivity: f32,
}

impl FreecamController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            position: Point3::ZERO,
            pitch: Angle::default(),
            yaw: Angle::default(),
            speed,
            sensitivity,
        }
    }
    pub fn update(&mut self, input: &WinitInputHelper, delta_time: f32) {
        if input.mouse_held(MouseButton::Right) {
            self.update_orientation(Vector2::from(input.mouse_diff()));
        }

        self.update_position(input_to_direction(input), delta_time);
    }

    fn update_orientation(&mut self, mouse_delta: Vector2) {
        self.set_pitch_yaw(
            self.pitch + Angle::new(mouse_delta.y * self.sensitivity),
            self.yaw - Angle::new(mouse_delta.x * self.sensitivity),
        );
    }

    fn set_pitch_yaw(&mut self, new_pitch: Angle, new_yaw: Angle) {
        const TWO_PI: f32 = std::f32::consts::PI * 2.0;
        let max_pitch = 88f32;
        self.pitch = new_pitch
            .min(Angle::from_degrees(max_pitch))
            .max(Angle::from_degrees(-max_pitch));
        self.yaw = Angle::new(new_yaw.radians.rem_euclid(TWO_PI));
    }

    fn update_position(&mut self, direction: Vector3, delta_time: f32) {
        let horizontal_movement = (direction * Vector3::new(1.0, 0.0, 1.0)).normalize_or_zero();
        let vertical_movement = Camera::up() * direction.y;
        let horizontal_movement = self.get_yaw_rotation() * horizontal_movement;

        self.position += horizontal_movement * self.speed * delta_time;
        self.position += vertical_movement * self.speed * delta_time;
    }

    fn get_yaw_rotation(&self) -> Quat {
        Quat::from_rotation_y(-self.yaw.radians)
    }

    fn get_pitch_rotation(&self) -> Quat {
        Quat::from_rotation_x(-self.pitch.radians)
    }
}

impl CameraController for FreecamController {
    fn position(&self) -> Point3 {
        self.position
    }

    fn orientation(&self) -> Quat {
        self.get_yaw_rotation() * self.get_pitch_rotation()
    }
}

fn input_to_direction(input: &WinitInputHelper) -> Vector3 {
    let mut direction = Vector3::ZERO;
    if input.key_held(KeyCode::KeyW) {
        direction += Camera::forward();
    }
    if input.key_held(KeyCode::KeyS) {
        direction -= Camera::forward();
    }

    if input.key_held(KeyCode::KeyD) {
        direction -= Camera::right();
    }
    if input.key_held(KeyCode::KeyA) {
        direction += Camera::right();
    }

    if input.key_held(KeyCode::Space) {
        direction += Camera::up();
    }
    if input.key_held(KeyCode::ShiftLeft) || input.key_held(KeyCode::ShiftRight) {
        direction -= Camera::up();
    }
    direction
}
