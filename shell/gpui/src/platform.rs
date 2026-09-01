#[cfg(target_os = "macos")]
mod macos;

#[cfg(not(target_os = "macos"))]
mod linux;

#[cfg(not(target_os = "macos"))]
pub use linux::MOD_KEY;
#[cfg(not(target_os = "macos"))]
pub(crate) use linux::{
    CUSTOM_TERMINAL_HINT, CUSTOM_TERMINAL_LABEL, TOOLBAR_LEADING_INSET, append_menu_bar, open_url,
    reveal_path, send_notification,
};
#[cfg(target_os = "macos")]
pub use macos::MOD_KEY;
#[cfg(target_os = "macos")]
pub(crate) use macos::{
    CUSTOM_TERMINAL_HINT, CUSTOM_TERMINAL_LABEL, TOOLBAR_LEADING_INSET, append_menu_bar, open_url,
    reveal_path, send_notification,
};
