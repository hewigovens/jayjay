#[cfg(not(target_os = "macos"))]
mod linux;
#[cfg(target_os = "macos")]
mod macos;

#[cfg(not(target_os = "macos"))]
pub use linux::{MONO_FONT_FALLBACK_NAMES, MONO_FONT_OPTIONS};
#[cfg(target_os = "macos")]
pub use macos::{MONO_FONT_FALLBACK_NAMES, MONO_FONT_OPTIONS};
