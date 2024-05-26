use glam::Quat;
use glamour::{Angle, Point3, Vector2, Vector3};
use winit::{event::MouseButton, keyboard::KeyCode};
use winit_input_helper::WinitInputHelper;

use super::{
    camera_controller::{
        CursorCapture, GeneralController, GeneralControllerSettings, IsCameraController,
    },
    Camera,
};

pub struct FreecamController {
    pub position: Point3,
    pub pitch: Angle,
    pub yaw: Angle,
}

impl FreecamController {
    pub fn new(controller: GeneralController) -> Self {
        let (yaw, pitch, _) = controller.orientation.to_euler(glam::EulerRot::YXZ);

        Self {
            position: controller.position,
            pitch: Angle::from(pitch),
            yaw: Angle::from(yaw),
        }
    }
    pub fn update(
        &mut self,
        input: &WinitInputHelper,
        delta_time: f32,
        settings: &GeneralControllerSettings,
    ) -> CursorCapture {
        let mut cursor_capture = CursorCapture::Free;
        if input.mouse_held(MouseButton::Right) {
            self.update_orientation(Vector2::from(input.mouse_diff()), settings);
            cursor_capture = CursorCapture::LockedAndHidden;
        }

        self.update_position(input_to_direction(input), delta_time, settings);
        if input.mouse_held(MouseButton::Middle) {
            self.update_pan_position(Vector2::from(input.mouse_diff()), delta_time, settings);
            cursor_capture = CursorCapture::LockedAndHidden;
        }
        cursor_capture
    }

    fn update_orientation(&mut self, mouse_delta: Vector2, settings: &GeneralControllerSettings) {
        self.set_pitch_yaw(
            self.pitch - Angle::new(mouse_delta.y * settings.rotation_sensitivity),
            self.yaw - Angle::new(mouse_delta.x * settings.rotation_sensitivity),
        );
    }

    fn update_pan_position(
        &mut self,
        direction: Vector2,
        delta_time: f32,
        settings: &GeneralControllerSettings,
    ) {
        let horizontal_movement = self.orientation() * (Camera::right() * direction.x * 1.0);
        let vertical_movement = self.orientation() * (Camera::up() * direction.y * -1.0);
        self.position += horizontal_movement * settings.pan_speed * delta_time;
        self.position += vertical_movement * settings.pan_speed * delta_time;
    }

    fn set_pitch_yaw(&mut self, new_pitch: Angle, new_yaw: Angle) {
        const TWO_PI: f32 = std::f32::consts::PI * 2.0;
        let max_pitch = 88f32;
        self.pitch = new_pitch
            .min(Angle::from_degrees(max_pitch))
            .max(Angle::from_degrees(-max_pitch));
        self.yaw = Angle::new(new_yaw.radians.rem_euclid(TWO_PI));
    }

    fn update_position(
        &mut self,
        direction: Vector3,
        delta_time: f32,
        settings: &GeneralControllerSettings,
    ) {
        let horizontal_movement = (direction * Vector3::new(1.0, 0.0, 1.0)).normalize_or_zero();
        let vertical_movement = Camera::up() * direction.y;
        let horizontal_movement = Quat::from_rotation_y(self.yaw.radians) * horizontal_movement;

        self.position += horizontal_movement * settings.fly_speed * delta_time;
        self.position += vertical_movement * settings.fly_speed * delta_time;
    }
}

impl IsCameraController for FreecamController {
    fn position(&self) -> Point3 {
        self.position
    }

    fn orientation(&self) -> Quat {
        Quat::from_euler(
            glam::EulerRot::YXZ,
            self.yaw.radians,
            self.pitch.radians,
            0.0,
        )
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
        direction += Camera::right();
    }
    if input.key_held(KeyCode::KeyA) {
        direction -= Camera::right();
    }

    if input.key_held(KeyCode::Space) {
        direction += Camera::up();
    }
    if input.key_held(KeyCode::ShiftLeft) || input.key_held(KeyCode::ShiftRight) {
        direction -= Camera::up();
    }
    direction
}
