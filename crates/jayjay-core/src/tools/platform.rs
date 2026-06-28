#[cfg(target_os = "macos")]
mod macos;

#[cfg(not(target_os = "macos"))]
mod linux;

#[cfg(not(target_os = "macos"))]
pub use linux::{EDITOR_OPTIONS, TERMINAL_OPTIONS, spawn_terminal};
#[cfg(target_os = "macos")]
pub use macos::{EDITOR_OPTIONS, TERMINAL_OPTIONS, spawn_terminal};
