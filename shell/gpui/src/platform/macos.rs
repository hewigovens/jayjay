use std::path::Path;
use std::process::Command;

use jayjay_core::tools::detach_stdio;

use gpui::Context;

use crate::app::theme::Theme;
use crate::repo::window::RepoWindow;

pub const MOD_KEY: &str = "cmd";
pub const SIDEBAR_TOGGLE_KEY: &str = "ctrl-cmd-s";
pub const SIDEBAR_TOGGLE_SHORTCUT_LABEL: &str = "⌃⌘S";
pub const SHOW_IN_FILE_MANAGER_LABEL: &str = "Show in Finder";
/// Matches the SwiftUI shell's unified toolbar on macOS 26.
pub const TOOLBAR_HEIGHT: f32 = 52.;
pub const TOOLBAR_LEADING_INSET: f32 = 96.;
pub const TOOLBAR_BUTTON_HEIGHT: f32 = 36.;
/// 18 puts the icons' 2-unit stroke on whole device pixels at 2x; SF Symbols in the SwiftUI toolbar measure about the same.
pub const TOOLBAR_ICON_SIZE: f32 = 18.;
pub const REPO_TRAFFIC_LIGHTS: (f32, f32) = (20., 19.);
pub const CUSTOM_TERMINAL_LABEL: &str = "App name";
pub const CUSTOM_TERMINAL_HINT: &str = "e.g. Terminal";

pub fn append_menu_bar(root: gpui::Div, _t: &Theme, _cx: &mut Context<RepoWindow>) -> gpui::Div {
    root
}

pub fn reveal_path(path: &Path) -> bool {
    detach_stdio(Command::new("open").arg("-R").arg(path))
        .spawn()
        .is_ok()
}

pub fn send_notification(_title: &str, _body: &str) -> bool {
    false
}
