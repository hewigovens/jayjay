use std::path::Path;

use crate::repo::{find_existing_binary, subprocess_command};

use super::config::ToolsConfig;
use super::editor::Editor;
use super::platform;
use super::terminal::Terminal;

/// Open `file_path` (relative to `repo_path`, or absolute) in the user's editor.
/// Returns false when the binary is missing or spawn fails.
pub fn open_in_editor(repo_path: &str, file_path: &str, cfg: &ToolsConfig) -> bool {
    let editor = Editor::from_id(&cfg.external_editor);
    let absolute = absolutize(repo_path, file_path);

    let launch = match editor {
        Some(Editor::SystemDefault) => {
            system_editor_launch(env_command(&["VISUAL", "EDITOR"]), &absolute)
        }
        other => {
            let cmd = match other {
                Some(e) if e != Editor::Custom => e.command().to_owned(),
                _ => cfg.custom_editor_command.clone(),
            };
            let launch_args = other.map(Editor::launch_args).unwrap_or_default();
            let in_terminal = classify_editor(&cmd) == EditorKind::Terminal;
            EditorLaunch::from_command(&cmd, launch_args, &absolute, in_terminal)
        }
    };
    launch.is_some_and(|launch| launch.spawn(repo_path, cfg))
}

/// `argv[0]` is the resolved executable and the target path is already included.
pub(super) struct EditorLaunch {
    pub(super) argv: Vec<String>,
    pub(super) in_terminal: bool,
}

impl EditorLaunch {
    fn from_command(
        cmd: &str,
        launch_args: &[&str],
        path: &str,
        in_terminal: bool,
    ) -> Option<Self> {
        let (binary, args) = resolved_command(cmd)?;
        let mut argv = vec![binary];
        argv.extend(launch_args.iter().map(|arg| (*arg).to_owned()));
        argv.extend(args);
        argv.push(path.to_owned());
        Some(Self { argv, in_terminal })
    }

    fn terminal_line(&self) -> String {
        shell_words::join(&self.argv)
    }

    fn spawn(self, repo_path: &str, cfg: &ToolsConfig) -> bool {
        if self.in_terminal {
            return open_in_terminal(repo_path, Some(&self.terminal_line()), cfg);
        }
        subprocess_command(&self.argv[0])
            .args(&self.argv[1..])
            .spawn()
            .is_ok()
    }
}

/// `$VISUAL`/`$EDITOR` are terminal-context commands (git runs them inside a terminal), so on Linux only a known GUI editor launches directly.
fn system_editor_launch(env_editor: Option<String>, path: &str) -> Option<EditorLaunch> {
    match env_editor {
        Some(cmd) => EditorLaunch::from_command(&cmd, &[], path, env_editor_needs_terminal(&cmd)),
        None => default_text_editor(path)
            .or_else(|| EditorLaunch::from_command("xdg-open", &[], path, false)),
    }
}

fn env_editor_needs_terminal(cmd: &str) -> bool {
    match classify_editor(cmd) {
        EditorKind::Terminal => true,
        EditorKind::Gui => false,
        EditorKind::Unknown => cfg!(target_os = "linux"),
    }
}

#[cfg(target_os = "macos")]
fn default_text_editor(_path: &str) -> Option<EditorLaunch> {
    None
}

#[cfg(not(target_os = "macos"))]
use platform::default_text_editor;

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

#[derive(Debug, PartialEq, Eq)]
enum EditorKind {
    Terminal,
    Gui,
    Unknown,
}

fn classify_editor(command: &str) -> EditorKind {
    let Ok(words) = shell_words::split(command) else {
        return EditorKind::Unknown;
    };
    let Some(binary) = words
        .first()
        .and_then(|binary| Path::new(binary).file_name()?.to_str())
    else {
        return EditorKind::Unknown;
    };
    match binary {
        "vi" | "vim" | "nvim" | "nano" | "micro" | "hx" | "helix" | "kak" => EditorKind::Terminal,
        "emacs" | "emacsclient" => {
            if words
                .iter()
                .any(|arg| matches!(arg.as_str(), "-nw" | "--no-window-system" | "-t" | "--tty"))
            {
                EditorKind::Terminal
            } else {
                EditorKind::Gui
            }
        }
        "code" | "code-insiders" | "codium" | "cursor" | "zed" | "subl" | "sublime_text"
        | "gnome-text-editor" | "gedit" | "kate" | "kwrite" | "mousepad" | "xed" | "pluma"
        | "geany" | "gvim" => EditorKind::Gui,
        _ => EditorKind::Unknown,
    }
}

/// Open the user's terminal at `repo_path`. If `command` is set, the terminal
/// runs it after `cd`-ing into `repo_path`.
pub fn open_in_terminal(repo_path: &str, command: Option<&str>, cfg: &ToolsConfig) -> bool {
    let term = Terminal::from_id(&cfg.terminal).unwrap_or(Terminal::SystemDefault);
    platform::spawn_terminal(term, repo_path, command, &cfg.custom_terminal_command)
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
    fn terminal_line_keeps_hostile_paths_as_one_quoted_word() {
        let launch = EditorLaunch {
            argv: vec![
                "vim".to_owned(),
                "/repo/a'$(touch /tmp/pwned)'.rs".to_owned(),
            ],
            in_terminal: true,
        };
        let line = launch.terminal_line();
        assert_eq!(shell_words::split(&line).unwrap(), launch.argv);
        assert!(line.contains("a'\\''$(touch /tmp/pwned)'\\''.rs"));
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
    fn editor_classification_covers_common_editors() {
        assert_eq!(classify_editor("nvim --clean"), EditorKind::Terminal);
        assert_eq!(classify_editor("/usr/bin/vim -f"), EditorKind::Terminal);
        assert_eq!(classify_editor("emacs -nw"), EditorKind::Terminal);
        assert_eq!(classify_editor("emacsclient -t"), EditorKind::Terminal);
        assert_eq!(classify_editor("emacs"), EditorKind::Gui);
        assert_eq!(classify_editor("code --wait"), EditorKind::Gui);
        assert_eq!(
            classify_editor("omarchy-launch-editor --inline"),
            EditorKind::Unknown
        );
    }

    #[test]
    fn env_editor_runs_in_terminal_unless_gui() {
        assert!(!env_editor_needs_terminal("code --wait"));
        assert!(env_editor_needs_terminal("nvim"));
        assert_eq!(
            env_editor_needs_terminal("omarchy-launch-editor --inline"),
            cfg!(target_os = "linux")
        );
    }
}
