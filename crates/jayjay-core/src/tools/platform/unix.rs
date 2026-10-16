pub fn command_words(command: &str) -> Option<Vec<String>> {
    shell_words::split(command).ok()
}

pub fn terminal_line(argv: &[String]) -> Option<String> {
    Some(shell_words::join(argv))
}

pub fn run_terminal_tool(_arguments: &[String]) -> Option<i32> {
    None
}

/// `cd '<cwd>' && <command>`, or just `cd '<cwd>'` if no command.
pub fn shell_line(cwd: &str, command: Option<&str>) -> String {
    let cd = format!("cd '{}'", escape_single_quotes(cwd));
    match command {
        Some(c) => format!("{cd} && {c}"),
        None => cd,
    }
}

/// Make `s` safe inside a single-quoted shell word via the `'\''` idiom.
fn escape_single_quotes(s: &str) -> String {
    s.replace('\'', "'\\''")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_line_keeps_hostile_paths_as_one_quoted_word() {
        let argv = [
            "vim".to_owned(),
            "/repo/a'$(touch /tmp/pwned)'.rs".to_owned(),
        ];
        let line = terminal_line(&argv).unwrap();
        assert_eq!(shell_words::split(&line).unwrap(), argv);
        assert!(line.contains("a'\\''$(touch /tmp/pwned)'\\''.rs"));
    }

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
