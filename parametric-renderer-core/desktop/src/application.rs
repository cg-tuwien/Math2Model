use glam::Vec3;
use renderer_core::{
    application::{AppCommand, Application, WasmCanvas},
    camera::camera_controller::{self, CameraController, IsCameraController},
    game::{MaterialInfo, ModelInfo, ShaderId, ShaderInfo},
    input::WinitAppHelper,
    transform::Transform,
};
use winit::event_loop::EventLoop;

use crate::config::{CacheFile, CachedCamera, CachedChosenController};

const CACHE_FILE: &'static str = "cache.json";
const HEART_SPHERE_SHADER_CODE: &'static str = include_str!("../../shaders/HeartSphere.wgsl");

fn save_cache(mut cache_file: CacheFile) -> impl FnOnce(&mut Application) {
    move |app: &mut Application| {
        let controller = app.app.camera_controller.general_controller();
        cache_file.camera = Some(CachedCamera {
            position: controller.position.to_array(),
            orientation: controller.orientation.to_array(),
            distance_to_center: controller.distance_to_center,
            chosen: match app.app.camera_controller.get_chosen_kind() {
                camera_controller::ChosenKind::Orbitcam => CachedChosenController::Orbitcam,
                camera_controller::ChosenKind::Freecam => CachedChosenController::Freecam,
            },
        });
        cache_file.save_to_file(CACHE_FILE).unwrap();
    }
}

pub fn run() -> anyhow::Result<()> {
    let event_loop = EventLoop::<AppCommand>::with_user_event().build()?;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let event_loop_proxy = event_loop.create_proxy();
    let cache_file = CacheFile::from_file(CACHE_FILE).unwrap_or_default();
    let cached_camera = cache_file.camera.clone();
    let mut application =
        Application::new(event_loop_proxy, save_cache(cache_file), WasmCanvas::new());

    application.app.profiler_settings.gpu = true;
    let shader_id = ShaderId("HeartSphere.wgsl".into());
    application.app.set_shader(
        shader_id.clone(),
        ShaderInfo {
            label: "HeartSphere".into(),
            code: HEART_SPHERE_SHADER_CODE.into(),
        },
    );
    application.app.update_models(vec![ModelInfo {
        id: "0659dcb1-6229-46bd-a306-6ceebfcf2e46".into(),
        transform: Transform {
            position: Vec3::new(0.0, 0.0, 0.0),
            ..Default::default()
        },
        material_info: MaterialInfo {
            color: Vec3::new(0.6, 1.0, 1.0),
            emissive: Vec3::new(0.0, 0.0, 0.0),
            roughness: 0.7,
            metallic: 0.1,
        },
        shader_id,
        instance_count: 5,
    }]);

    if let Some(CachedCamera {
        position,
        orientation,
        distance_to_center,
        chosen,
    }) = cached_camera
    {
        application.app.camera_controller = CameraController::new(
            camera_controller::GeneralController {
                position: Vec3::from(position),
                orientation: glam::Quat::from_array(orientation),
                distance_to_center,
            },
            application.app.camera_controller.settings,
            match chosen {
                CachedChosenController::Orbitcam => camera_controller::ChosenKind::Orbitcam,
                CachedChosenController::Freecam => camera_controller::ChosenKind::Freecam,
            },
        );
    }

    event_loop.run_app(&mut WinitAppHelper::new(application))?;
    Ok(())
}
