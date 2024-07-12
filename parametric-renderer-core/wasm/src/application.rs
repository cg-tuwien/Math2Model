use std::sync::{Arc, Mutex};

use glamour::{Point3, Vector2};
use renderer_core::{
    application::{CpuApplication, GpuApplication, ModelInfo, WindowOrFallback},
    camera::camera_controller::{self, CameraController},
    input::{InputHandler, WindowInputs, WinitAppHelper},
};
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
use web_sys::HtmlCanvasElement;
use winit::{
    application::ApplicationHandler, dpi::PhysicalSize, event_loop::EventLoopProxy, window::Window,
};
pub struct Application {
    window: Option<Arc<Window>>,
    app: CpuApplication,
    /** For wasm32 */
    _canvas: HtmlCanvasElement,
}

impl Application {
    const DEFAULT_SHADER_CODE: &'static str = include_str!("../../shaders/HeartSphere.wgsl");

    pub fn new(canvas: HtmlCanvasElement) -> anyhow::Result<Self> {
        let mut app = CpuApplication::new()?;
        app.camera_controller = CameraController::new(
            camera_controller::GeneralController {
                position: Point3::new(0.0, 0.0, 4.0),
                orientation: glam::Quat::IDENTITY,
                distance_to_center: 4.0,
            },
            app.camera_controller.settings,
            camera_controller::ChosenKind::Orbitcam,
        );

        app.update_models(vec![ModelInfo {
            label: "Default Model".to_owned(),
            transform: renderer_core::transform::Transform {
                position: glamour::Point3::new(0.0, 1.0, 0.0),
                ..Default::default()
            },
            material_info: renderer_core::application::MaterialInfo {
                color: glamour::Vector3::new(0.6, 1.0, 1.0),
                emissive: glamour::Vector3::new(0.0, 0.0, 0.0),
                roughness: 0.7,
                metallic: 0.1,
            },
            evaluate_image_code: Application::DEFAULT_SHADER_CODE.to_owned(),
        }]);
        Ok(Self {
            window: None,
            app,
            _canvas: canvas,
        })
    }

    fn create_surface(&mut self, window: Window) {
        let window = Arc::new(window);
        self.window = Some(window.clone());

        let gpu_builder = self.app.start_create_gpu(WindowOrFallback::Window(window));

        let task = async move {
            let gpu_application = gpu_builder.create_surface().await.unwrap();
            APP_COMMANDS.with(|commands| {
                if let Some(proxy) = &*commands.lock().unwrap() {
                    let _ = proxy.send_event(AppCommand::CreateGpu(gpu_application));
                }
            });
        };
        wasm_bindgen_futures::spawn_local(task);
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.app
            .resize(Vector2::new(new_size.width, new_size.height));
    }

    pub fn update(&mut self, inputs: &WindowInputs) {
        self.app.update(inputs)
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.app.render()
    }
}

pub enum AppCommand {
    UpdateModels(Vec<ModelInfo>),
    CreateGpu(GpuApplication),
}
thread_local! {
    static APP_COMMANDS: Mutex<Option<EventLoopProxy<AppCommand>>> = Mutex::new(None);
}

#[derive(Serialize, Deserialize)]
pub struct WasmModelInfo {
    pub label: String,
    pub transform: WasmTransform,
    pub material_info: WasmMaterialInfo,
    pub evaluate_image_code: String,
}
#[derive(Serialize, Deserialize)]
pub struct WasmTransform {
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub scale: f32,
}

#[derive(Serialize, Deserialize)]
pub struct WasmMaterialInfo {
    pub color: [f32; 3],
    pub emissive: [f32; 3],
    pub roughness: f32,
    pub metallic: f32,
}

#[wasm_bindgen]
pub fn update_models(js_models: JsValue) {
    APP_COMMANDS.with(|commands| {
        if let Some(proxy) = &*commands.lock().unwrap() {
            let js_models: Vec<WasmModelInfo> = serde_wasm_bindgen::from_value(js_models).unwrap();
            let models = js_models
                .into_iter()
                .map(|v| ModelInfo {
                    label: v.label,
                    transform: renderer_core::transform::Transform {
                        position: v.transform.position.into(),
                        rotation: glam::Quat::from_euler(
                            glam::EulerRot::XYZ,
                            v.transform.rotation[0],
                            v.transform.rotation[1],
                            v.transform.rotation[2],
                        ),
                        scale: v.transform.scale,
                    },
                    material_info: renderer_core::application::MaterialInfo {
                        color: v.material_info.color.into(),
                        emissive: v.material_info.emissive.into(),
                        roughness: v.material_info.roughness,
                        metallic: v.material_info.metallic,
                    },
                    evaluate_image_code: v.evaluate_image_code,
                })
                .collect();
            let _ = proxy.send_event(AppCommand::UpdateModels(models));
        }
    });
}

pub async fn run(canvas: HtmlCanvasElement) -> anyhow::Result<()> {
    let event_loop = winit::event_loop::EventLoop::<AppCommand>::with_user_event().build()?;
    APP_COMMANDS.with(|commands| {
        *commands.lock().unwrap() = Some(event_loop.create_proxy());
    });
    let application = Application::new(canvas)?;
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
        self.create_surface(window);
    }

    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, event: AppCommand) {
        match event {
            AppCommand::UpdateModels(models) => {
                self.app.update_models(models);
            }
            AppCommand::CreateGpu(gpu_application) => {
                self.app.set_gpu(gpu_application);
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
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
