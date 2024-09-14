use std::sync::Arc;

use glam::{UVec2, Vec3};
use pollster::FutureExt;
use renderer_core::{
    camera::camera_controller::{self, CameraController, IsCameraController},
    game::{GameRes, MaterialInfo, ModelInfo, ShaderId, ShaderInfo},
    input::{InputHandler, WindowInputs, WinitAppHelper},
    renderer::{GpuApplication, GpuApplicationBuilder},
    transform::Transform,
    window_or_fallback::WindowOrFallback,
};
use tracing::{error, info, warn};
use winit::{application::ApplicationHandler, dpi::PhysicalSize, window::Window};

use crate::config::{CacheFile, CachedCamera, CachedChosenController};

pub struct Application {
    app: GameRes,
    window: Option<Arc<Window>>,
    renderer: Option<GpuApplication>,
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

        let mut app = GameRes::new();
        app.profiler_settings.gpu = true;
        let shader_id = ShaderId("HeartSphere.wgsl".into());
        app.set_shader(
            shader_id.clone(),
            ShaderInfo {
                label: "HeartSphere".into(),
                code: Application::DEFAULT_SHADER_CODE.into(),
            },
        );
        app.update_models(vec![ModelInfo {
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
                    position: Vec3::from(position),
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
            renderer: None,
        })
    }

    fn create_surface(&mut self, window: Arc<Window>) {
        let renderer = GpuApplicationBuilder::new(WindowOrFallback::Window(window.clone()))
            .block_on()
            .unwrap()
            .build(&self.app.camera)
            .unwrap();
        self.renderer = Some(renderer);
        window.request_redraw();
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if let Some(renderer) = &mut self.renderer {
            renderer.resize(UVec2::new(new_size.width, new_size.height));
        }
    }

    pub fn update(&mut self, inputs: &WindowInputs) {
        self.app.update(inputs);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        if let Some(renderer) = &mut self.renderer {
            renderer.render(&self.app)
        } else {
            Ok(())
        }
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
            match self.renderer.as_mut().and_then(|v| v.get_profiling_data()) {
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
                if let Some(gpu) = &mut self.renderer {
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
