//! External editor + terminal launchers, shared by both shells.
//!
//! `open_in_editor` and `open_in_terminal` take a `ToolsConfig` with the
//! user's chosen editor / terminal IDs (matching the SwiftUI `AppSettings`
//! enums). Mac-specific paths use AppleScript / `open -a`; non-mac uses
//! common terminal commands and falls back to `x-terminal-emulator` / `xterm`.

mod editor;
mod platform;
mod terminal;

use std::path::{Component, Path, PathBuf};

use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};

use crate::repo::{find_existing_binary, subprocess_command};

use editor::Editor;
use terminal::{Terminal, escape_single_quotes};

pub use platform::{EDITOR_OPTIONS, TERMINAL_OPTIONS};

/// User-configured tool choices. Field names match the SwiftUI
/// `AppSettings` keys so the same config flows through both shells.
#[derive(Debug, Clone, Default)]
pub struct ToolsConfig {
    pub external_editor: String,
    pub custom_editor_command: String,
    pub terminal: String,
    pub custom_terminal_command: String,
}

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
    let term = Terminal::from_id(&cfg.terminal).unwrap_or(Terminal::SystemDefault);
    platform::spawn_terminal(term, repo_path, command, &cfg.custom_terminal_command)
}

/// Return a `file://` URL for an existing non-directory file inside `repo_path`.
pub fn repo_file_url(repo_path: &str, file_path: &str) -> Option<String> {
    existing_repo_file_path(repo_path, file_path)
        .as_deref()
        .map(file_url_from_path)
}

fn existing_repo_file_path(repo_path: &str, file_path: &str) -> Option<PathBuf> {
    let relative = Path::new(file_path);
    if relative.is_absolute()
        || relative.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return None;
    }

    let repo = Path::new(repo_path).canonicalize().ok()?;
    let candidate = repo.join(relative).canonicalize().ok()?;
    if !candidate.starts_with(&repo) || !candidate.is_file() {
        return None;
    }
    Some(candidate)
}

fn file_url_from_path(path: &Path) -> String {
    let mut path = path.to_string_lossy().replace('\\', "/");
    if cfg!(windows) && path.as_bytes().get(1) == Some(&b':') {
        path.insert(0, '/');
    }
    format!("file://{}", utf8_percent_encode(&path, FILE_URL_PATH_SET))
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

const FILE_URL_PATH_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'%')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'[')
    .add(b']')
    .add(b'^')
    .add(b'`')
    .add(b'{')
    .add(b'|')
    .add(b'}');

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

    #[test]
    fn repo_file_url_opens_existing_repo_file() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("docs").join("index page#v?.html");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, "<html></html>").unwrap();

        let url = repo_file_url(tmp.path().to_str().unwrap(), "docs/index page#v?.html")
            .expect("html file should be openable");

        assert!(url.starts_with("file:///"), "{url}");
        assert!(url.ends_with("/docs/index%20page%23v%3F.html"), "{url}");
    }

    #[test]
    fn repo_file_url_rejects_dirs_missing_files_and_escapes() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir(tmp.path().join("docs")).unwrap();
        let outside = tempfile::NamedTempFile::new().unwrap();

        assert_eq!(repo_file_url(tmp.path().to_str().unwrap(), "docs"), None);
        assert_eq!(
            repo_file_url(tmp.path().to_str().unwrap(), "missing.html"),
            None
        );
        assert_eq!(
            repo_file_url(
                tmp.path().to_str().unwrap(),
                outside.path().to_str().unwrap()
            ),
            None
        );
        assert_eq!(
            repo_file_url(tmp.path().to_str().unwrap(), "../outside.html"),
            None
        );
    }
}
