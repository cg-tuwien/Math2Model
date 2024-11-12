use std::sync::Arc;

use glam::UVec2;
use winit::window::Window;

pub enum WindowOrFallback {
    Window(Arc<Window>),
    Headless { size: UVec2 },
}

impl WindowOrFallback {
    pub fn size(&self) -> UVec2 {
        match self {
            WindowOrFallback::Window(window) => {
                let window_size = window.inner_size();
                UVec2::new(window_size.width, window_size.height)
            }
            WindowOrFallback::Headless { size } => *size,
        }
    }

    pub fn as_window(&self) -> Option<Arc<Window>> {
        match self {
            WindowOrFallback::Window(window) => Some(window.clone()),
            WindowOrFallback::Headless { .. } => None,
        }
    }
}
