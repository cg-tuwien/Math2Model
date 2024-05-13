use glamour::Angle;

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
