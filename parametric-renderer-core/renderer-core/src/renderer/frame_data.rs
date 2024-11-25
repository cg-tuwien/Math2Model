/// Values that change every frame*
#[derive(Clone)]
pub struct FrameData {
    pub camera: crate::camera::Camera,
    pub mouse_pos: glam::Vec2,
    pub mouse_held: bool,
    pub lod_stage: Option<std::sync::Arc<dyn Fn(&crate::game::ShaderId, &str) + 'static>>,
}

impl Default for FrameData {
    fn default() -> Self {
        Self {
            camera: crate::camera::Camera::new(crate::camera::CameraSettings::default()),
            mouse_pos: Default::default(),
            mouse_held: Default::default(),
            lod_stage: Default::default(),
        }
    }
}
