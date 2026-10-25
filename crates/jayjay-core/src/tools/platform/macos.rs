use std::process::Command;

use super::super::launcher::{EditorLaunch, detach_stdio};
use super::super::terminal::Terminal;
use super::unix::shell_line;

pub const DEFAULT_EDITOR: &str = "vscode";
pub const UNKNOWN_EDITOR_NEEDS_TERMINAL: bool = false;

pub const EDITOR_OPTIONS: &[(&str, &str)] = &[
    ("vscode", "Visual Studio Code"),
    ("vscodium", "VSCodium"),
    ("cursor", "Cursor"),
    ("zed", "Zed"),
    ("xcode", "Xcode"),
    ("vim", "Vim"),
    ("custom", "Custom"),
];

pub const TERMINAL_OPTIONS: &[(&str, &str)] = &[
    ("terminal", "Terminal"),
    ("iterm", "iTerm2"),
    ("ghostty", "Ghostty"),
    ("rex", "Rex"),
    ("custom", "Custom"),
];

pub fn default_text_editor(_path: &str) -> Option<EditorLaunch> {
    None
}

/// The file's own association could run a `.command` or `.app`.
pub fn open_path(path: &str) -> bool {
    detach_stdio(Command::new("/usr/bin/open").arg("-t").arg(path))
        .spawn()
        .is_ok()
}

pub fn spawn_terminal(term: Terminal, cwd: &str, command: Option<&str>, custom: &str) -> bool {
    match term {
        Terminal::SystemDefault | Terminal::Custom => {
            let app = if matches!(term, Terminal::Custom) && !custom.is_empty() {
                custom.to_owned()
            } else {
                "Terminal".to_owned()
            };
            run_applescript(&format!(
                "tell application \"{}\" to do script \"{}\"",
                escape_double_quotes(&app),
                escape_double_quotes(&shell_line(cwd, command)),
            ))
        }
        Terminal::ITerm => run_applescript(&iterm_script(&shell_line(cwd, command))),
        Terminal::Ghostty => spawn_ghostty(cwd, command),
        Terminal::Rex if command.is_none() => open_rex(cwd),
        Terminal::Rex
        | Terminal::PowerShell
        | Terminal::GnomeTerminal
        | Terminal::Konsole
        | Terminal::LxTerminal
        | Terminal::Alacritty
        | Terminal::Kitty => run_applescript(&format!(
            "tell application \"Terminal\" to do script \"{}\"",
            escape_double_quotes(&shell_line(cwd, command)),
        )),
    }
}

/// Ghostty consumes `-e` argv literally; terminal-editor commands need shell parsing first.
fn spawn_ghostty(cwd: &str, command: Option<&str>) -> bool {
    let mut cmd = Command::new("/usr/bin/open");
    cmd.args([
        "-na",
        "Ghostty.app",
        "--args",
        &format!("--working-directory={cwd}"),
    ]);
    if let Some(c) = command {
        cmd.args(["-e", "bash", "-c", c]);
    }
    detach_stdio(&mut cmd).spawn().is_ok()
}

/// Rex's folder-open handler cannot launch a command.
fn open_rex(cwd: &str) -> bool {
    detach_stdio(Command::new("/usr/bin/open").args(["-b", "com.superlogical.rex", cwd]))
        .spawn()
        .is_ok()
}

fn run_applescript(source: &str) -> bool {
    detach_stdio(Command::new("/usr/bin/osascript").args(["-e", source]))
        .spawn()
        .is_ok()
}

/// Use the bundle id because the app bundle is named `iTerm.app`.
fn iterm_script(line: &str) -> String {
    let escaped = escape_double_quotes(line);
    format!(
        r#"tell application id "com.googlecode.iterm2"
    activate
    try
        tell current window
            create tab with default profile command "/bin/zsh"
            tell current session
                write text "{escaped}"
            end tell
        end tell
    on error
        create window with default profile command "/bin/zsh"
        tell current window
            tell current session
                write text "{escaped}"
            end tell
        end tell
    end try
end tell"#
    )
}

fn escape_double_quotes(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}
