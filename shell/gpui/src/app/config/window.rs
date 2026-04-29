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

impl WindowState {
    pub fn is_set(&self) -> bool {
        self.width > 0.0 && self.height > 0.0
    }
}
