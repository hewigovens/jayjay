#[cfg(target_os = "linux")]
mod fontconfig;
#[cfg(not(target_os = "macos"))]
mod linux_windows;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(target_os = "linux"))]
mod macos_windows;

#[cfg(target_os = "linux")]
pub(super) use fontconfig::pick;
#[cfg(not(target_os = "macos"))]
pub(super) use linux_windows::{monospaced_families, platform_default_mono, user_families};
#[cfg(target_os = "macos")]
pub(super) use macos::{monospaced_families, platform_default_mono, user_families};
#[cfg(not(target_os = "linux"))]
pub(super) use macos_windows::pick;
