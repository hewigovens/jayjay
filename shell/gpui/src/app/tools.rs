//! Thin wrapper over `jayjay_core::tools` — pulls the user's tool config
//! out of `AppConfig` and forwards. Real launcher logic lives in core so
//! both shells share it.

use gpui::App;

use crate::app::config;

pub use jayjay_core::tools::{EDITOR_OPTIONS, TERMINAL_OPTIONS};

pub fn open_in_editor(repo_path: &str, file_path: &str, cx: &App) -> bool {
    jayjay_core::open_in_editor(repo_path, file_path, &cfg(cx))
}

pub fn open_in_terminal(repo_path: &str, cx: &App) -> bool {
    jayjay_core::open_in_terminal(repo_path, None, &cfg(cx))
}

fn cfg(cx: &App) -> jayjay_core::ToolsConfig {
    let app_cfg = config::current(cx);
    jayjay_core::ToolsConfig {
        external_editor: app_cfg.tools.external_editor.clone(),
        custom_editor_command: app_cfg.tools.custom_editor_command.clone(),
        terminal: app_cfg.tools.terminal.clone(),
        custom_terminal_command: app_cfg.tools.custom_terminal_command.clone(),
    }
}
