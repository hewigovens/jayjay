use jayjay_core::CoreError;

use super::review::ReviewCommand;

/// Must run before any GPUI/window init; `None` means "not a CLI command" and the caller falls through to normal window init.
pub fn run_and_exit_if_needed(arguments: &[String]) -> Option<i32> {
    let outcome = dispatch(arguments)?;
    if outcome.is_error {
        eprint!("{}", outcome.message);
    } else {
        print!("{}", outcome.message);
    }
    Some(outcome.exit_code)
}

struct CommandOutcome {
    exit_code: i32,
    message: String,
    is_error: bool,
}

impl CommandOutcome {
    fn ok(message: String) -> Self {
        Self {
            exit_code: 0,
            message,
            is_error: false,
        }
    }

    fn err(message: String) -> Self {
        Self {
            exit_code: 1,
            message,
            is_error: true,
        }
    }
}

fn dispatch(arguments: &[String]) -> Option<CommandOutcome> {
    let first = arguments.first()?;
    if first == "--version" || first == "-v" {
        return Some(CommandOutcome::ok(format!(
            "jayjay {}\n",
            env!("CARGO_PKG_VERSION")
        )));
    }
    if first != "review" {
        return None;
    }

    Some(match ReviewCommand::parse(&arguments[1..]) {
        Ok(command) => match command.run() {
            Ok(output) => CommandOutcome::ok(output),
            Err(error) => CommandOutcome::err(format!("error: {}\n", describe_error(&error))),
        },
        Err(message) => CommandOutcome::err(format!("error: {message}\n")),
    })
}

/// Text must match the SwiftUI shell's reconstructed error strings exactly, not `CoreError`'s own `Display` impl, to keep error output identical across shells.
fn describe_error(error: &CoreError) -> String {
    match error {
        CoreError::RepoNotFound { path } => format!("repository not found: {path}"),
        CoreError::RevNotFound { rev } => format!("revision not found: {rev}"),
        CoreError::DiffSelectionStale { path } => {
            format!("{path}: file changed since the diff was rendered — refresh and retry")
        }
        CoreError::Review { message }
        | CoreError::Diff { message }
        | CoreError::Internal { message } => message.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn no_args_falls_through_to_gui_init() {
        assert!(dispatch(&[]).is_none());
    }

    #[test]
    fn plain_repo_path_falls_through_to_gui_init() {
        assert!(dispatch(&args(&["/some/repo"])).is_none());
    }

    #[test]
    fn version_flags_print_version_and_exit_zero() {
        for flag in ["--version", "-v"] {
            let outcome = dispatch(&args(&[flag])).expect("handled");
            assert_eq!(outcome.exit_code, 0);
            assert!(!outcome.is_error);
            assert_eq!(
                outcome.message,
                format!("jayjay {}\n", env!("CARGO_PKG_VERSION"))
            );
        }
    }

    #[test]
    fn missing_review_subcommand_exits_one_with_usage_error() {
        let outcome = dispatch(&args(&["review"])).expect("handled");
        assert_eq!(outcome.exit_code, 1);
        assert!(outcome.is_error);
        assert_eq!(outcome.message, "error: missing review subcommand\n");
    }

    #[test]
    fn unknown_review_subcommand_exits_one_with_usage_error() {
        let outcome = dispatch(&args(&["review", "bogus"])).expect("handled");
        assert_eq!(outcome.exit_code, 1);
        assert_eq!(outcome.message, "error: unknown review subcommand: bogus\n");
    }

    #[test]
    fn describe_error_matches_swift_reconstructed_text() {
        assert_eq!(
            describe_error(&CoreError::RepoNotFound {
                path: "/tmp/x".to_string()
            }),
            "repository not found: /tmp/x"
        );
        assert_eq!(
            describe_error(&CoreError::RevNotFound {
                rev: "abc".to_string()
            }),
            "revision not found: abc"
        );
        assert_eq!(
            describe_error(&CoreError::Review {
                message: "not a changed line".to_string()
            }),
            "not a changed line"
        );
    }
}
