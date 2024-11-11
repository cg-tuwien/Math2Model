use std::sync::Arc;

use glam::UVec2;
use log::{error, info, warn};
use pinky_swear::PinkySwear;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event_loop::EventLoopProxy,
    keyboard::{Key, NamedKey},
    window::Window,
};

use crate::{
    game::{GameRes, ShaderId},
    input::{InputHandler, WindowInputs},
    renderer::{GpuApplication, GpuApplicationBuilder},
    window_or_fallback::WindowOrFallback,
};
pub struct WasmCanvas {
    #[cfg(target_arch = "wasm32")]
    pub canvas: web_sys::HtmlCanvasElement,
}
impl WasmCanvas {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new() -> Self {
        Self {}
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new(canvas: web_sys::HtmlCanvasElement) -> Self {
        Self { canvas }
    }
}

pub struct Application {
    pub app: GameRes,
    window: Option<Arc<Window>>,
    pub renderer: Option<GpuApplication>,
    app_commands: EventLoopProxy<AppCommand>,
    on_exit_callback: Option<Box<dyn FnOnce(&mut Application)>>,
    pub on_shader_compiled: Option<Arc<dyn Fn(&ShaderId, Vec<wgpu::CompilationMessage>)>>,
    _canvas: WasmCanvas,
}

impl Application {
    pub fn new(
        app_commands: EventLoopProxy<AppCommand>,
        on_exit: impl FnOnce(&mut Application) + 'static,
        canvas: WasmCanvas,
    ) -> Self {
        Self {
            window: None,
            app: GameRes::new(),
            renderer: None,
            app_commands,
            on_exit_callback: Some(Box::new(on_exit)),
            on_shader_compiled: None,
            _canvas: canvas,
        }
    }

    fn on_exit(&mut self) {
        info!("Stopping the application.");
        if let Some(on_exit_callback) = self.on_exit_callback.take() {
            on_exit_callback(self);
        }
    }

    fn create_surface(&mut self, window: Window) {
        let window = Arc::new(window);
        self.window = Some(window.clone());

        let gpu_builder = GpuApplicationBuilder::new(WindowOrFallback::Window(window));

        let app_commands = self.app_commands.clone();
        let on_shader_compiled = self.on_shader_compiled.clone();
        let task = async move {
            println!("Creating renderer");
            let renderer = gpu_builder.await.unwrap().build();
            let _ = run_on_main(app_commands, move |app| {
                for (shader_id, shader_info) in &app.app.shaders {
                    renderer.set_shader(shader_id.clone(), shader_info, on_shader_compiled.clone());
                }
                app.renderer = Some(renderer)
            })
            .await;
        };
        any_spawner::Executor::spawn_local(task);
    }
}

pub enum AppCommand {
    RunCallback(Box<dyn FnOnce(&mut Application)>),
}

/// Run a function on the main thread and awaits its result.
/// Not a part of the Application, because we want to be able to call this without the lifetime constraint of the Application.
#[must_use]
pub async fn run_on_main<Callback, T>(
    app_commands: EventLoopProxy<AppCommand>,
    callback: Callback,
) -> T
where
    Callback: (FnOnce(&mut Application) -> T) + 'static,
    T: Send + 'static,
{
    let (promise, pinky) = PinkySwear::<T>::new();
    let callback = move |app: &mut Application| {
        let return_value = callback(app);
        pinky.swear(return_value);
    };
    app_commands
        .send_event(AppCommand::RunCallback(Box::new(callback)))
        .map_err(|_| ())
        .expect("Failed to send event, event loop not running?");
    promise.await
}

impl ApplicationHandler<AppCommand> for Application {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        // A really good app might recreate the renderer here?
        if let Some(window) = &self.window {
            window.request_redraw();
            return;
        }

        let window_attributes = Window::default_attributes();
        #[cfg(target_arch = "wasm32")]
        let window_attributes = {
            use winit::platform::web::WindowAttributesExtWebSys;
            window_attributes.with_canvas(Some(self._canvas.canvas.clone()))
        };

        let window = event_loop.create_window(window_attributes).unwrap();
        self.create_surface(window);
    }

    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, event: AppCommand) {
        match event {
            AppCommand::RunCallback(callback) => callback(self),
        }
    }

    fn new_events(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _cause: winit::event::StartCause,
    ) {
        any_spawner::Executor::poll_local();
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
            // TODO: I think I can get rid of this
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
        #[cfg(not(target_arch = "wasm32"))]
        if input
            .keyboard
            .just_released_logical(Key::Named(NamedKey::Escape))
        {
            self.on_exit();
            return event_loop.exit();
        }
        if input.close_requested {
            self.on_exit();
            return event_loop.exit();
        }

        // Press P to print profiling data
        #[cfg(not(target_arch = "wasm32"))]
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
            self.renderer
                .as_mut()
                .map(|r| r.resize(UVec2::new(width, height)));
        }
        self.app.update(&input);
        match self.renderer.as_mut().map(|r| r.render(&self.app)) {
            None => (),
            Some(Ok(_)) => (),
            Some(Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated)) => {
                info!("Lost or outdated surface");
                // Nothing to do, surface will be recreated
            }
            Some(Err(wgpu::SurfaceError::OutOfMemory)) => {
                error!("Out of memory");
                self.on_exit();
                return event_loop.exit();
            }
            Some(Err(e)) => {
                warn!("Unexpected error: {:?}", e);
            }
        }
    }
}
