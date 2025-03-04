use glam::{Quat, Vec2, Vec3};
use winit::event::MouseButton;

use crate::{input::WindowInputs, renderer::CursorCapture};

use super::{
    angle::Angle,
    camera_controller::{GeneralController, GeneralControllerSettings, IsCameraController},
    Camera,
};

struct LogarithmicDistance(f32);
impl LogarithmicDistance {
    fn new(distance: f32) -> Self {
        Self(distance.ln())
    }

    fn distance(&self) -> f32 {
        self.0.exp()
    }
}

pub struct OrbitcamController {
    pub center: Vec3,
    pub pitch: Angle,
    pub yaw: Angle,
    logarithmic_distance: LogarithmicDistance,
}

impl OrbitcamController {
    pub fn new(controller: GeneralController) -> Self {
        let center = controller.position
            + controller.orientation * (Camera::forward() * controller.distance_to_center);

        let (yaw, pitch, _) = controller.orientation.to_euler(glam::EulerRot::YXZ);

        Self {
            center,
            pitch: Angle::new(pitch),
            yaw: Angle::new(yaw),
            logarithmic_distance: LogarithmicDistance::new(controller.distance_to_center),
        }
    }
    pub fn update(
        &mut self,
        input: &WindowInputs,
        delta_time: f32,
        settings: &GeneralControllerSettings,
    ) -> CursorCapture {
        let mut cursor_capture = CursorCapture::Free;
        let mouse_delta = Vec2::new(input.mouse.motion.0 as f32, input.mouse.motion.1 as f32);
        if input.mouse.pressed(MouseButton::Right) {
            self.update_orientation(mouse_delta, settings);
            cursor_capture = CursorCapture::LockedAndHidden;
        }

        if input.mouse.pressed(MouseButton::Left) {
            self.update_pan_position(mouse_delta, delta_time, settings);
            cursor_capture = CursorCapture::LockedAndHidden;
        }

        self.logarithmic_distance.0 +=
            -1.0 * (input.mouse.scroll_delta.y as f32) * 0.1 * delta_time;

        cursor_capture
    }

    fn update_orientation(&mut self, mouse_delta: Vec2, settings: &GeneralControllerSettings) {
        self.set_pitch_yaw(
            self.pitch - Angle::new(mouse_delta.y * settings.rotation_sensitivity),
            self.yaw - Angle::new(mouse_delta.x * settings.rotation_sensitivity),
        );
    }

    fn set_pitch_yaw(&mut self, new_pitch: Angle, new_yaw: Angle) {
        const TWO_PI: f32 = std::f32::consts::PI * 2.0;
        let max_pitch = 88f32;
        self.pitch = new_pitch
            .min(Angle::from_degrees(0.))
            .max(Angle::from_degrees(-max_pitch));
        self.yaw = Angle::new(new_yaw.radians.rem_euclid(TWO_PI));
    }

    fn update_pan_position(
        &mut self,
        direction: Vec2,
        delta_time: f32,
        settings: &GeneralControllerSettings,
    ) {
        let horizontal_movement = self.orientation() * (Camera::right() * direction.x * -1.0);
        let vertical_movement = self.orientation() * (Camera::up() * direction.y * 1.0);
        self.center += horizontal_movement * settings.pan_speed * delta_time;
        self.center += vertical_movement * settings.pan_speed * delta_time;
    }
}

impl IsCameraController for OrbitcamController {
    fn position(&self) -> Vec3 {
        self.center + self.orientation() * Vec3::new(0.0, 0.0, self.logarithmic_distance.distance())
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
            distance_to_center: self.logarithmic_distance.distance(),
        }
    }
}
