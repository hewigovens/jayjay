use crate::repo::window::DESCRIPTION_DEFAULT;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct LayoutConfig {
    pub sidebar_width: f32,
    pub description_height: f32,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            sidebar_width: 360.0,
            description_height: DESCRIPTION_DEFAULT,
        }
    }
}
