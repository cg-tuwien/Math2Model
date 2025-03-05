#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CursorCaptureRequest {
    Free,
    LockedAndHidden,
}

#[derive(Debug, Clone, Copy)]
pub enum WindowCursorCapture {
    Free,
    LockedAndHidden(winit::dpi::PhysicalPosition<f64>),
}

impl WindowCursorCapture {
    pub fn update(&mut self, request: WindowCursorCapture, window: &winit::window::Window) {
        *self = match (&self, request) {
            (WindowCursorCapture::LockedAndHidden(position), WindowCursorCapture::Free) => {
                window
                    .set_cursor_grab(winit::window::CursorGrabMode::None)
                    .unwrap();
                window.set_cursor_visible(true);
                let _ = window.set_cursor_position(*position);
                WindowCursorCapture::Free
            }
            (WindowCursorCapture::Free, WindowCursorCapture::Free) => WindowCursorCapture::Free,
            (
                WindowCursorCapture::LockedAndHidden(_),
                WindowCursorCapture::LockedAndHidden(position),
            ) => WindowCursorCapture::LockedAndHidden(position),
            (WindowCursorCapture::Free, WindowCursorCapture::LockedAndHidden(cursor_position)) => {
                window
                    .set_cursor_grab(winit::window::CursorGrabMode::Confined)
                    .or_else(|_e| window.set_cursor_grab(winit::window::CursorGrabMode::Locked))
                    .unwrap();
                window.set_cursor_visible(false);
                WindowCursorCapture::LockedAndHidden(cursor_position)
            }
        };
    }
}
