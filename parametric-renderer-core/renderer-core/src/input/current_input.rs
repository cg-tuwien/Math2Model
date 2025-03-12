use std::collections::HashSet;

use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta},
    keyboard::{Key, KeyCode, NativeKeyCode, PhysicalKey},
};

pub struct WindowInputs<'a> {
    pub mouse: WindowMouseInputs<'a>,
    pub keyboard: WindowKeyboardInputs<'a>,
    pub new_size: Option<PhysicalSize<u32>>,
    pub new_scale_factor: Option<f64>,
}

pub struct WindowMouseInputs<'a> {
    pub position: PhysicalPosition<f64>,
    /// With mouse acceleration
    pub position_delta: (f64, f64),
    /// Raw, unfiltered mouse motion
    pub motion: (f64, f64),
    pub scroll_delta: PhysicalPosition<f64>,
    pub inputs: Vec<MouseInput>,
    pub held: &'a HashSet<MouseButton>,
}

impl<'a> WindowMouseInputs<'a> {
    pub fn pressed(&self, button: MouseButton) -> bool {
        self.held.contains(&button)
    }

    pub fn just_pressed(&self, button: MouseButton) -> bool {
        self.inputs
            .iter()
            .any(|input| input.state == ElementState::Pressed && input.button == button)
    }

    pub fn just_released(&self, button: MouseButton) -> bool {
        self.inputs
            .iter()
            .any(|input| input.state == ElementState::Released && input.button == button)
    }
}

pub struct WindowKeyboardInputs<'a> {
    pub inputs: Vec<KeyEvent>,
    pub physical_held: &'a HashSet<PhysicalKey>,
    pub logical_held: &'a HashSet<Key>,
}

impl<'a> WindowKeyboardInputs<'a> {
    pub fn pressed_logical(&self, key: Key) -> bool {
        self.logical_held.contains(&key)
    }

    pub fn just_pressed_logical(&self, key: Key) -> bool {
        self.inputs
            .iter()
            .any(|input| input.state == ElementState::Pressed && input.logical_key == key)
    }

    pub fn just_released_logical(&self, key: Key) -> bool {
        self.inputs
            .iter()
            .any(|input| input.state == ElementState::Released && input.logical_key == key)
    }

    pub fn pressed_physical(&self, key: KeyCode) -> bool {
        let key = PhysicalKey::Code(key);
        self.physical_held.contains(&key)
    }

    pub fn just_pressed_physical(&self, key: KeyCode) -> bool {
        let key = PhysicalKey::Code(key);
        self.inputs
            .iter()
            .any(|input| input.state == ElementState::Pressed && input.physical_key == key)
    }

    pub fn just_released_physical(&self, key: KeyCode) -> bool {
        let key = PhysicalKey::Code(key);
        self.inputs
            .iter()
            .any(|input| input.state == ElementState::Released && input.physical_key == key)
    }

    pub fn pressed_physical_unidentified(&self, key: NativeKeyCode) -> bool {
        let key = PhysicalKey::Unidentified(key);
        self.physical_held.contains(&key)
    }

    pub fn just_pressed_physical_unidentified(&self, key: NativeKeyCode) -> bool {
        let key = PhysicalKey::Unidentified(key);
        self.inputs
            .iter()
            .any(|input| input.state == ElementState::Pressed && input.physical_key == key)
    }

    pub fn just_released_physical_unidentified(&self, key: NativeKeyCode) -> bool {
        let key = PhysicalKey::Unidentified(key);
        self.inputs
            .iter()
            .any(|input| input.state == ElementState::Released && input.physical_key == key)
    }

    pub fn text_input(&self) -> String {
        self.inputs
            .iter()
            .filter_map(|input| input.text.as_ref().map(|v| v.as_str()))
            .collect()
    }
}

pub struct WindowInputCollector {
    start_cursor_position: PhysicalPosition<f64>,
    end_cursor_position: PhysicalPosition<f64>,
    mouse_motion: (f64, f64),
    scroll_delta: PhysicalPosition<f64>,
    mouse_inputs: Vec<MouseInput>,
    mouse_held: HashSet<MouseButton>,
    key_inputs: Vec<KeyEvent>,
    physical_key_held: HashSet<PhysicalKey>,
    key_held: HashSet<Key>,
    new_size: Option<PhysicalSize<u32>>,
    new_scale_factor: Option<f64>,
    // Not supported at the moment
    // - dropped_file
    // - step_start and step_duration
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MouseInput {
    state: ElementState,
    button: MouseButton,
}

impl WindowInputCollector {
    pub fn new() -> Self {
        Self {
            start_cursor_position: Default::default(),
            end_cursor_position: Default::default(),
            mouse_motion: (0.0, 0.0),
            scroll_delta: Default::default(),
            mouse_inputs: Vec::new(),
            mouse_held: HashSet::new(),
            key_inputs: Vec::new(),
            physical_key_held: HashSet::new(),
            key_held: HashSet::new(),
            new_size: None,
            new_scale_factor: None,
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
                match state {
                    ElementState::Pressed => {
                        self.mouse_held.insert(*button);
                    }
                    ElementState::Released => {
                        self.mouse_held.remove(button);
                    }
                }
            }
            winit::event::WindowEvent::MouseWheel { delta, .. } => {
                // Taken from winit_input_helper, which took it from somewhere else.
                const PIXELS_PER_LINE: f64 = 38.0;
                match delta {
                    MouseScrollDelta::LineDelta(x, y) => {
                        self.scroll_delta.x += (*x as f64) * PIXELS_PER_LINE;
                        self.scroll_delta.y += (*y as f64) * PIXELS_PER_LINE;
                    }
                    MouseScrollDelta::PixelDelta(delta) => {
                        self.scroll_delta.x += delta.x;
                        self.scroll_delta.y += delta.y;
                    }
                }
            }
            winit::event::WindowEvent::KeyboardInput { event, .. } => {
                self.key_inputs.push(event.clone());
                match event.state {
                    ElementState::Pressed => {
                        self.key_held.insert(event.logical_key.clone());
                        self.physical_key_held.insert(event.physical_key);
                    }
                    ElementState::Released => {
                        self.key_held.remove(&event.logical_key);
                        self.physical_key_held.remove(&event.physical_key);
                    }
                }
            }
            winit::event::WindowEvent::Resized(size) => {
                self.new_size = Some(*size);
            }
            winit::event::WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.new_scale_factor = Some(*scale_factor);
            }
            _ => {}
        }
    }

    pub fn handle_device_event(&mut self, event: &winit::event::DeviceEvent) {
        match event {
            winit::event::DeviceEvent::MouseMotion { delta } => {
                self.mouse_motion.0 += delta.0;
                self.mouse_motion.1 += delta.1;
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
            mouse: WindowMouseInputs {
                position: self.end_cursor_position,
                position_delta: cursor_delta,
                motion: self.mouse_motion,
                scroll_delta: self.scroll_delta,
                inputs: self.mouse_inputs.clone(),
                held: &self.mouse_held,
            },
            keyboard: WindowKeyboardInputs {
                inputs: self.key_inputs.clone(),
                physical_held: &self.physical_key_held,
                logical_held: &self.key_held,
            },
            new_size: self.new_size,
            new_scale_factor: self.new_scale_factor,
        };

        self.start_cursor_position = self.end_cursor_position;
        self.scroll_delta = Default::default();
        self.mouse_motion = (0.0, 0.0);
        self.mouse_inputs.clear();
        self.key_inputs.clear();
        self.new_size = None;
        self.new_scale_factor = None;

        inputs
    }
}
