use crate::{CliCommandOutcome, CoreError};

use super::review::ReviewCommand;

/// Runs app-owned headless commands before either desktop shell initializes its UI. `None` lets the caller continue normal app startup.
pub fn run_app_cli_command(arguments: &[String], version: &str) -> Option<CliCommandOutcome> {
    let first = arguments.first()?;
    if first == "--version" || first == "-v" {
        return Some(CliCommandOutcome::ok(format!("jayjay {version}\n")));
    }
    if first == crate::JAYJAY_CONFIG_COMMAND {
        return Some(if arguments.len() == 1 {
            CliCommandOutcome::ok(crate::JJ_TOOL_CONFIG)
        } else {
            CliCommandOutcome::err("error: usage: jayjay config\n")
        });
    }
    if first != crate::JAYJAY_REVIEW_COMMAND {
        return None;
    }

    Some(match ReviewCommand::parse(&arguments[1..]) {
        Ok(command) => match command.run() {
            Ok(output) => CliCommandOutcome::ok(output),
            Err(error) => CliCommandOutcome::err(format!("error: {}\n", describe_error(&error))),
        },
        Err(message) => CliCommandOutcome::err(format!("error: {message}\n")),
    })
}

// Stable CLI output, deliberately diverging from CoreError's Display in places (no "diff error:" prefix, capitalized "Canceled"); pinned by errors_use_shell_independent_text.
fn describe_error(error: &CoreError) -> String {
    match error {
        CoreError::RepoNotFound { path } => format!("repository not found: {path}"),
        CoreError::RevNotFound { rev } => format!("revision not found: {rev}"),
        CoreError::DiffSelectionStale { path } => {
            format!("{path}: file changed since the diff was rendered — refresh and retry")
        }
        CoreError::ConflictEditorStale { path } => {
            format!("{path}: conflict changed since the editor opened — refresh and retry")
        }
        CoreError::FileEditorStale { path } => {
            format!("{path}: file changed since the editor opened — refresh and retry")
        }
        CoreError::Review { message }
        | CoreError::Diff { message }
        | CoreError::Internal { message } => message.clone(),
        CoreError::Canceled => "Canceled".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::args;

    #[test]
    fn non_cli_arguments_fall_through_to_app_startup() {
        assert!(run_app_cli_command(&[], "1.2.3").is_none());
        assert!(run_app_cli_command(&args(&["/some/repo"]), "1.2.3").is_none());
    }

    #[test]
    fn version_flags_print_the_supplied_app_version() {
        for flag in ["--version", "-v"] {
            assert_eq!(
                run_app_cli_command(&args(&[flag]), "1.2.3"),
                Some(CliCommandOutcome::ok("jayjay 1.2.3\n"))
            );
        }
    }

    #[test]
    fn config_prints_paste_ready_jj_configuration() {
        assert_eq!(
            run_app_cli_command(&args(&["config"]), "1.2.3"),
            Some(CliCommandOutcome::ok(crate::JJ_TOOL_CONFIG))
        );
    }

    #[test]
    fn config_rejects_extra_arguments() {
        let outcome = run_app_cli_command(&args(&["config", "extra"]), "1.2.3").expect("handled");
        assert!(outcome.is_error());
        assert_eq!(
            outcome,
            CliCommandOutcome::err("error: usage: jayjay config\n")
        );
    }

    #[test]
    fn review_usage_errors_have_stable_output_and_exit_code() {
        assert_eq!(
            run_app_cli_command(&args(&["review"]), "1.2.3"),
            Some(CliCommandOutcome::err("error: missing review subcommand\n"))
        );
        assert_eq!(
            run_app_cli_command(&args(&["review", "bogus"]), "1.2.3"),
            Some(CliCommandOutcome::err(
                "error: unknown review subcommand: bogus\n"
            ))
        );
    }

    #[test]
    fn errors_use_shell_independent_text() {
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
        assert_eq!(
            describe_error(&CoreError::Diff {
                message: "left side vanished".to_string()
            }),
            "left side vanished"
        );
        assert_eq!(describe_error(&CoreError::Canceled), "Canceled");
    }
}
