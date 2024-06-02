use glam::Quat;
use glamour::{Angle, Point3, Vector2, Vector3};
use winit::{event::MouseButton, keyboard::KeyCode};

use crate::{application::CursorCapture, input::WindowInputs};

use super::{
    camera_controller::{GeneralController, GeneralControllerSettings, IsCameraController},
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
        input: &WindowInputs,
        delta_time: f32,
        settings: &GeneralControllerSettings,
    ) -> CursorCapture {
        let mut cursor_capture = CursorCapture::Free;
        let mouse_delta = Vector2::new(input.mouse.motion.0 as f32, input.mouse.motion.1 as f32);
        if input.mouse.pressed(MouseButton::Right) {
            self.update_orientation(mouse_delta, settings);
            cursor_capture = CursorCapture::LockedAndHidden;
        }

        self.update_position(input_to_direction(input), delta_time, settings);
        if input.mouse.pressed(MouseButton::Middle) {
            self.update_pan_position(mouse_delta, delta_time, settings);
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

// Magic number.
const FREECAM_DISTANCE_TO_CENTER: f32 = 15.;

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

    fn general_controller(&self) -> GeneralController {
        GeneralController {
            position: self.position(),
            orientation: self.orientation(),
            distance_to_center: FREECAM_DISTANCE_TO_CENTER,
        }
    }
}

fn input_to_direction(input: &WindowInputs) -> Vector3 {
    let mut direction = Vector3::ZERO;
    if input.keyboard.pressed_physical(KeyCode::KeyW) {
        direction += Camera::forward();
    }
    if input.keyboard.pressed_physical(KeyCode::KeyS) {
        direction -= Camera::forward();
    }

    if input.keyboard.pressed_physical(KeyCode::KeyD) {
        direction += Camera::right();
    }
    if input.keyboard.pressed_physical(KeyCode::KeyA) {
        direction -= Camera::right();
    }

    if input.keyboard.pressed_physical(KeyCode::Space) {
        direction += Camera::up();
    }
    if input.keyboard.pressed_physical(KeyCode::ShiftLeft)
        || input.keyboard.pressed_physical(KeyCode::ShiftRight)
    {
        direction -= Camera::up();
    }
    direction
}
