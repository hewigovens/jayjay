#[cfg(target_os = "macos")]
mod macos;

#[cfg(all(unix, not(target_os = "macos")))]
mod linux;
#[cfg(any(windows, test))]
mod powershell;
#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

#[cfg(all(unix, not(target_os = "macos")))]
pub use linux::{
    DEFAULT_EDITOR, EDITOR_OPTIONS, TERMINAL_OPTIONS, UNKNOWN_EDITOR_NEEDS_TERMINAL,
    default_text_editor, open_path, spawn_terminal,
};
#[cfg(target_os = "macos")]
pub use macos::{
    DEFAULT_EDITOR, EDITOR_OPTIONS, TERMINAL_OPTIONS, UNKNOWN_EDITOR_NEEDS_TERMINAL,
    default_text_editor, open_path, spawn_terminal,
};
#[cfg(unix)]
pub use unix::{command_words, run_terminal_tool, terminal_line};
#[cfg(windows)]
pub use windows::{
    DEFAULT_EDITOR, EDITOR_OPTIONS, TERMINAL_OPTIONS, UNKNOWN_EDITOR_NEEDS_TERMINAL, command_words,
    default_text_editor, open_path, run_terminal_tool, shell as windows_shell, spawn_terminal,
    terminal_line,
};
