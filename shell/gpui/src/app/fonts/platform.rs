#[cfg(not(target_os = "macos"))]
mod linux;
#[cfg(target_os = "macos")]
mod macos;

#[cfg(not(target_os = "macos"))]
pub(super) fn platform_default_mono() -> String {
    linux::platform_default_mono()
}

#[cfg(target_os = "macos")]
pub(super) fn platform_default_mono() -> String {
    macos::platform_default_mono()
}

#[cfg(not(target_os = "macos"))]
pub(super) use linux::{monospaced_families, user_families};

/// font-kit's CoreText source loads every font into memory and has no file paths; GPUI ships on Linux and Windows only.
#[cfg(target_os = "macos")]
pub(super) fn monospaced_families() -> Vec<String> {
    Vec::new()
}

#[cfg(target_os = "macos")]
pub(super) fn user_families() -> Vec<String> {
    Vec::new()
}
