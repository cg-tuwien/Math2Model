use glam::{UVec2, Vec3};
use renderer_core::{
    camera::camera_controller::{self, CameraController},
    game::{GameRes, ModelInfo, ShaderId, ShaderInfo, WindowOrFallback},
    input::{InputHandler, WindowInputs, WinitAppHelper},
};
use std::sync::{Arc, Mutex};
use tracing::{error, info, warn};
use wasm_bindgen::{prelude::wasm_bindgen, JsError};
use web_sys::HtmlCanvasElement;
use winit::{application::ApplicationHandler, dpi::PhysicalSize, window::Window};

use crate::wasm_abi::{
    WasmCompilationResult, WasmCompilationResults, WasmModelInfo, WasmShaderInfo,
};

#[wasm_bindgen]
pub struct WasmApplication {
    app: Arc<Mutex<GameRes>>,
}

#[wasm_bindgen]
impl WasmApplication {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<WasmApplication, JsError> {
        let mut app = GameRes::new(); //.map_err(|e| JsError::from(&*e))?;
        app.camera_controller = CameraController::new(
            camera_controller::GeneralController {
                position: Vec3::new(0.0, 0.0, 4.0),
                orientation: glam::Quat::IDENTITY,
                distance_to_center: 4.0,
            },
            app.camera_controller.settings,
            camera_controller::ChosenKind::Orbitcam,
        );
        Ok(Self {
            app: Arc::new(Mutex::new(app)),
        })
    }

    pub async fn run(&self, canvas: HtmlCanvasElement) -> Result<(), JsError> {
        let event_loop = winit::event_loop::EventLoop::builder().build()?;
        let application = Application::new(canvas, self.app.clone());
        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::EventLoopExtWebSys;
            event_loop.spawn_app(WinitAppHelper::new(application));
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            event_loop.run_app(&mut WinitAppHelper::new(application))?;
        }
        Ok(())
    }

    pub fn update_models(&self, js_models: Vec<WasmModelInfo>) {
        let models = js_models
            .into_iter()
            .map(|v| ModelInfo {
                transform: v.transform.into(),
                material_info: v.material_info.into(),
                shader_id: ShaderId(v.shader_id),
            })
            .collect();
        let mut app = self.app.lock().unwrap();
        app.update_models(models);
    }

    pub async fn update_shader(&self, shader_info: WasmShaderInfo) -> WasmCompilationResults {
        let shader_id = ShaderId(shader_info.id);
        let info = ShaderInfo {
            label: shader_info.label,
            code: shader_info.code,
        };
        {
            self.app.lock().unwrap().set_shader(shader_id.clone(), info);
        }

        // OWO Am I not holding a mutex lock for way too long here?
        let compilation_results =
            { self.app.lock().unwrap().get_compilation_messages(shader_id) }.await;
        let results = compilation_results
            .into_iter()
            .map(|(shader_id, messages)| WasmCompilationResult {
                shader_id: shader_id.0,
                messages: messages.into_iter().map(Into::into).collect(),
            })
            .collect();
        WasmCompilationResults { results }
    }

    pub fn remove_shader(&self, id: String) {
        self.app.lock().unwrap().remove_shader(&ShaderId(id));
    }
}

/// For Winit
pub struct Application {
    window: Option<Arc<Window>>,
    app: Arc<Mutex<GameRes>>,
    /** For wasm32 */
    _canvas: HtmlCanvasElement,
}

impl Application {
    pub fn new(canvas: HtmlCanvasElement, app: Arc<Mutex<GameRes>>) -> Self {
        Self {
            window: None,
            app,
            _canvas: canvas,
        }
    }

    fn create_surface(&mut self, window: Window) {
        let window = Arc::new(window);
        self.window = Some(window.clone());

        let gpu_builder = {
            self.app
                .lock()
                .unwrap()
                .start_create_gpu(WindowOrFallback::Window(window))
        };

        let app = self.app.clone();
        let task = async move {
            let gpu_application = gpu_builder.create_surface().await.unwrap();
            app.lock().unwrap().set_gpu(gpu_application);
        };
        wasm_bindgen_futures::spawn_local(task);
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.app
            .lock()
            .unwrap()
            .resize(UVec2::new(new_size.width, new_size.height));
    }

    pub fn update(&mut self, inputs: &WindowInputs) {
        self.app.lock().unwrap().update(inputs)
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.app.lock().unwrap().render()
    }
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

        let window = event_loop.create_window(window_attributes).unwrap();
        // TODO: Call event_loop.create_proxy() here (after https://github.com/rust-windowing/winit/issues/3741 is resolved in the next winit version)
        // And pass that to create_surface
        // Then, create_surface no longer needs the APP_COMMANDS thread_local!
        self.create_surface(window);
    }

    fn window_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match &event {
            winit::event::WindowEvent::RedrawRequested => match self.window {
                Some(ref mut window) => window.request_redraw(),
                None => {}
            },
            winit::event::WindowEvent::CursorMoved { .. } => match self.window {
                Some(ref mut window) => window.request_redraw(),
                None => {}
            },
            _ => {}
        }
    }
}

impl InputHandler for Application {
    fn update(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, input: WindowInputs<'_>) {
        // Don't react to "esc"
        if input.close_requested {
            info!("Stopping the application.");
            event_loop.exit();
            return;
        }

        if let Some(PhysicalSize { width, height }) = input.new_size {
            self.resize(winit::dpi::PhysicalSize { width, height });
        }
        self.update(&input);
        match self.render() {
            Ok(_) => (),
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                warn!("Lost or outdated surface");
                if let Some(gpu) = &mut self.app.lock().unwrap().gpu {
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
