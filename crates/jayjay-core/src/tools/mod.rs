//! External editor + terminal launchers, shared by both shells.
//!
//! `open_in_editor` and `open_in_terminal` take a `ToolsConfig` with the
//! user's chosen editor / terminal IDs (matching the SwiftUI `AppSettings`
//! enums). Mac-specific paths use AppleScript / `open -a`; non-mac falls
//! back to `x-terminal-emulator` / `xterm`.

mod editor;
mod terminal;

use std::path::Path;
use std::process::Command;

use crate::repo::find_existing_binary;

use editor::Editor;
use terminal::{Terminal, spawn_terminal};

/// User-configured tool choices. Field names match the SwiftUI
/// `AppSettings` keys so the same config flows through both shells.
#[derive(Debug, Clone, Default)]
pub struct ToolsConfig {
    pub external_editor: String,
    pub custom_editor_command: String,
    pub terminal: String,
    pub custom_terminal_command: String,
}

/// `(config_id, display_label)` pairs for the editor picker.
pub const EDITOR_OPTIONS: &[(&str, &str)] = &[
    ("vscode", "Visual Studio Code"),
    ("vscodium", "VSCodium"),
    ("zed", "Zed"),
    ("xcode", "Xcode"),
    ("vim", "Vim"),
    ("custom", "Custom"),
];

/// `(config_id, display_label)` pairs for the terminal picker.
pub const TERMINAL_OPTIONS: &[(&str, &str)] = &[
    ("terminal", "Terminal"),
    ("iterm", "iTerm2"),
    ("ghostty", "Ghostty"),
    ("custom", "Custom"),
];

/// Open `file_path` (relative to `repo_path`, or absolute) in the user's editor.
/// Returns false when the binary is missing or spawn fails.
pub fn open_in_editor(repo_path: &str, file_path: &str, cfg: &ToolsConfig) -> bool {
    let editor = Editor::from_id(&cfg.external_editor);
    let absolute = absolutize(repo_path, file_path);

    let cmd = match editor {
        Some(e) if e != Editor::Custom => e.command().to_owned(),
        _ => cfg.custom_editor_command.clone(),
    };
    if cmd.is_empty() {
        return false;
    }
    if editor.is_some_and(Editor::is_terminal) {
        return open_in_terminal(repo_path, Some(&format!("{cmd} '{absolute}'")), cfg);
    }
    let Some(binary) = find_existing_binary(&cmd) else {
        return false;
    };
    Command::new(binary).arg(&absolute).spawn().is_ok()
}

/// Open the user's terminal at `repo_path`. If `command` is set, the terminal
/// runs it after `cd`-ing into `repo_path`.
pub fn open_in_terminal(repo_path: &str, command: Option<&str>, cfg: &ToolsConfig) -> bool {
    let term = Terminal::from_id(&cfg.terminal).unwrap_or(Terminal::AppleTerminal);
    spawn_terminal(term, repo_path, command, &cfg.custom_terminal_command)
}

fn absolutize(repo_path: &str, file_path: &str) -> String {
    let p = Path::new(file_path);
    if p.is_absolute() {
        return file_path.to_owned();
    }
    Path::new(repo_path)
        .join(file_path)
        .to_string_lossy()
        .into_owned()
}
