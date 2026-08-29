use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct FeaturesConfig {
    pub(crate) skip_abandon_confirmation: bool,
    pub(crate) confirm_drag_rebase: bool,
}

impl Default for FeaturesConfig {
    fn default() -> Self {
        Self {
            skip_abandon_confirmation: false,
            confirm_drag_rebase: true,
        }
    }
}
