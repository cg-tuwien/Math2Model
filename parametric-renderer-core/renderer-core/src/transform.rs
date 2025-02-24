use glam::{Mat4, Vec3};

#[derive(Debug, Clone, PartialEq)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: glam::Quat,
    pub scale: f32,
}

impl Transform {
    pub fn to_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(
            Vec3::new(self.scale, self.scale, self.scale),
            self.rotation,
            self.position,
        )
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 0.0),
            rotation: glam::Quat::IDENTITY,
            scale: 1.0,
        }
    }
}
