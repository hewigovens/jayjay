//! External editor and terminal launchers shared by both shells.

mod config;
mod editor;
mod file_url;
mod launcher;
mod platform;
mod terminal;

pub use config::ToolsConfig;
pub use file_url::repo_file_url;
pub use launcher::{open_in_editor, open_in_terminal};
pub use platform::{EDITOR_OPTIONS, TERMINAL_OPTIONS};
