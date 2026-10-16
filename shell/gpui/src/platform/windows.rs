use std::path::Path;

pub const SHOW_IN_FILE_MANAGER_LABEL: &str = "Show in File Explorer";
pub const CUSTOM_TERMINAL_LABEL: &str = "Command (accepts PowerShell arguments)";
/// GPUI drops the native caption buttons and dragging for a transparent title bar on Windows.
pub const TRANSPARENT_TITLEBAR: bool = false;
pub const JJ_PACKAGE_MANAGER: (&str, &str) = ("WinGet", "winget install jj-vcs.jj");

pub fn open_url(target: &str) -> bool {
    jayjay_core::tools::windows_shell::open(target)
}

pub fn reveal_path(path: &Path) -> bool {
    jayjay_core::tools::windows_shell::reveal(path)
}

pub fn send_notification(_title: &str, _body: &str) -> bool {
    false
}
