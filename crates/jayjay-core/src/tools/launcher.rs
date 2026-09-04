use std::path::Path;

use crate::repo::{find_existing_binary, subprocess_command};

use super::config::ToolsConfig;
use super::editor::Editor;
use super::platform;
use super::terminal::{Terminal, escape_single_quotes};

/// Open `file_path` (relative to `repo_path`, or absolute) in the user's editor.
/// Returns false when the binary is missing or spawn fails.
pub fn open_in_editor(repo_path: &str, file_path: &str, cfg: &ToolsConfig) -> bool {
    let editor = Editor::from_id(&cfg.external_editor);
    let absolute = absolutize(repo_path, file_path);

    let cmd = match editor {
        Some(Editor::SystemDefault) => system_editor_command(),
        Some(e) if e != Editor::Custom => e.command().to_owned(),
        _ => cfg.custom_editor_command.clone(),
    };
    if cmd.is_empty() {
        return false;
    }
    if command_is_terminal_editor(&cmd) {
        return open_in_terminal(
            repo_path,
            Some(&editor_terminal_command(&cmd, &absolute)),
            cfg,
        );
    }
    let Some((binary, args)) = resolved_command(&cmd) else {
        return false;
    };
    let mut launch_args: Vec<_> = editor
        .map(Editor::launch_args)
        .unwrap_or_default()
        .iter()
        .map(|arg| (*arg).to_owned())
        .collect();
    launch_args.extend(args);
    subprocess_command(&binary)
        .args(launch_args)
        .arg(&absolute)
        .spawn()
        .is_ok()
}

fn system_editor_command() -> String {
    env_command(&["VISUAL", "EDITOR"]).unwrap_or_else(|| "xdg-open".to_owned())
}

pub(super) fn env_command(names: &[&str]) -> Option<String> {
    names.iter().find_map(|name| {
        std::env::var(name)
            .ok()
            .filter(|value| !value.trim().is_empty())
    })
}

pub(super) fn resolved_command(command: &str) -> Option<(String, Vec<String>)> {
    let mut words = shell_words::split(command).ok()?.into_iter();
    let binary = find_existing_binary(&words.next()?)?;
    Some((binary, words.collect()))
}

fn command_is_terminal_editor(command: &str) -> bool {
    let Ok(words) = shell_words::split(command) else {
        return false;
    };
    let Some(binary) = words
        .first()
        .and_then(|binary| Path::new(binary).file_name()?.to_str())
    else {
        return false;
    };
    match binary {
        "vi" | "vim" | "nvim" | "nano" | "micro" | "hx" | "helix" | "kak" => true,
        "emacs" => words
            .iter()
            .any(|arg| arg == "-nw" || arg == "--no-window-system"),
        _ => false,
    }
}

/// Open the user's terminal at `repo_path`. If `command` is set, the terminal
/// runs it after `cd`-ing into `repo_path`.
pub fn open_in_terminal(repo_path: &str, command: Option<&str>, cfg: &ToolsConfig) -> bool {
    let term = Terminal::from_id(&cfg.terminal).unwrap_or(Terminal::SystemDefault);
    platform::spawn_terminal(term, repo_path, command, &cfg.custom_terminal_command)
}

/// Build the `<editor> '<path>'` shell command for a terminal editor.
/// `path` is attacker-controlled (any cloned repo), so single-quote escape it
/// before splicing into the shell line the terminal executes.
fn editor_terminal_command(cmd: &str, path: &str) -> String {
    format!("{cmd} '{}'", escape_single_quotes(path))
}

fn absolutize(repo_path: &str, file_path: &str) -> String {
    let path = Path::new(file_path);
    if path.is_absolute() {
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
        // Crafted filename in a cloned repo: the closing quote + payload must not escape the single-quoted word.
        let cmd = editor_terminal_command("vim", "/repo/a'$(touch /tmp/pwned)'.rs");
        // The only unescaped single quotes are the wrapping pair we added.
        assert!(cmd.starts_with("vim '"));
        assert!(cmd.ends_with('\''));
        // The payload's quotes are escaped via the '\'' idiom, so `$(...)` stays literal inside the quoted word rather than running as a command.
        assert!(cmd.contains("a'\\''$(touch /tmp/pwned)'\\''.rs"));
    }

    #[test]
    fn custom_editor_command_keeps_arguments_separate() {
        let exe = std::env::current_exe().unwrap();
        let command = format!("{} -c 'exit 0'", shell_words::quote(&exe.to_string_lossy()));
        let (binary, args) = resolved_command(&command).expect("test binary");
        assert_eq!(Path::new(&binary), exe);
        assert_eq!(args, ["-c", "exit 0"]);
    }

    #[test]
    fn terminal_editor_detection_covers_common_editors() {
        assert!(command_is_terminal_editor("nvim --clean"));
        assert!(command_is_terminal_editor("/usr/bin/vim -f"));
        assert!(command_is_terminal_editor("nano"));
        assert!(command_is_terminal_editor("emacs -nw"));
        assert!(!command_is_terminal_editor("emacs"));
        assert!(!command_is_terminal_editor("code --wait"));
    }
}
