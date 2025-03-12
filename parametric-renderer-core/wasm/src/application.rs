use glam::{Quat, Vec3};
use log::error;
use renderer_core::{
    application::{AppCommand, Application, WasmCanvas, run_on_main},
    camera::camera_controller::{self, CameraController},
    game::{ModelInfo, ShaderId, ShaderInfo, TextureData, TextureId, TextureInfo},
    input::WinitAppHelper,
    time::TimeStats,
};
use std::sync::{Arc, Mutex};
use wasm_bindgen::{JsError, JsValue, prelude::wasm_bindgen};
use web_sys::{HtmlCanvasElement, ImageBitmap};
use winit::event_loop::{EventLoop, EventLoopProxy};

use crate::wasm_abi::{WasmCompilationMessage, WasmFrameTime, WasmModelInfo, WasmShaderInfo};

#[wasm_bindgen]
pub struct WasmApplication {
    event_loop_proxy: Option<EventLoopProxy<AppCommand>>,
    last_time_stats: TimeStats,
    time_stats: Arc<Mutex<TimeStats>>,
}

#[wasm_bindgen]
impl WasmApplication {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<WasmApplication, JsError> {
        Ok(Self {
            event_loop_proxy: None,
            last_time_stats: Default::default(),
            time_stats: Default::default(),
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
        self.time_stats = application.time_stats.clone();
        application.app.profiler_settings.gpu = true;
        application.app.camera_controller = CameraController::new(
            camera_controller::GeneralController {
                position: Vec3::new(3.0, 7.0, 6.0),
                // TODO: Do this once glam updoots
                // orientation: Quat::look_at_rh(Vec3::new(3.0, 7.0, 6.0), Vec3::ZERO, Camera::up()),
                orientation: Quat::from_euler(glam::EulerRot::YXZ, 0.5, -0.6, 0.0),
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
                instance_count: v.instance_count,
            })
            .collect::<Vec<_>>();
        let _ = run_on_main(self.event_loop_proxy.clone().unwrap(), |app| {
            app.renderer.as_mut().map(|renderer| {
                renderer.update_models(&models);
            });
            app.app.update_models(models);
        })
        .await;
    }

    pub async fn update_shader(&self, shader_info: WasmShaderInfo) {
        let shader_id = ShaderId(shader_info.id);
        let info = ShaderInfo {
            label: shader_info.label,
            code: shader_info.code,
        };

        let _ = run_on_main(self.event_loop_proxy.clone().unwrap(), {
            let shader_id = shader_id.clone();
            move |app| {
                let on_shader_compiled = app.on_shader_compiled.clone();
                app.renderer.as_mut().map(|renderer| {
                    renderer.set_shader(shader_id.clone(), &info, on_shader_compiled)
                });
                app.app.set_shader(shader_id, info);
            }
        })
        .await;
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

    pub async fn update_texture(&self, texture_id: String, image: ImageBitmap) {
        let id = TextureId(texture_id);
        let info = TextureInfo {
            width: image.width(),
            height: image.height(),
            #[cfg(target_arch = "wasm32")]
            data: TextureData::Image(image),
            #[cfg(not(target_arch = "wasm32"))]
            data: TextureData::Bytes(vec![0, 0, 0]),
        };

        let _ = run_on_main(self.event_loop_proxy.clone().unwrap(), {
            let id = id.clone();
            move |app| {
                app.renderer
                    .as_mut()
                    .map(|renderer| renderer.set_texture(id.clone(), &info));
                app.app.set_texture(id, info);
            }
        })
        .await;
    }

    pub async fn remove_texture(&self, id: String) {
        let _ = run_on_main(self.event_loop_proxy.clone().unwrap(), |app| {
            let id = TextureId(id);
            app.app.remove_texture(&id);
            app.renderer
                .as_mut()
                .map(|renderer| renderer.remove_texture(&id));
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
            app.app.lod_stage = wrapped;
        })
        .await;
    }

    pub async fn set_on_shader_compiled(
        &mut self,
        on_shader_compiled: Option<web_sys::js_sys::Function>,
    ) {
        let wrapped = on_shader_compiled.map(|on_shader_compiled| -> Arc<dyn Fn(&ShaderId, Vec<wgpu::CompilationMessage>) + 'static> {
            Arc::new(move |shader_id: &ShaderId, messages: Vec<wgpu::CompilationMessage>| {
                let this = wasm_bindgen::JsValue::NULL;
                let messages = messages
                    .into_iter()
                    .map(|message| WasmCompilationMessage::from(message))
                    .collect::<Vec<_>>();
                match on_shader_compiled.call2(
                    &this,
                    &JsValue::from_str(&shader_id.0),
                    &serde_wasm_bindgen::to_value(&messages).unwrap(),
                ) {
                    Ok(_) => (),
                    Err(e) => error!("Error calling on_shader_compiled: {:?}", e),
                }
            })
        });
        let _ = run_on_main(self.event_loop_proxy.clone().unwrap(), move |app| {
            app.on_shader_compiled = wrapped;
        })
        .await;
    }

    pub fn get_frame_time(&mut self) -> WasmFrameTime {
        if let Ok(new_stats) = self.time_stats.try_lock() {
            self.last_time_stats = new_stats.clone();
        }
        WasmFrameTime {
            avg_delta_time: self.last_time_stats.avg_delta_time,
            avg_gpu_time: self.last_time_stats.avg_gpu_time,
        }
    }

    pub async fn set_threshold_factor(&self, factor: f32) {
        let _ = run_on_main(self.event_loop_proxy.clone().unwrap(), move |app| {
            if let Some(renderer) = &app.renderer {
                renderer.set_threshold_factor(factor);
            }
        })
        .await;
    }
}
