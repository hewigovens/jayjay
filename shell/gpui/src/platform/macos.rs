use std::path::Path;
use std::process::Command;

use gpui::Context;
use objc2_app_kit::NSWorkspace;
use objc2_foundation::{NSString, NSURL};

use crate::app::theme::Theme;
use crate::repo::window::RepoWindow;

pub const MOD_KEY: &str = "cmd";
pub const TOOLBAR_LEADING_INSET: f32 = 78.;
pub const CUSTOM_TERMINAL_LABEL: &str = "App name";
pub const CUSTOM_TERMINAL_HINT: &str = "e.g. Terminal";

pub fn append_menu_bar(root: gpui::Div, _t: &Theme, _cx: &mut Context<RepoWindow>) -> gpui::Div {
    root
}

pub fn reveal_path(path: &Path) -> bool {
    Command::new("open").arg("-R").arg(path).spawn().is_ok()
}

pub fn open_url(target: &str) -> bool {
    let Some(url) = NSURL::URLWithString(&NSString::from_str(target)) else {
        return false;
    };
    NSWorkspace::sharedWorkspace().openURL(&url)
}
