use std::collections::HashMap;

use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    window::WindowId,
};

use super::current_input::{WindowInputCollector, WindowInputs};

pub struct WinitAppHelper<Application> {
    pub app: Application,
    windows: HashMap<WindowId, WindowInputCollector>,
}

impl<Application> WinitAppHelper<Application> {
    pub fn new(app: Application) -> Self {
        Self {
            app,
            windows: HashMap::new(),
        }
    }
}

pub trait InputHandler {
    fn update(&mut self, event_loop: &ActiveEventLoop, input: WindowInputs);
}

impl<Application, U> ApplicationHandler<U> for WinitAppHelper<Application>
where
    Application: ApplicationHandler<U> + InputHandler,
    U: 'static,
{
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.app.resumed(event_loop);
    }
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: winit::event::StartCause) {
        self.app.new_events(event_loop, cause);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.app.about_to_wait(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: WindowId,
        event: winit::event::WindowEvent,
    ) {
        if matches!(event, WindowEvent::Destroyed) {
            self.windows.remove(&window_id);
        }

        let window_helper = self
            .windows
            .entry(window_id)
            .or_insert_with(WindowInputCollector::new);
        window_helper.handle_window_event(&event);
        if matches!(event, WindowEvent::RedrawRequested) {
            let update = window_helper.step();
            self.app.update(event_loop, update);
        }

        self.app.window_event(event_loop, window_id, event);
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        for window_helper in self.windows.values_mut() {
            window_helper.handle_device_event(&event);
        }
        self.app.device_event(event_loop, device_id, event);
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        self.app.suspended(event_loop);
    }
    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: U) {
        self.app.user_event(event_loop, event);
    }

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        self.app.exiting(event_loop);
    }

    fn memory_warning(&mut self, event_loop: &ActiveEventLoop) {
        self.app.memory_warning(event_loop);
    }
}
