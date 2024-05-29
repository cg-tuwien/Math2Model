use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta},
};

pub struct WindowInputs {
    pub cursor_position: PhysicalPosition<f64>,
    /// With mouse acceleration
    pub cursor_delta: (f64, f64),
    /// Raw, unfiltered mouse motion
    pub mouse_motion: (f64, f64),
    pub scroll_delta: MouseScrollDelta,
    pub mouse_inputs: Vec<MouseInput>,
    pub key_inputs: Vec<KeyEvent>,
}

pub struct WindowInputCollector {
    start_cursor_position: PhysicalPosition<f64>,
    end_cursor_position: PhysicalPosition<f64>,
    mouse_motion: (f64, f64),
    scroll_delta: MouseScrollDelta,
    mouse_inputs: Vec<MouseInput>,
    key_inputs: Vec<KeyEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MouseInput {
    state: ElementState,
    button: MouseButton,
}

impl WindowInputCollector {
    pub fn new() -> Self {
        Self {
            start_cursor_position: PhysicalPosition::new(0.0, 0.0),
            end_cursor_position: PhysicalPosition::new(0.0, 0.0),
            mouse_motion: (0.0, 0.0),
            scroll_delta: MouseScrollDelta::LineDelta(0.0, 0.0),
            mouse_inputs: Vec::new(),
            key_inputs: Vec::new(),
        }
    }

    pub fn handle_window_event(&mut self, event: &winit::event::WindowEvent) {
        match event {
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                self.end_cursor_position = *position;
            }
            winit::event::WindowEvent::MouseInput { state, button, .. } => {
                self.mouse_inputs.push(MouseInput {
                    state: *state,
                    button: *button,
                });
            }
            winit::event::WindowEvent::MouseWheel { delta, .. } => {
                self.scroll_delta = *delta;
            }
            winit::event::WindowEvent::KeyboardInput { event, .. } => {
                self.key_inputs.push(event.clone());
            }
            _ => {}
        }
    }

    pub fn handle_device_event(&mut self, event: &winit::event::DeviceEvent) {
        match event {
            winit::event::DeviceEvent::MouseMotion { delta } => {
                self.mouse_motion = *delta;
            }
            _ => {}
        }
    }

    pub fn step(&mut self) -> WindowInputs {
        let cursor_delta = (
            self.end_cursor_position.x - self.start_cursor_position.x,
            self.end_cursor_position.y - self.start_cursor_position.y,
        );
        let inputs = WindowInputs {
            cursor_position: self.end_cursor_position,
            cursor_delta,
            mouse_motion: self.mouse_motion,
            scroll_delta: self.scroll_delta,
            mouse_inputs: self.mouse_inputs.clone(),
            key_inputs: self.key_inputs.clone(),
        };

        self.start_cursor_position = self.end_cursor_position;
        self.scroll_delta = MouseScrollDelta::LineDelta(0.0, 0.0);
        self.mouse_motion = (0.0, 0.0);
        self.mouse_inputs.clear();
        self.key_inputs.clear();

        inputs
    }
}
