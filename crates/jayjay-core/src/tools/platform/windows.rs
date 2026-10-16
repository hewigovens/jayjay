pub mod shell;
mod terminal_tool;

use std::path::Path;

pub use shell::command_words;
pub use terminal_tool::{run_terminal_tool, terminal_line};

use super::super::launcher::{EditorLaunch, detach_stdio, resolved_command};
use super::super::terminal::Terminal;
use super::powershell;
use crate::repo::platform::system32;
use crate::repo::{find_existing_binary, subprocess_command};

pub const DEFAULT_EDITOR: &str = "system";
pub const UNKNOWN_EDITOR_NEEDS_TERMINAL: bool = true;

pub const EDITOR_OPTIONS: &[(&str, &str)] = &[
    ("system", "System Editor"),
    ("vscode", "Visual Studio Code"),
    ("vscodium", "VSCodium"),
    ("cursor", "Cursor"),
    ("zed", "Zed"),
    ("sublime", "Sublime Text"),
    ("vim", "Vim"),
    ("nvim", "Neovim"),
    ("custom", "Custom"),
];

pub const TERMINAL_OPTIONS: &[(&str, &str)] = &[
    ("terminal", "Windows Terminal"),
    ("powershell", "Windows PowerShell"),
    ("custom", "Custom"),
];

pub fn default_text_editor(_path: &str) -> Option<EditorLaunch> {
    None
}

/// The default `open` verb would run scripts from the repository.
pub fn open_path(path: &str) -> bool {
    let path = path.to_owned();
    shell::detached(move || {
        shell::edit(&path)
            || detach_stdio(subprocess_command(system32().join("notepad.exe")).arg(&path))
                .spawn()
                .is_ok()
    })
}

pub fn spawn_terminal(term: Terminal, cwd: &str, command: Option<&str>, custom: &str) -> bool {
    let powershell = system32().join(r"WindowsPowerShell\v1.0\powershell.exe");
    let payload = powershell::terminal_payload(cwd, command);
    let args = [
        "-NoLogo",
        "-NoProfile",
        "-NoExit",
        "-EncodedCommand",
        &payload,
    ];
    let host = match term {
        Terminal::Custom if !custom.trim().is_empty() => {
            return resolved_command(custom).is_some_and(|host| spawn_in(host, &powershell, &args));
        }
        Terminal::PowerShell => None,
        _ => find_existing_binary("wt")
            .map(|wt| (wt, ["-w", "new", "new-tab"].map(str::to_owned).to_vec())),
    };
    host.is_some_and(|host| spawn_in(host, &powershell, &args))
        || shell::launch(powershell, args.join(" "))
}

fn spawn_in((host, host_args): (String, Vec<String>), powershell: &Path, args: &[&str]) -> bool {
    detach_stdio(
        subprocess_command(host)
            .args(host_args)
            .arg(powershell)
            .args(args),
    )
    .spawn()
    .is_ok()
}
