use glam::{Quat, Vec3};
use renderer_core::{
    application::{run_on_main, AppCommand, Application, WasmCanvas},
    camera::camera_controller::{self, CameraController},
    game::{ModelInfo, ShaderId, ShaderInfo},
    input::WinitAppHelper,
};
use std::sync::Arc;
use tracing::error;
use wasm_bindgen::{prelude::wasm_bindgen, JsError, JsValue};
use web_sys::HtmlCanvasElement;
use winit::event_loop::{EventLoop, EventLoopProxy};

use crate::wasm_abi::{WasmCompilationResults, WasmModelInfo, WasmShaderInfo};

#[wasm_bindgen]
pub struct WasmApplication {
    event_loop_proxy: Option<EventLoopProxy<AppCommand>>,
}

#[wasm_bindgen]
impl WasmApplication {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<WasmApplication, JsError> {
        Ok(Self {
            event_loop_proxy: None,
        })
    }

    pub fn run(&mut self, _canvas: HtmlCanvasElement) -> Result<(), JsError> {
        let event_loop = EventLoop::<AppCommand>::with_user_event().build()?;
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        let event_loop_proxy = event_loop.create_proxy();
        self.event_loop_proxy = Some(event_loop_proxy.clone());
        #[cfg(target_arch = "wasm32")]
        let wasm_canvas = WasmCanvas::new(_canvas);
        #[cfg(not(target_arch = "wasm32"))]
        let wasm_canvas = WasmCanvas::new();
        let mut application = Application::new(event_loop_proxy, |_| {}, wasm_canvas);
        application.app.camera_controller = CameraController::new(
            camera_controller::GeneralController {
                position: Vec3::new(0.0, 0.0, 4.0),
                orientation: Quat::IDENTITY,
                distance_to_center: 4.0,
            },
            application.app.camera_controller.settings,
            camera_controller::ChosenKind::Orbitcam,
        );
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
                id: v.id,
                transform: v.transform.into(),
                material_info: v.material_info.into(),
                shader_id: ShaderId(v.shader_id),
            })
            .collect();
        let _ = run_on_main(self.event_loop_proxy.clone().unwrap(), |app| {
            app.app.update_models(models)
        })
        .await;
    }

    pub async fn update_shader(&self, shader_info: WasmShaderInfo) -> WasmCompilationResults {
        let shader_id = ShaderId(shader_info.id);
        let info = ShaderInfo {
            label: shader_info.label,
            code: shader_info.code,
        };

        let _ = run_on_main(self.event_loop_proxy.clone().unwrap(), {
            let shader_id = shader_id.clone();
            move |app| {
                app.renderer
                    .as_mut()
                    .map(|renderer| renderer.set_shader(shader_id.clone(), &info));
                app.app.set_shader(shader_id, info);
            }
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
        let _ = run_on_main(self.event_loop_proxy.clone().unwrap(), |app| {
            let shader_id = ShaderId(id);
            app.app.remove_shader(&shader_id);
            app.renderer
                .as_mut()
                .map(|renderer| renderer.remove_shader(&shader_id));
        })
        .await;
    }

    pub async fn set_lod_stage(&self, stage: Option<web_sys::js_sys::Function>) {
        let wrapped = stage.map(|stage| -> Arc<dyn Fn(&ShaderId, &str) + 'static> {
            Arc::new(move |shader_id: &ShaderId, buffer_id: &str| {
                let this = wasm_bindgen::JsValue::NULL;
                match stage.call2(
                    &this,
                    &JsValue::from_str(&shader_id.0),
                    &JsValue::from_str(buffer_id),
                ) {
                    Ok(_) => (),
                    Err(e) => error!("Error calling LOD stage: {:?}", e),
                }
            })
        });

        let _ = run_on_main(self.event_loop_proxy.clone().unwrap(), move |app| {
            app.app.lod_stage = wrapped
        })
        .await;
    }
}
