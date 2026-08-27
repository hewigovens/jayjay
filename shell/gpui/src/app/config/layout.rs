use crate::repo::window::{DESCRIPTION_DEFAULT, FILE_COLUMN_DEFAULT};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct LayoutConfig {
    pub(crate) sidebar_width: f32,
    pub(crate) file_column_width: f32,
    pub(crate) description_height: f32,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            sidebar_width: 360.0,
            file_column_width: FILE_COLUMN_DEFAULT,
            description_height: DESCRIPTION_DEFAULT,
        }
    }
}
