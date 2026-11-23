use crate::repo::window::{OPERATION_LOG_DEFAULT, SECONDARY_PANE_DEFAULT};
use crate::ui::overlay::PromptSize;
use crate::windows::repo_list::RECENT_PANEL_DEFAULT;
use crate::windows::settings::SETTINGS_SIDEBAR_DEFAULT;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct LayoutConfig {
    pub(crate) sidebar_width: f32,
    pub(crate) sidebar_hidden: bool,
    pub show_status_bar: bool,
    pub(crate) show_operation_log: bool,
    pub(crate) operation_log_width: f32,
    #[serde(alias = "file_column_width")]
    pub(crate) secondary_pane_width: f32,
    pub recent_repos_panel: bool,
    pub recent_repos_panel_width: f32,
    pub(crate) settings_sidebar_width: f32,
    pub(crate) description_editor_width: f32,
    pub(crate) description_editor_body_height: f32,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            sidebar_width: 360.0,
            sidebar_hidden: false,
            show_status_bar: true,
            show_operation_log: false,
            operation_log_width: OPERATION_LOG_DEFAULT,
            secondary_pane_width: SECONDARY_PANE_DEFAULT,
            recent_repos_panel: false,
            recent_repos_panel_width: RECENT_PANEL_DEFAULT,
            settings_sidebar_width: SETTINGS_SIDEBAR_DEFAULT,
            description_editor_width: PromptSize::DEFAULT.width,
            description_editor_body_height: PromptSize::DEFAULT.body_height,
        }
    }
}
