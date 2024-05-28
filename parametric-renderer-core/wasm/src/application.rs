use std::sync::{Arc, Mutex};

use renderer_core::application::CpuApplication;
use tracing::{error, info, warn};
use web_sys::HtmlCanvasElement;
use winit::{application::ApplicationHandler, window::Window};
use winit_input_helper::{WinitInputApp, WinitInputHelper, WinitInputUpdate};

pub struct Application {
    window: Option<Arc<Window>>,
    app: Arc<Mutex<CpuApplication>>,
    /** For wasm32 */
    _canvas: HtmlCanvasElement,
}

impl Application {
    pub fn new(canvas: HtmlCanvasElement) -> anyhow::Result<Self> {
        let app = CpuApplication::new()?;
        Ok(Self {
            window: None,
            app: Arc::new(Mutex::new(app)),
            _canvas: canvas,
        })
    }

    fn create_surface(&mut self, window: Arc<Window>) {
        let app_mutex = self.app.clone();
        let task = async move {
            let mut app = match app_mutex.try_lock() {
                Ok(app) => app,
                Err(_) => {
                    warn!("Failed to lock the application");
                    return;
                }
            };

            // TODO: Fix this
            // warning: this `MutexGuard` is held across an `await` point
            // https://docs.rs/async-lock/latest/async_lock/
            app.create_surface(window).await.unwrap();
        };
        wasm_bindgen_futures::spawn_local(task);
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        let mut app = match self.app.try_lock() {
            Ok(app) => app,
            Err(_) => {
                warn!("Failed to lock the application for resize");
                return;
            }
        };
        app.resize(new_size);
    }

    pub fn update(&mut self, inputs: &WinitInputHelper) {
        let mut app = match self.app.try_lock() {
            Ok(app) => app,
            Err(_) => {
                return;
            }
        };
        app.update(inputs)
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let mut app = match self.app.try_lock() {
            Ok(app) => app,
            Err(_) => {
                return Ok(());
            }
        };
        app.render()
    }
}

pub async fn run(canvas: HtmlCanvasElement) -> anyhow::Result<()> {
    let event_loop = winit::event_loop::EventLoop::new()?;
    let application = Application::new(canvas)?;
    event_loop.run_app(&mut WinitInputApp::new(application))?;
    Ok(())
}

impl ApplicationHandler<()> for Application {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
            return;
        }

        let window_attributes = Window::default_attributes();
        #[cfg(target_arch = "wasm32")]
        let window_attributes = {
            use winit::platform::web::WindowAttributesExtWebSys;
            window_attributes.with_canvas(Some(self._canvas.clone()))
        };

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
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
                let mut app = match self.app.try_lock() {
                    Ok(app) => app,
                    Err(_) => {
                        return;
                    }
                };

                match app.gpu {
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
        // Don't react to "esc"
        if input.close_requested() || input.destroyed() {
            info!("Stopping the application.");
            event_loop.exit();
            return;
        }

        {
            let mut app = match self.app.try_lock() {
                Ok(app) => app,
                Err(_) => {
                    return;
                }
            };
            app.delta_time = input.delta_time().unwrap_or_default().as_secs_f32();
        }
        if let Some((width, height)) = input.resolution() {
            self.resize(winit::dpi::PhysicalSize { width, height });
        }
        self.update(input);
        match self.render() {
            Ok(_) => (),
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                let mut app = match self.app.try_lock() {
                    Ok(app) => app,
                    Err(_) => {
                        return;
                    }
                };
                if let Some(gpu) = &mut app.gpu {
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
