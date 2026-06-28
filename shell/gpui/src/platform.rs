#[cfg(target_os = "macos")]
mod macos;

#[cfg(not(target_os = "macos"))]
mod linux;

#[cfg(not(target_os = "macos"))]
pub use linux::{
    CUSTOM_TERMINAL_HINT, CUSTOM_TERMINAL_LABEL, MOD_KEY, TOOLBAR_LEADING_INSET, append_menu_bar,
    reveal_path,
};
#[cfg(target_os = "macos")]
pub use macos::{
    CUSTOM_TERMINAL_HINT, CUSTOM_TERMINAL_LABEL, MOD_KEY, TOOLBAR_LEADING_INSET, append_menu_bar,
    reveal_path,
};
