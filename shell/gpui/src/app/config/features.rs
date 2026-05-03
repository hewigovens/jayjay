use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct FeaturesConfig {
    pub skip_abandon_confirmation: bool,
    pub confirm_drag_rebase: bool,
}

impl Default for FeaturesConfig {
    fn default() -> Self {
        Self {
            skip_abandon_confirmation: false,
            confirm_drag_rebase: true,
        }
    }
}
