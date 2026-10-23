#[cfg(target_os = "macos")]
mod macos;

#[cfg(all(unix, not(target_os = "macos")))]
mod linux;
#[cfg(not(target_os = "macos"))]
mod non_macos;
#[cfg(windows)]
mod windows;

#[cfg(all(unix, not(target_os = "macos")))]
pub(crate) use linux::{
    CUSTOM_TERMINAL_LABEL, JJ_PACKAGE_MANAGER, file_manager_name, reveal_path, send_notification,
};
#[cfg(target_os = "macos")]
pub(crate) use macos::{
    CUSTOM_TERMINAL_LABEL, JJ_PACKAGE_MANAGER, REPO_TRAFFIC_LIGHTS, TOOLBAR_BUTTON_HEIGHT,
    TOOLBAR_HEIGHT, TOOLBAR_ICON_SIZE, TOOLBAR_LEADING_INSET, append_menu_bar, file_manager_name,
    reveal_path, send_notification,
};
#[cfg(target_os = "macos")]
pub use macos::{MOD_KEY, SIDEBAR_TOGGLE_KEY};
#[cfg(not(target_os = "macos"))]
pub use non_macos::{MOD_KEY, SIDEBAR_TOGGLE_KEY};
#[cfg(not(target_os = "macos"))]
pub(crate) use non_macos::{
    REPO_TRAFFIC_LIGHTS, TOOLBAR_BUTTON_HEIGHT, TOOLBAR_HEIGHT, TOOLBAR_ICON_SIZE,
    TOOLBAR_LEADING_INSET, append_menu_bar,
};
#[cfg(windows)]
pub(crate) use windows::{
    CUSTOM_TERMINAL_LABEL, JJ_PACKAGE_MANAGER, file_manager_name, open_url, reveal_path,
    send_notification,
};

pub(crate) fn show_in_file_manager_label() -> String {
    jayjay_i18n::common::show_in(&file_manager_name())
}

/// Runs the platform's URL handler with stdio detached.
#[cfg(not(windows))]
pub(crate) fn open_url(target: &str) -> bool {
    open::commands(target).into_iter().any(|mut command| {
        jayjay_core::tools::detach_stdio(&mut command)
            .status()
            .is_ok_and(|status| status.success())
    })
}

pub(crate) fn numeric_locale() -> String {
    ["LC_ALL", "LC_NUMERIC", "LANG"]
        .into_iter()
        .find_map(|key| std::env::var(key).ok().filter(|value| !value.is_empty()))
        .unwrap_or_else(|| {
            #[cfg(windows)]
            {
                windows::numeric_locale()
            }
            #[cfg(not(windows))]
            {
                String::new()
            }
        })
}
