use gpui::WindowBounds;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct WindowState {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub maximized: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        // (0, 0, 0, 0) is treated as "no saved bounds — use centered default"
        Self {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            maximized: false,
        }
    }
}

impl From<WindowBounds> for WindowState {
    fn from(bounds: WindowBounds) -> Self {
        let (bounds, maximized) = match bounds {
            WindowBounds::Maximized(bounds) => (bounds, true),
            WindowBounds::Windowed(bounds) | WindowBounds::Fullscreen(bounds) => (bounds, false),
        };
        Self {
            x: f32::from(bounds.origin.x),
            y: f32::from(bounds.origin.y),
            width: f32::from(bounds.size.width),
            height: f32::from(bounds.size.height),
            maximized,
        }
    }
}

impl WindowState {
    pub fn is_set(&self) -> bool {
        self.width > 0.0 && self.height > 0.0
    }
}
