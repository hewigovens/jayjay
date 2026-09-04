use std::process::Command;

mod desktop_entry;

pub use desktop_entry::default_text_editor;

use super::super::launcher::{detach_stdio, env_command, resolved_command};
use super::super::terminal::{Terminal, shell_line};

pub const EDITOR_OPTIONS: &[(&str, &str)] = &[
    ("system", "System Editor"),
    ("vscode", "Visual Studio Code"),
    ("vscodium", "VSCodium"),
    ("cursor", "Cursor"),
    ("zed", "Zed"),
    ("sublime", "Sublime Text"),
    ("gnome-text-editor", "GNOME Text Editor"),
    ("kate", "Kate"),
    ("vim", "Vim"),
    ("nvim", "Neovim"),
    ("custom", "Custom"),
];

pub const TERMINAL_OPTIONS: &[(&str, &str)] = &[
    ("terminal", "System Terminal"),
    ("gnome-terminal", "GNOME Terminal"),
    ("konsole", "Konsole"),
    ("lxterminal", "LXTerminal"),
    ("alacritty", "Alacritty"),
    ("kitty", "Kitty"),
    ("ghostty", "Ghostty"),
    ("custom", "Custom"),
];

pub fn spawn_terminal(term: Terminal, cwd: &str, command: Option<&str>, custom: &str) -> bool {
    let line = shell_line(cwd, command);
    let payload = format!("{line}; exec $SHELL");

    match term {
        Terminal::SystemDefault | Terminal::ITerm => spawn_default(&payload),
        Terminal::GnomeTerminal => spawn_with_args(
            "gnome-terminal",
            vec![
                "--working-directory".to_owned(),
                cwd.to_owned(),
                "--".to_owned(),
                "bash".to_owned(),
                "-lc".to_owned(),
                payload,
            ],
        ),
        Terminal::Konsole => spawn_with_args(
            "konsole",
            vec![
                "--workdir".to_owned(),
                cwd.to_owned(),
                "-e".to_owned(),
                "bash".to_owned(),
                "-lc".to_owned(),
                payload,
            ],
        ),
        Terminal::LxTerminal => spawn_with_args(
            "lxterminal",
            vec![
                format!("--working-directory={cwd}"),
                "-e".to_owned(),
                "bash".to_owned(),
                "-lc".to_owned(),
                payload,
            ],
        ),
        Terminal::Alacritty => spawn_with_args(
            "alacritty",
            vec![
                "--working-directory".to_owned(),
                cwd.to_owned(),
                "-e".to_owned(),
                "bash".to_owned(),
                "-lc".to_owned(),
                payload,
            ],
        ),
        Terminal::Kitty => spawn_with_args(
            "kitty",
            vec![
                "--directory".to_owned(),
                cwd.to_owned(),
                "bash".to_owned(),
                "-lc".to_owned(),
                payload,
            ],
        ),
        Terminal::Ghostty => spawn_with_args(
            "ghostty",
            vec![
                format!("--working-directory={cwd}"),
                "-e".to_owned(),
                "bash".to_owned(),
                "-lc".to_owned(),
                payload,
            ],
        ),
        Terminal::Custom => {
            if custom.is_empty() {
                spawn_default(&payload)
            } else {
                spawn_generic(custom, &payload)
            }
        }
    }
}

fn spawn_default(payload: &str) -> bool {
    env_command(&["TERMINAL"]).is_some_and(|terminal| spawn_generic(&terminal, payload))
        || spawn_with_args(
            "xdg-terminal-exec",
            vec!["bash".to_owned(), "-lc".to_owned(), payload.to_owned()],
        )
        || ["x-terminal-emulator", "xterm"]
            .into_iter()
            .any(|bin| spawn_generic(bin, payload))
}

fn spawn_generic(binary: &str, payload: &str) -> bool {
    spawn_with_args(
        binary,
        vec!["-e", "bash", "-lc", payload]
            .into_iter()
            .map(str::to_owned)
            .collect(),
    )
}

fn spawn_with_args(command: &str, args: Vec<String>) -> bool {
    let Some((path, mut command_args)) = resolved_command(command) else {
        return false;
    };
    command_args.extend(args);
    detach_stdio(Command::new(path).args(command_args))
        .spawn()
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::{EDITOR_OPTIONS, TERMINAL_OPTIONS};

    #[test]
    fn linux_tool_options_use_linux_apps() {
        assert!(!EDITOR_OPTIONS.iter().any(|(id, _)| *id == "xcode"));
        assert!(!TERMINAL_OPTIONS.iter().any(|(id, _)| *id == "iterm"));
        assert!(TERMINAL_OPTIONS.iter().any(|(id, _)| *id == "lxterminal"));
        assert!(EDITOR_OPTIONS.iter().any(|(id, _)| *id == "nvim"));
        assert!(EDITOR_OPTIONS.iter().any(|(id, _)| *id == "system"));
    }
}
