use glam::{UVec2, Vec3};
use pinky_swear::PinkySwear;
use renderer_core::{
    camera::camera_controller::{self, CameraController},
    game::{GameRes, ModelInfo, ShaderId, ShaderInfo},
    input::{InputHandler, WindowInputs, WinitAppHelper},
    renderer::{GpuApplication, GpuApplicationBuilder},
    window_or_fallback::WindowOrFallback,
};
use std::sync::{Arc, Mutex};
use tracing::{error, info, warn};
use wasm_bindgen::{prelude::wasm_bindgen, JsError, JsValue};
use web_sys::HtmlCanvasElement;
use winit::{
    application::ApplicationHandler, dpi::PhysicalSize, event_loop::EventLoopProxy, window::Window,
};

use crate::wasm_abi::{WasmCompilationResults, WasmModelInfo, WasmShaderInfo};

#[wasm_bindgen]
pub struct WasmApplication {
    // Nothing to see here
}

thread_local! {
    static APP_COMMANDS: Mutex<Option<EventLoopProxy<AppCommand>>> = Mutex::new(None);
}

pub enum AppCommand {
    RunCallback(Box<dyn FnOnce(&mut Application)>),
}

/// Run a function on the main thread and awaits its result.
///
/// Such a callback may return a 'static Future. If that happens, we just spawn the future with wasm-bindgen-futures.
/// Not a part of the WasmApplication, because we want to be able to call this without the lifetime constraint of the WasmApplication.
#[must_use]
async fn run_on_main<Callback, T>(callback: Callback) -> T
where
    Callback: (FnOnce(&mut Application) -> T) + 'static,
    T: Send + 'static,
{
    let (promise, pinky) = PinkySwear::<T>::new();
    APP_COMMANDS.with(move |commands| {
        let guard = commands.lock().unwrap();
        let proxy = guard.as_ref().expect("No event loop proxy");
        let callback = move |app: &mut Application| {
            let return_value = callback(app);
            pinky.swear(return_value);
        };
        proxy
            .send_event(AppCommand::RunCallback(Box::new(callback)))
            .map_err(|_| ())
            .expect("Failed to send event, event loop not running?");
    });
    promise.await
}

#[wasm_bindgen]
impl WasmApplication {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<WasmApplication, JsError> {
        Ok(Self {})
    }

    pub fn run(&self, canvas: HtmlCanvasElement) -> Result<(), JsError> {
        let event_loop = winit::event_loop::EventLoop::<AppCommand>::with_user_event().build()?;
        APP_COMMANDS.with(|commands| {
            *commands.lock().unwrap() = Some(event_loop.create_proxy());
        });
        let application = Application::new(canvas);
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

    pub async fn update_models(&self, js_models: Vec<WasmModelInfo>) {
        let models = js_models
            .into_iter()
            .map(|v| ModelInfo {
                transform: v.transform.into(),
                material_info: v.material_info.into(),
                shader_id: ShaderId(v.shader_id),
            })
            .collect();
        let _ = run_on_main(|app| app.app.update_models(models)).await;
    }

    pub async fn update_shader(&self, shader_info: WasmShaderInfo) -> WasmCompilationResults {
        let shader_id = ShaderId(shader_info.id);
        let info = ShaderInfo {
            label: shader_info.label,
            code: shader_info.code,
        };

        let _ = run_on_main({
            let shader_id = shader_id.clone();
            move |app| app.app.set_shader(shader_id, info)
        })
        .await;

        let results = vec![];
        // OWO Am I not holding a mutex lock for way too long here?
        /*let compilation_results = run_on_main(|app| app.renderer);
            { self.app.lock().unwrap().get_compilation_messages(shader_id) }.await;
        let results = compilation_results
            .into_iter()
            .map(|(shader_id, messages)| WasmCompilationResult {
                shader_id: shader_id.0,
                messages: messages.into_iter().map(Into::into).collect(),
            })
            .collect();*/
        WasmCompilationResults { results }
    }

    pub async fn remove_shader(&self, id: String) {
        let _ = run_on_main(|app| app.app.remove_shader(&ShaderId(id))).await;
    }

    pub async fn set_lod_stage(&self, stage: Option<web_sys::js_sys::Function>) {
        let wrapped = stage.map(|stage| -> Box<dyn Fn(&ShaderId, u32) + 'static> {
            Box::new(move |shader_id: &ShaderId, buffer_id: u32| {
                let this = wasm_bindgen::JsValue::NULL;
                match stage.call2(
                    &this,
                    &JsValue::from_str(&shader_id.0),
                    &JsValue::from_f64(buffer_id as f64),
                ) {
                    Ok(_) => (),
                    Err(e) => error!("Error calling LOD stage: {:?}", e),
                }
            })
        });

        let _ = run_on_main(move |app| app.app.lod_stage = wrapped).await;
    }
}

/// For Winit
pub struct Application {
    app: GameRes,
    window: Option<Arc<Window>>,
    renderer: Option<GpuApplication>,
    /** For wasm32 */
    _canvas: HtmlCanvasElement,
}

impl Application {
    pub fn new(canvas: HtmlCanvasElement) -> Self {
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
        Self {
            window: None,
            app,
            renderer: None,
            _canvas: canvas,
        }
    }

    fn create_surface(&mut self, window: Window) {
        let window = Arc::new(window);
        self.window = Some(window.clone());

        let gpu_builder = GpuApplicationBuilder::new(WindowOrFallback::Window(window));

        let task = async move {
            let renderer = gpu_builder.await.unwrap().build();
            let _ = run_on_main(|app| app.renderer = Some(renderer)).await;
        };
        wasm_bindgen_futures::spawn_local(task);
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

impl ApplicationHandler<AppCommand> for Application {
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

    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, event: AppCommand) {
        match event {
            AppCommand::RunCallback(callback) => callback(self),
        }
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
