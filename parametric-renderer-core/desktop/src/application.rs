use std::sync::Arc;

use glamour::{Point3, Vector3};
use pollster::FutureExt;
use renderer_core::{
    application::{CpuApplication, MaterialInfo, ModelInfo, ProfilerSettings},
    camera::camera_controller::{self, CameraController, IsCameraController},
    input::{InputHandler, WindowInputs, WinitAppHelper},
    transform::Transform,
};
use tracing::{error, info, warn};
use winit::{application::ApplicationHandler, dpi::PhysicalSize, window::Window};

use crate::config::{CacheFile, CachedCamera, CachedChosenController};

pub struct Application {
    app: CpuApplication,
    window: Option<Arc<Window>>,
    cache_file: CacheFile,
}

impl Drop for Application {
    fn drop(&mut self) {
        let controller = self.app.camera_controller.general_controller();
        self.cache_file.camera = Some(CachedCamera {
            position: controller.position.to_array(),
            orientation: controller.orientation.to_array(),
            distance_to_center: controller.distance_to_center,
            chosen: match self.app.camera_controller.get_chosen_kind() {
                camera_controller::ChosenKind::Orbitcam => CachedChosenController::Orbitcam,
                camera_controller::ChosenKind::Freecam => CachedChosenController::Freecam,
            },
        });
        self.cache_file
            .save_to_file(Application::CACHE_FILE)
            .unwrap();
    }
}

impl Application {
    const CACHE_FILE: &'static str = "cache.json";
    const DEFAULT_SHADER_CODE: &'static str = include_str!("../../shaders/HeartSphere.wgsl");
    pub fn new() -> anyhow::Result<Self> {
        let cache_file = CacheFile::from_file(Application::CACHE_FILE).unwrap_or_default();

        let mut app = CpuApplication::new()?;
        app.set_profiling(ProfilerSettings { gpu: true });
        app.update_models(vec![ModelInfo {
            label: "Default Model".to_owned(),
            transform: Transform {
                position: Point3::new(0.0, 1.0, 0.0),
                ..Default::default()
            },
            material_info: MaterialInfo {
                color: Vector3::new(0.6, 1.0, 1.0),
                emissive: Vector3::new(0.0, 0.0, 0.0),
                roughness: 0.7,
                metallic: 0.1,
            },
            evaluate_image_code: Application::DEFAULT_SHADER_CODE.to_owned(),
        }]);

        if let Some(CachedCamera {
            position,
            orientation,
            distance_to_center,
            chosen,
        }) = cache_file.camera.clone()
        {
            app.camera_controller = CameraController::new(
                camera_controller::GeneralController {
                    position: Point3::from(position),
                    orientation: glam::Quat::from_array(orientation),
                    distance_to_center,
                },
                app.camera_controller.settings,
                match chosen {
                    CachedChosenController::Orbitcam => camera_controller::ChosenKind::Orbitcam,
                    CachedChosenController::Freecam => camera_controller::ChosenKind::Freecam,
                },
            );
        }

        Ok(Self {
            window: None,
            app,
            cache_file,
        })
    }

    fn create_surface(&mut self, window: Arc<Window>) {
        self.app.create_surface(window).block_on().unwrap();
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.app.resize(new_size);
    }

    pub fn update(&mut self, inputs: &WindowInputs) {
        self.app.update(inputs);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.app.render()
    }
}

pub async fn run() -> anyhow::Result<()> {
    let event_loop = winit::event_loop::EventLoop::new()?;
    let application = Application::new()?;
    event_loop.run_app(&mut WinitAppHelper::new(application))?;
    Ok(())
}

impl ApplicationHandler<()> for Application {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
            return;
        }

        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );
        self.window = Some(window.clone());
        self.create_surface(window);
    }

    fn window_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match &event {
            winit::event::WindowEvent::RedrawRequested => {
                if let Some(window) = &self.window {
                    window.request_redraw();
                    return;
                }
            }
            _ => {}
        }
    }
}

impl InputHandler for Application {
    fn update(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, input: WindowInputs<'_>) {
        use winit::keyboard::{Key, NamedKey};
        if input
            .keyboard
            .just_released_logical(Key::Named(NamedKey::Escape))
            || input.close_requested
        {
            info!("Stopping the application.");
            event_loop.exit();
            return;
        }
        // Press P to print profiling data
        if input
            .keyboard
            .just_pressed_physical(winit::keyboard::KeyCode::KeyP)
        {
            match self.app.get_profiling_data() {
                Some(data) => {
                    let file_name = format!(
                        "profile-{}.json",
                        // use the current time as a unique-enugh identifier
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_millis()
                    );
                    wgpu_profiler::chrometrace::write_chrometrace(
                        std::path::Path::new(&file_name),
                        &data,
                    )
                    .unwrap();
                    info!("Profiling data written to {file_name}");
                }
                None => {
                    warn!("Profiling data not available");
                }
            }
        }

        if let Some(PhysicalSize { width, height }) = input.new_size {
            self.resize(winit::dpi::PhysicalSize { width, height });
        }
        self.update(&input);
        match self.render() {
            Ok(_) => (),
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                // TODO: Window closing and opening is borked
                if let Some(gpu) = &mut self.app.gpu {
                    let _ = gpu.resize(gpu.size());
                }
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                error!("Out of memory");
                event_loop.exit();
            }
            Err(e) => {
                warn!("Unexpected error: {:?}", e);
            }
        }
    }
}
