use std::path::Path;
use std::process::{Command, Stdio};

use crate::repo::{find_existing_binary, subprocess_command};
use crate::ssh::RemoteLocation;

use super::config::ToolsConfig;
use super::editor::Editor;
use super::platform;
use super::terminal::Terminal;

/// Open `file_path` (relative to `repo_path`, or absolute) in the user's editor, at the 1-based `line` when the editor supports it.
/// Returns false when the binary is missing or spawn fails.
pub fn open_in_editor(
    repo_path: &str,
    file_path: &str,
    line: Option<u32>,
    cfg: &ToolsConfig,
) -> bool {
    if RemoteLocation::from_path(repo_path).is_some() {
        return false;
    }
    let editor = Editor::from_id(&cfg.external_editor).unwrap_or(Editor::Custom);
    let absolute = absolutize(repo_path, file_path);

    let launch = match editor {
        Editor::SystemDefault => match env_command(&["VISUAL", "EDITOR"]) {
            Some(cmd) => {
                let in_terminal = env_editor_needs_terminal(&cmd);
                EditorLaunch::from_command(&cmd, &[], vec![absolute], in_terminal)
            }
            None => match platform::default_text_editor(&absolute) {
                Some(launch) => Some(launch),
                None => return platform::open_path(&absolute),
            },
        },
        Editor::Custom => EditorLaunch::from_command(
            &cfg.custom_editor_command,
            &[],
            vec![absolute],
            classify_editor(&cfg.custom_editor_command) == EditorKind::Terminal,
        ),
        named => EditorLaunch::from_command(
            named.command(),
            named.launch_args(),
            named.target_args(&absolute, line),
            classify_editor(named.command()) == EditorKind::Terminal,
        ),
    };
    launch.is_some_and(|launch| launch.spawn(repo_path, cfg))
}

pub fn editor_opens_at_line(external_editor: &str) -> bool {
    Editor::from_id(external_editor).is_some_and(Editor::opens_at_line)
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
        target: Vec<String>,
        in_terminal: bool,
    ) -> Option<Self> {
        let (binary, args) = resolved_command(cmd)?;
        let mut argv = vec![binary];
        argv.extend(launch_args.iter().map(|arg| (*arg).to_owned()));
        argv.extend(args);
        argv.extend(target);
        Some(Self { argv, in_terminal })
    }

    fn spawn(self, repo_path: &str, cfg: &ToolsConfig) -> bool {
        if self.in_terminal {
            return platform::terminal_line(&self.argv)
                .is_some_and(|line| open_in_terminal(repo_path, Some(&line), cfg));
        }
        detach_stdio(subprocess_command(&self.argv[0]).args(&self.argv[1..]))
            .spawn()
            .is_ok()
    }
}

/// `$VISUAL`/`$EDITOR` are terminal-context commands (git runs them inside a terminal), so outside macOS only a known GUI editor launches directly.
fn env_editor_needs_terminal(cmd: &str) -> bool {
    match classify_editor(cmd) {
        EditorKind::Terminal => true,
        EditorKind::Gui => false,
        EditorKind::Unknown => platform::UNKNOWN_EDITOR_NEEDS_TERMINAL,
    }
}

/// External tools inherit JayJay's stdio otherwise, and their chatter (browsers, terminals) lands in its log.
pub fn detach_stdio(command: &mut Command) -> &mut Command {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
}

pub(super) fn env_command(names: &[&str]) -> Option<String> {
    names.iter().find_map(|name| {
        std::env::var(name)
            .ok()
            .filter(|value| !value.trim().is_empty())
    })
}

pub(super) fn resolved_command(command: &str) -> Option<(String, Vec<String>)> {
    let mut words = platform::command_words(command)?.into_iter();
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
    let Some(words) = platform::command_words(command) else {
        return EditorKind::Unknown;
    };
    let Some(binary) = words
        .first()
        .and_then(|binary| Path::new(binary).file_stem()?.to_str())
        .map(str::to_ascii_lowercase)
    else {
        return EditorKind::Unknown;
    };
    match binary.as_str() {
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
        | "geany" | "gvim" | "notepad" | "notepad++" => EditorKind::Gui,
        _ => EditorKind::Unknown,
    }
}

/// Open the user's terminal at `repo_path`. If `command` is set, the terminal
/// runs it after `cd`-ing into `repo_path`.
pub fn open_in_terminal(repo_path: &str, command: Option<&str>, cfg: &ToolsConfig) -> bool {
    if RemoteLocation::from_path(repo_path).is_some() {
        return false;
    }
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
    fn custom_editor_command_keeps_arguments_separate() {
        let exe = std::env::current_exe().unwrap();
        let command = format!("\"{}\" -c \"exit 0\"", exe.display());
        let (binary, args) = resolved_command(&command).expect("test binary");
        assert_eq!(Path::new(&binary), exe);
        assert_eq!(args, ["-c", "exit 0"]);
    }

    #[test]
    fn editor_classification_covers_common_editors() {
        assert_eq!(classify_editor("nvim --clean"), EditorKind::Terminal);
        assert_eq!(classify_editor("/usr/bin/vim -f"), EditorKind::Terminal);
        assert_eq!(
            classify_editor("/opt/Neovim/NVIM.exe"),
            EditorKind::Terminal
        );
        #[cfg(windows)]
        assert_eq!(
            classify_editor(r#""C:\Program Files\Neovim\nvim.exe" --clean"#),
            EditorKind::Terminal
        );
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
            platform::UNKNOWN_EDITOR_NEEDS_TERMINAL
        );
    }
}
