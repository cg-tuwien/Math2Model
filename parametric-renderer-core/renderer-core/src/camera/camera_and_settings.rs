use glam::{Mat4, Quat, UVec2, Vec3};

use super::{angle::Angle, camera_controller::IsCameraController};

#[derive(Debug, Clone)]
pub struct CameraSettings {
    pub z_near: f32,
    pub z_far: f32,
    pub fov: Angle,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            z_near: 0.1,
            z_far: 100.0,
            fov: Angle::from_degrees(60.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Camera {
    pub position: Vec3,
    pub orientation: Quat,
    pub settings: CameraSettings,

    size: UVec2,
    view: Mat4,
    proj: Mat4,
}

impl Camera {
    pub fn new(size: UVec2, settings: CameraSettings) -> Self {
        let position = Vec3::ZERO;
        let orientation = Quat::IDENTITY;

        let aspect_ratio = size.x as f32 / size.y as f32;
        let proj = Mat4::perspective_infinite_reverse_rh(
            settings.fov.radians,
            aspect_ratio,
            settings.z_near,
        );

        let view = calculate_view(position, orientation);

        Self {
            size,
            position,
            orientation,
            settings,
            proj,
            view,
        }
    }

    pub fn size(&self) -> UVec2 {
        self.size
    }

    /// Positions the camera
    pub fn view_matrix(&self) -> Mat4 {
        self.view
    }

    pub fn projection_matrix(&self) -> Mat4 {
        self.proj
    }

    pub fn update_camera(&mut self, controller: &impl IsCameraController) {
        self.position = controller.position();
        self.orientation = controller.orientation();

        self.view = calculate_view(self.position, self.orientation);
    }

    pub fn update_size(&mut self, size: UVec2) {
        let aspect_ratio = size.x as f32 / size.y as f32;
        // See https://docs.rs/glam/0.27.0/src/glam/f32/sse2/mat4.rs.html#969-982
        self.proj.as_mut()[0] = self.proj.as_mut()[5] / aspect_ratio;
        self.size = size;
    }

    /// in world-space
    pub const fn forward() -> Vec3 {
        Vec3::new(0.0, 0.0, -1.0)
    }

    /// in world-space
    pub const fn right() -> Vec3 {
        Vec3::new(1.0, 0.0, 0.0)
    }

    /// in world-space
    pub const fn up() -> Vec3 {
        Vec3::new(0.0, 1.0, 0.0)
    }
}

fn calculate_view(position: Vec3, orientation: Quat) -> Mat4 {
    let cam_direction = orientation * Camera::forward();
    let target = position + cam_direction;

    Mat4::look_at_rh(position, target, Camera::up())
}
