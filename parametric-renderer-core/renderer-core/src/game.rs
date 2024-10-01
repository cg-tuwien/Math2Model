use std::{collections::HashMap, sync::Arc};

use glam::{Vec2, Vec3};
use web_time::Instant;

use crate::{
    camera::{
        camera_controller::{
            CameraController, ChosenKind, GeneralController, GeneralControllerSettings,
        },
        Camera, CameraSettings,
    },
    input::WindowInputs,
    renderer::{frame_counter::Seconds, CursorCapture, WindowCursorCapture},
    transform::Transform,
};

#[derive(Debug, Clone, Default)]
pub struct ProfilerSettings {
    pub gpu: bool,
}

#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub transform: Transform,
    pub material_info: MaterialInfo,
    pub shader_id: ShaderId,
}

#[derive(Debug, Clone)]
pub struct MaterialInfo {
    pub color: Vec3,
    pub emissive: Vec3,
    pub roughness: f32,
    pub metallic: f32,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ShaderId(pub String);

#[derive(Clone)]
pub struct ShaderInfo {
    pub label: String,
    pub code: String,
}

pub struct GameRes {
    pub camera_controller: CameraController,
    pub models: Vec<ModelInfo>,
    pub shaders: HashMap<ShaderId, ShaderInfo>,
    last_update_instant: Option<Instant>,
    pub camera: Camera,
    pub mouse: Vec2,
    pub mouse_held: bool,
    pub cursor_capture: WindowCursorCapture,
    pub profiler_settings: ProfilerSettings,
    pub lod_stage: Option<Arc<dyn Fn(&ShaderId, u32) + 'static>>,
}

impl GameRes {
    pub fn new() -> Self {
        let camera = Camera::new(CameraSettings::default());
        let camera_controller = CameraController::new(
            GeneralController {
                position: Vec3::new(0.0, 0.0, 4.0),
                orientation: glam::Quat::IDENTITY,
                distance_to_center: 4.0,
            },
            GeneralControllerSettings {
                fly_speed: 5.0,
                pan_speed: 1.0,
                rotation_sensitivity: 0.01,
            },
            ChosenKind::Freecam,
        );

        Self {
            camera,
            camera_controller,
            models: vec![],
            shaders: HashMap::new(),
            last_update_instant: None,
            mouse: Vec2::ZERO,
            mouse_held: false,
            cursor_capture: WindowCursorCapture::Free,
            profiler_settings: ProfilerSettings::default(),
            lod_stage: None,
        }
    }

    pub fn update_models(&mut self, models: Vec<ModelInfo>) {
        self.models = models;
    }

    pub fn set_shader(&mut self, shader_id: ShaderId, info: ShaderInfo) {
        self.shaders.insert(shader_id.clone(), info);
    }

    pub fn remove_shader(&mut self, shader_id: &ShaderId) {
        self.shaders.remove(shader_id);
    }

    pub fn update(&mut self, inputs: &WindowInputs) {
        let now = Instant::now();
        if let Some(last_update_instant) = self.last_update_instant {
            let delta = Seconds((now - last_update_instant).as_secs_f32());
            self.cursor_capture = match self.camera_controller.update(inputs, delta.0) {
                CursorCapture::Free => WindowCursorCapture::Free,
                CursorCapture::LockedAndHidden => {
                    WindowCursorCapture::LockedAndHidden(inputs.mouse.position)
                }
            };
        }
        self.last_update_instant = Some(now);
        self.camera.update_camera(&self.camera_controller);
        self.mouse = Vec2::new(
            inputs.mouse.position.x as f32,
            inputs.mouse.position.y as f32,
        );
        self.mouse_held = inputs.mouse.pressed(winit::event::MouseButton::Left);
    }
}
