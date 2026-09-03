use crate::repo::window::{DESCRIPTION_DEFAULT, SECONDARY_PANE_DEFAULT};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct LayoutConfig {
    pub(crate) sidebar_width: f32,
    #[serde(alias = "file_column_width")]
    pub(crate) secondary_pane_width: f32,
    pub(crate) description_height: f32,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            sidebar_width: 360.0,
            secondary_pane_width: SECONDARY_PANE_DEFAULT,
            description_height: DESCRIPTION_DEFAULT,
        }
    }
}
