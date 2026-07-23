use std::path::Path;
use std::process::Command;

use gpui::{Context, ParentElement};

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
    Command::new("xdg-open").arg(target).spawn().is_ok()
        || Command::new("open").arg(target).spawn().is_ok()
}

pub fn open_url(target: &str) -> bool {
    Command::new("xdg-open")
        .arg(target)
        .status()
        .is_ok_and(|status| status.success())
}
