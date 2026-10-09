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
pub(super) use linux::user_mono_families;

/// font-kit's CoreText source has no file paths to tell user fonts apart; GPUI ships on Linux and Windows only.
#[cfg(target_os = "macos")]
pub(super) fn user_mono_families() -> Vec<String> {
    Vec::new()
}
