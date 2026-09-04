use std::path::Path;
use std::process::Command;

use jayjay_core::tools::detach_stdio;

use gpui::{Context, ParentElement};

#[cfg(target_os = "linux")]
use crate::app::APP_ID;
use crate::app::theme::Theme;
use crate::repo::window::RepoWindow;

pub const MOD_KEY: &str = "ctrl";
pub const TOOLBAR_LEADING_INSET: f32 = 12.;
pub const CUSTOM_TERMINAL_LABEL: &str = "Command";
pub const CUSTOM_TERMINAL_HINT: &str = "e.g. alacritty";

pub fn append_menu_bar(root: gpui::Div, t: &Theme, cx: &mut Context<RepoWindow>) -> gpui::Div {
    root.child(crate::ui::app_menu::menu_bar(t, cx))
}

pub fn reveal_path(path: &Path) -> bool {
    let target = if path.is_file() {
        path.parent().unwrap_or(path)
    } else {
        path
    };
    detach_stdio(Command::new("xdg-open").arg(target))
        .spawn()
        .is_ok()
        || detach_stdio(Command::new("open").arg(target))
            .spawn()
            .is_ok()
}

#[cfg(target_os = "linux")]
pub fn send_notification(title: &str, body: &str) -> bool {
    notify_rust::Notification::new()
        .appname("JayJay")
        .summary(title)
        .body(body)
        .icon(APP_ID)
        .show()
        .is_ok()
}

#[cfg(not(target_os = "linux"))]
pub fn send_notification(_title: &str, _body: &str) -> bool {
    false
}
