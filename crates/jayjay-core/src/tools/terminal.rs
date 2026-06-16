use std::process::Command;

#[cfg(not(target_os = "macos"))]
use crate::repo::find_existing_binary;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub(super) enum Terminal {
    AppleTerminal,
    ITerm,
    Ghostty,
    Custom,
}

impl Terminal {
    pub(super) fn from_id(id: &str) -> Option<Self> {
        Some(match id {
            "terminal" => Self::AppleTerminal,
            "iterm" => Self::ITerm,
            "ghostty" => Self::Ghostty,
            "custom" => Self::Custom,
            _ => return None,
        })
    }
}

#[cfg(target_os = "macos")]
pub(super) fn spawn_terminal(
    term: Terminal,
    cwd: &str,
    command: Option<&str>,
    custom: &str,
) -> bool {
    match term {
        Terminal::AppleTerminal | Terminal::Custom => {
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
    }
}

#[cfg(not(target_os = "macos"))]
pub(super) fn spawn_terminal(
    _term: Terminal,
    cwd: &str,
    command: Option<&str>,
    _custom: &str,
) -> bool {
    let line = shell_line(cwd, command);
    let payload = format!("{line}; exec $SHELL");
    for bin in ["x-terminal-emulator", "xterm"] {
        if find_existing_binary(bin).is_some() {
            return Command::new(bin)
                .args(["-e", "bash", "-lc", &payload])
                .spawn()
                .is_ok();
        }
    }
    false
}

/// `cd '<cwd>' && <command>` — or just `cd '<cwd>'` if no command.
fn shell_line(cwd: &str, command: Option<&str>) -> String {
    let cd = format!("cd '{}'", escape_single_quotes(cwd));
    match command {
        Some(c) => format!("{cd} && {c}"),
        None => cd,
    }
}

/// Ghostty: `--working-directory=<cwd>` for cwd; `-e bash -c <cmd>` for the
/// command path. Ghostty's `-e` consumes argv literally, so quoted paths in
/// `vim '<file>'` only parse correctly when bash gets them first.
#[cfg(target_os = "macos")]
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
    cmd.spawn().is_ok()
}

#[cfg(target_os = "macos")]
fn run_applescript(source: &str) -> bool {
    Command::new("/usr/bin/osascript")
        .args(["-e", source])
        .spawn()
        .is_ok()
}

/// Use bundle id rather than `"iTerm2"` — the actual .app on disk is
/// `iTerm.app`, so AppleScript's name-based lookup misses.
#[cfg(target_os = "macos")]
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

/// Make `s` safe inside a single-quoted shell word via the `'\''` idiom.
/// Used for any path/dir spliced into a shell line.
pub(super) fn escape_single_quotes(s: &str) -> String {
    s.replace('\'', "'\\''")
}

#[cfg(target_os = "macos")]
fn escape_double_quotes(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_line_without_command_just_cds() {
        assert_eq!(shell_line("/my repo", None), "cd '/my repo'");
    }

    #[test]
    fn shell_line_escapes_cwd_apostrophe() {
        assert_eq!(shell_line("/my'repo", None), "cd '/my'\\''repo'");
    }

    #[test]
    fn shell_line_appends_command_after_cd() {
        assert_eq!(
            shell_line("/repo", Some("vim '/repo/main.rs'")),
            "cd '/repo' && vim '/repo/main.rs'"
        );
    }

    #[test]
    fn shell_line_keeps_quoted_injection_payload_inert() {
        // Mirrors what open_in_editor builds for a crafted filename: once the
        // path is single-quote escaped, the payload cannot break out of the
        // command's quoted word even after splicing into `cd ... && ...`.
        let cmd = "vim '/repo/a'\\''$(touch /tmp/pwned)'\\''.rs'";
        let line = shell_line("/repo", Some(cmd));
        assert_eq!(line, format!("cd '/repo' && {cmd}"));
        // Outside the wrapping quotes there is no bare `$(` that a shell would
        // expand; the payload only appears inside escaped single quotes.
        assert!(!line.contains("&& $("));
        assert!(line.contains("'\\''$(touch /tmp/pwned)'\\''"));
    }
}
