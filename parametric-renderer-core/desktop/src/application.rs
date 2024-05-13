use glamour::{Angle, Point3};
use pollster::FutureExt;
use renderer_core::application::CpuApplication;
use tracing::{error, info, warn};
use winit::{application::ApplicationHandler, window::Window};
use winit_input_helper::{WinitInputApp, WinitInputHelper, WinitInputUpdate};

use crate::config::{CacheFile, CachedCamera, ConfigFile};

pub struct Application {
    app: CpuApplication,
    config_file: ConfigFile,
    cache_file: CacheFile,
}

impl Drop for Application {
    fn drop(&mut self) {
        self.cache_file.camera = Some(CachedCamera::FirstPerson {
            position: self.app.freecam_controller.position.to_array(),
            pitch: self.app.freecam_controller.pitch.radians,
            yaw: self.app.freecam_controller.yaw.radians,
        });
        self.cache_file
            .save_to_file(Application::CACHE_FILE)
            .unwrap();
    }
}

impl Application {
    const CONFIG_FILE: &'static str = "config.json";
    const CACHE_FILE: &'static str = "cache.json";
    pub fn new() -> anyhow::Result<Self> {
        let config_file = ConfigFile::from_file(Application::CONFIG_FILE).unwrap_or_default();
        let cache_file = CacheFile::from_file(Application::CACHE_FILE).unwrap_or_default();

        let mut app = CpuApplication::new()?;

        if let Some(CachedCamera::FirstPerson {
            position,
            pitch,
            yaw,
        }) = cache_file.camera
        {
            app.freecam_controller.position = Point3::from(position);
            app.freecam_controller.pitch = Angle::from(pitch);
            app.freecam_controller.yaw = Angle::from(yaw);
        }

        Ok(Self {
            app,
            config_file,
            cache_file,
        })
    }

    fn create_surface(&mut self, window: Window) {
        self.app.create_surface(window).block_on().unwrap();
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.app.resize(new_size);
    }

    pub fn update(&mut self, inputs: &WinitInputHelper) {
        self.app.update(inputs);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.app.render()
    }
}

pub async fn run() -> anyhow::Result<()> {
    let event_loop = winit::event_loop::EventLoop::new()?;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let application = Application::new()?;
    event_loop.run_app(&mut WinitInputApp::new(application))?;
    Ok(())
}

impl ApplicationHandler<()> for Application {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let _ = self.create_surface(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );
    }

    fn window_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match &event {
            winit::event::WindowEvent::RedrawRequested => {
                // Shouldn't need anything here
                match self.app.gpu {
                    Some(ref mut gpu) => gpu.request_redraw(),
                    None => {}
                }
            }
            _ => {}
        }
    }
}

impl WinitInputUpdate for Application {
    fn update(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        input: &WinitInputHelper,
    ) {
        use winit::keyboard::{Key, NamedKey};

        if input.key_released_logical(Key::Named(NamedKey::Escape))
            || input.close_requested()
            || input.destroyed()
        {
            info!("Stopping the application.");
            event_loop.exit();
            return;
        }
        self.app.delta_time = input.delta_time().unwrap_or_default().as_secs_f32();
        if let Some((width, height)) = input.resolution() {
            self.resize(winit::dpi::PhysicalSize { width, height });
        }
        self.update(input);
        match self.render() {
            Ok(_) => (),
            Err(wgpu::SurfaceError::Lost) => {
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
