// Heavily based on https://github.com/rukai/winit_input_helper/
// But with a lot of modifications
mod current_input;
mod cursor_capture;
mod winit_helper;

pub use current_input::{MouseInput, WindowInputs, WindowKeyboardInputs, WindowMouseInputs};
pub use cursor_capture::{CursorCaptureRequest, WindowCursorCapture};
pub use winit_helper::{InputHandler, WinitAppHelper};
