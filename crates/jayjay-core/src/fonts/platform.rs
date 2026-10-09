#[cfg(not(target_os = "macos"))]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;
#[cfg(not(any(target_os = "macos", windows)))]
mod xdg;

#[cfg(not(target_os = "macos"))]
pub use linux::{MONO_FONT_FALLBACK_NAMES, MONO_FONT_OPTIONS};
#[cfg(target_os = "macos")]
pub use macos::{MONO_FONT_FALLBACK_NAMES, MONO_FONT_OPTIONS, user_font_dirs};
#[cfg(windows)]
pub use windows::user_font_dirs;
#[cfg(not(any(target_os = "macos", windows)))]
pub use xdg::user_font_dirs;
