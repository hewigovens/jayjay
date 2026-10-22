use std::path::Path;
use std::process::Command;

use jayjay_core::tools::detach_stdio;

#[cfg(target_os = "linux")]
use crate::app::APP_ID;

pub const SHOW_IN_FILE_MANAGER_LABEL: &str = "Show in File Manager";
pub const CUSTOM_TERMINAL_LABEL: &str = "Command";
pub const JJ_PACKAGE_MANAGER: (&str, &str) = ("Homebrew", "brew install jj");

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
