#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub(super) enum Terminal {
    SystemDefault,
    ITerm,
    Ghostty,
    GnomeTerminal,
    Konsole,
    LxTerminal,
    Alacritty,
    Kitty,
    Custom,
}

impl Terminal {
    pub(super) fn from_id(id: &str) -> Option<Self> {
        Some(match id {
            "terminal" => Self::SystemDefault,
            "iterm" => Self::ITerm,
            "ghostty" => Self::Ghostty,
            "gnome-terminal" => Self::GnomeTerminal,
            "konsole" => Self::Konsole,
            "lxterminal" => Self::LxTerminal,
            "alacritty" => Self::Alacritty,
            "kitty" => Self::Kitty,
            "custom" => Self::Custom,
            _ => return None,
        })
    }
}

/// `cd '<cwd>' && <command>`, or just `cd '<cwd>'` if no command.
pub(super) fn shell_line(cwd: &str, command: Option<&str>) -> String {
    let cd = format!("cd '{}'", escape_single_quotes(cwd));
    match command {
        Some(c) => format!("{cd} && {c}"),
        None => cd,
    }
}

/// Make `s` safe inside a single-quoted shell word via the `'\''` idiom.
pub(super) fn escape_single_quotes(s: &str) -> String {
    s.replace('\'', "'\\''")
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
