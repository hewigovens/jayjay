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
