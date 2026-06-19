//! External editor + terminal launchers, shared by both shells.
//!
//! `open_in_editor` and `open_in_terminal` take a `ToolsConfig` with the
//! user's chosen editor / terminal IDs (matching the SwiftUI `AppSettings`
//! enums). Mac-specific paths use AppleScript / `open -a`; non-mac falls
//! back to `x-terminal-emulator` / `xterm`.

mod editor;
mod terminal;

use std::path::Path;

use crate::repo::{find_existing_binary, subprocess_command};

use editor::Editor;
use terminal::{Terminal, escape_single_quotes, spawn_terminal};

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
    ("cursor", "Cursor"),
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
        return open_in_terminal(
            repo_path,
            Some(&editor_terminal_command(&cmd, &absolute)),
            cfg,
        );
    }
    let Some(binary) = find_existing_binary(&cmd) else {
        return false;
    };
    let args = editor.map(Editor::launch_args).unwrap_or_default();
    subprocess_command(&binary)
        .args(args)
        .arg(&absolute)
        .spawn()
        .is_ok()
}

/// Open the user's terminal at `repo_path`. If `command` is set, the terminal
/// runs it after `cd`-ing into `repo_path`.
pub fn open_in_terminal(repo_path: &str, command: Option<&str>, cfg: &ToolsConfig) -> bool {
    let term = Terminal::from_id(&cfg.terminal).unwrap_or(Terminal::AppleTerminal);
    spawn_terminal(term, repo_path, command, &cfg.custom_terminal_command)
}

/// Build the `<editor> '<path>'` shell command for a terminal editor.
/// `path` is attacker-controlled (any cloned repo), so single-quote escape it
/// before splicing into the shell line the terminal executes.
fn editor_terminal_command(cmd: &str, path: &str) -> String {
    format!("{cmd} '{}'", escape_single_quotes(path))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_command_quotes_plain_path() {
        assert_eq!(
            editor_terminal_command("vim", "/repo/src/main.rs"),
            "vim '/repo/src/main.rs'"
        );
    }

    #[test]
    fn editor_command_keeps_apostrophe_filename_intact() {
        // A legitimate filename with an apostrophe must stay a single word.
        assert_eq!(
            editor_terminal_command("vim", "/repo/don't.md"),
            "vim '/repo/don'\\''t.md'"
        );
    }

    #[test]
    fn editor_command_neutralizes_injection_payload() {
        // Crafted filename in a cloned repo: the closing quote + payload must
        // not escape the single-quoted word.
        let cmd = editor_terminal_command("vim", "/repo/a'$(touch /tmp/pwned)'.rs");
        // The only unescaped single quotes are the wrapping pair we added.
        assert!(cmd.starts_with("vim '"));
        assert!(cmd.ends_with('\''));
        // The payload's quotes are escaped via the '\'' idiom, so `$(...)`
        // stays literal inside the quoted word rather than running as a command.
        assert!(cmd.contains("a'\\''$(touch /tmp/pwned)'\\''.rs"));
    }
}
