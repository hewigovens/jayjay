use crate::repo::window::SECONDARY_PANE_DEFAULT;
use crate::windows::repo_list::RECENT_PANEL_DEFAULT;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct LayoutConfig {
    pub(crate) sidebar_width: f32,
    pub(crate) sidebar_hidden: bool,
    #[serde(alias = "file_column_width")]
    pub(crate) secondary_pane_width: f32,
    pub recent_repos_panel: bool,
    pub recent_repos_panel_width: f32,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            sidebar_width: 360.0,
            sidebar_hidden: false,
            secondary_pane_width: SECONDARY_PANE_DEFAULT,
            recent_repos_panel: false,
            recent_repos_panel_width: RECENT_PANEL_DEFAULT,
        }
    }
}
