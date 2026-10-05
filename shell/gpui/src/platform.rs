#[cfg(target_os = "macos")]
mod macos;

#[cfg(not(target_os = "macos"))]
mod linux;

#[cfg(not(target_os = "macos"))]
pub(crate) use linux::{
    CUSTOM_TERMINAL_HINT, CUSTOM_TERMINAL_LABEL, REPO_TRAFFIC_LIGHTS, SHOW_IN_FILE_MANAGER_LABEL,
    TOOLBAR_BUTTON_HEIGHT, TOOLBAR_HEIGHT, TOOLBAR_ICON_SIZE, TOOLBAR_LEADING_INSET,
    append_menu_bar, reveal_path, send_notification,
};
#[cfg(not(target_os = "macos"))]
pub use linux::{MOD_KEY, SIDEBAR_TOGGLE_KEY};
#[cfg(target_os = "macos")]
pub(crate) use macos::{
    CUSTOM_TERMINAL_HINT, CUSTOM_TERMINAL_LABEL, REPO_TRAFFIC_LIGHTS, SHOW_IN_FILE_MANAGER_LABEL,
    TOOLBAR_BUTTON_HEIGHT, TOOLBAR_HEIGHT, TOOLBAR_ICON_SIZE, TOOLBAR_LEADING_INSET,
    append_menu_bar, reveal_path, send_notification,
};
#[cfg(target_os = "macos")]
pub use macos::{MOD_KEY, SIDEBAR_TOGGLE_KEY};

/// Runs the platform's URL handler with stdio detached; `open` knows the right command on Linux, macOS, and Windows.
pub(crate) fn open_url(target: &str) -> bool {
    open::commands(target).into_iter().any(|mut command| {
        jayjay_core::tools::detach_stdio(&mut command)
            .status()
            .is_ok_and(|status| status.success())
    })
}
