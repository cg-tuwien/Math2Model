use glam::Quat;
use glamour::{Angle, Matrix4, Point3, Vector3};

use super::{camera_controller::CameraController, camera_settings::CameraSettings};

#[derive(Debug)]
pub struct Camera {
    pub position: Point3,
    pub orientation: Quat,
    pub settings: CameraSettings,

    view: Matrix4<f32>,
    proj: Matrix4<f32>,
}

impl Camera {
    pub fn new(aspect_ratio: f32, settings: CameraSettings) -> Self {
        let position = Point3::ZERO;
        let orientation = Quat::IDENTITY;

        let proj =
            calculate_projection(aspect_ratio, settings.fov, settings.z_near, settings.z_far);

        let view = calculate_view(position, orientation);

        Self {
            position,
            orientation,
            settings,
            proj,
            view,
        }
    }

    /// Positions the camera
    pub fn view_matrix(&self) -> Matrix4<f32> {
        self.view
    }

    pub fn projection_matrix(&self) -> Matrix4<f32> {
        self.proj
    }

    pub fn update_camera(&mut self, controller: &impl CameraController) {
        self.position = controller.position();
        self.orientation = controller.orientation();

        self.view = calculate_view(self.position, self.orientation);
    }

    pub fn update_aspect_ratio(&mut self, aspect_ratio: f32) {
        // See https://docs.rs/glam/0.27.0/src/glam/f32/sse2/mat4.rs.html#969-982
        self.proj.as_cols_mut()[0][0] = -self.proj.as_cols()[1][1] / aspect_ratio;
    }

    /// in world-space
    pub const fn forward() -> Vector3 {
        Vector3::new(0.0, 0.0, -1.0)
    }

    /// in world-space
    pub const fn right() -> Vector3 {
        Vector3::new(1.0, 0.0, 0.0)
    }

    /// in world-space
    pub const fn up() -> Vector3 {
        Vector3::new(0.0, 1.0, 0.0)
    }
}

/// fov is expected to be in radians
fn calculate_projection(aspect_ratio: f32, fov: Angle, near: f32, _far: f32) -> Matrix4<f32> {
    Matrix4::perspective_infinite_reverse_rh(fov, aspect_ratio, near)
}

fn calculate_view(position: Point3, orientation: Quat) -> Matrix4<f32> {
    let cam_direction = orientation * Camera::forward();
    let target = position + cam_direction;

    Matrix4::look_at_rh(position, target, Camera::up())
}
