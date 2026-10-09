#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;
#[cfg(not(any(target_os = "macos", windows)))]
mod xdg;

#[cfg(target_os = "macos")]
pub use macos::user_font_dirs;
#[cfg(windows)]
pub use windows::user_font_dirs;
#[cfg(not(any(target_os = "macos", windows)))]
pub use xdg::user_font_dirs;
