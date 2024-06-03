use glamour::{Matrix4, Point3, Vector3};

#[derive(Debug, Clone)]
pub struct Transform {
    pub position: Point3,
    pub rotation: glam::Quat,
    pub scale: f32,
}

impl Transform {
    pub fn to_matrix(&self) -> Matrix4<f32> {
        Matrix4::from_scale_rotation_translation(
            Vector3::new(self.scale, self.scale, self.scale),
            self.rotation,
            self.position.to_vector(),
        )
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Point3::new(0.0, 0.0, 0.0),
            rotation: glam::Quat::IDENTITY,
            scale: 1.0,
        }
    }
}
