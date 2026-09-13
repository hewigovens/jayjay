use clap::error::ErrorKind;
use clap::{Args, Parser, Subcommand};

use crate::{NoteSide, ReviewOutputFormat};

use super::review::ReviewCommand;

/// Whether the parsed review command line starts the workflow or prints static text.
#[derive(Debug)]
pub(super) enum ReviewOutcome {
    Run(ReviewCommand),
    Print(String),
}

pub(super) fn parse_review(arguments: &[String]) -> Result<ReviewOutcome, String> {
    let program = "jayjay review".to_owned();
    match ReviewCli::try_parse_from(std::iter::once(&program).chain(arguments)) {
        Ok(cli) => {
            let subcommand = cli
                .command
                .ok_or_else(|| "missing review subcommand".to_owned())?;
            Ok(ReviewOutcome::Run(subcommand.into()))
        }
        Err(error) => match error.kind() {
            ErrorKind::DisplayHelp => Ok(ReviewOutcome::Print(error.to_string())),
            _ => Err(error_message(&error.to_string())),
        },
    }
}

// clap renders `error: <message>\n\nUsage: ...`; keep the message (including argument names on following lines) but drop the usage and hint blocks.
fn error_message(error: &str) -> String {
    let message = error.strip_prefix("error: ").unwrap_or(error);
    message.split("\n\n").next().unwrap_or_default().to_owned()
}

#[derive(Parser)]
#[command(disable_version_flag = true)]
struct ReviewCli {
    #[command(subcommand)]
    command: Option<ReviewSubcommand>,
}

#[derive(Subcommand)]
enum ReviewSubcommand {
    /// List review notes
    Notes(NotesArgs),
    /// Resolve a review note
    ResolveNote(ResolveNoteArgs),
    /// Add a review note
    AddNote(AddNoteArgs),
    /// Show the review state of every file in the working-copy change
    Status(StatusArgs),
    /// Mark a file, or one change group, reviewed on behalf of an agent
    Mark(MarkArgs),
    /// Drop agent review marks, leaving a person's marks in place
    Unmark(UnmarkArgs),
}

impl From<ReviewSubcommand> for ReviewCommand {
    fn from(subcommand: ReviewSubcommand) -> Self {
        match subcommand {
            ReviewSubcommand::Notes(args) => Self::Notes {
                repo: args.repo,
                format: args.format.into(),
                include_resolved: args.include_resolved,
            },
            ReviewSubcommand::ResolveNote(args) => Self::ResolveNote {
                id: args.id,
                repo: args.repo,
            },
            ReviewSubcommand::AddNote(args) => Self::AddNote {
                repo: args.repo,
                file: args.file,
                line: args.line,
                side: args.side.into(),
                message: args.message,
            },
            ReviewSubcommand::Status(args) => Self::Status {
                repo: args.repo,
                format: args.format.into(),
            },
            ReviewSubcommand::Mark(args) => Self::Mark {
                repo: args.repo,
                file: args.file,
                line: args.line,
                side: args.side.into(),
                expected_commit: args.expected_commit,
            },
            ReviewSubcommand::Unmark(args) => Self::Unmark {
                repo: args.repo,
                file: args.file,
            },
        }
    }
}

#[derive(Args)]
struct NotesArgs {
    /// Path to the jj repository (default: current directory)
    #[arg(long, default_value = ".", allow_hyphen_values = true)]
    repo: String,

    /// Output format
    #[arg(long, default_value = "text")]
    format: OutputFormat,

    /// Include resolved notes in the output
    #[arg(long)]
    include_resolved: bool,
}

#[derive(Clone, clap::ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

impl From<OutputFormat> for ReviewOutputFormat {
    fn from(format: OutputFormat) -> Self {
        match format {
            OutputFormat::Text => ReviewOutputFormat::Text,
            OutputFormat::Json => ReviewOutputFormat::Json,
        }
    }
}

#[derive(Args)]
struct ResolveNoteArgs {
    /// Path to the jj repository (default: current directory)
    #[arg(long, default_value = ".", allow_hyphen_values = true)]
    repo: String,

    /// Review note id
    id: String,
}

#[derive(Args)]
struct AddNoteArgs {
    /// Path to the jj repository (default: current directory)
    #[arg(long, default_value = ".", allow_hyphen_values = true)]
    repo: String,

    /// File path relative to the repository root
    #[arg(long, allow_hyphen_values = true)]
    file: String,

    /// 1-based line number in the file
    #[arg(long, allow_hyphen_values = true)]
    line: u32,

    /// Diff side the note applies to
    #[arg(long, default_value = "new", allow_hyphen_values = true)]
    side: NoteSideArg,

    /// Review note message
    #[arg(short = 'm', long = "message", allow_hyphen_values = true)]
    message: String,
}

#[derive(Args)]
struct StatusArgs {
    /// Path to the jj repository (default: current directory)
    #[arg(long, default_value = ".", allow_hyphen_values = true)]
    repo: String,

    /// Output format
    #[arg(long, default_value = "text")]
    format: OutputFormat,
}

#[derive(Args)]
struct MarkArgs {
    /// Path to the jj repository (default: current directory)
    #[arg(long, default_value = ".", allow_hyphen_values = true)]
    repo: String,

    /// File path relative to the repository root
    #[arg(long, allow_hyphen_values = true)]
    file: String,

    /// 1-based changed line whose change group to mark; omit to mark the whole file
    #[arg(long, allow_hyphen_values = true)]
    line: Option<u32>,

    /// Diff side the line is on
    #[arg(long, default_value = "new", allow_hyphen_values = true)]
    side: NoteSideArg,

    /// Commit id reported by `status` when the diff was inspected
    #[arg(long, allow_hyphen_values = true)]
    expected_commit: String,
}

#[derive(Args)]
struct UnmarkArgs {
    /// Path to the jj repository (default: current directory)
    #[arg(long, default_value = ".", allow_hyphen_values = true)]
    repo: String,

    /// File to unmark; omit to drop every agent mark on the change
    #[arg(long, allow_hyphen_values = true)]
    file: Option<String>,
}

#[derive(Clone, clap::ValueEnum)]
enum NoteSideArg {
    New,
    Old,
}

impl From<NoteSideArg> for NoteSide {
    fn from(side: NoteSideArg) -> Self {
        match side {
            NoteSideArg::New => NoteSide::New,
            NoteSideArg::Old => NoteSide::Old,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::args;

    fn run_parse(arguments: &[&str]) -> ReviewCommand {
        match parse_review(&args(arguments)).expect("parses") {
            ReviewOutcome::Run(command) => command,
            ReviewOutcome::Print(text) => panic!("unexpected print: {text}"),
        }
    }

    fn parse_err(arguments: &[&str]) -> String {
        parse_review(&args(arguments)).expect_err("expected parse error")
    }

    #[test]
    fn notes_defaults_and_options_parse() {
        assert_eq!(
            run_parse(&["notes"]),
            ReviewCommand::Notes {
                repo: ".".to_owned(),
                format: ReviewOutputFormat::Text,
                include_resolved: false,
            }
        );

        assert_eq!(
            run_parse(&[
                "notes",
                "--repo",
                "/tmp/repo",
                "--format",
                "json",
                "--include-resolved"
            ]),
            ReviewCommand::Notes {
                repo: "/tmp/repo".to_owned(),
                format: ReviewOutputFormat::Json,
                include_resolved: true,
            }
        );
    }

    #[test]
    fn add_note_requires_anchor_and_message() {
        assert!(parse_err(&["add-note"]).contains("--file"));
        assert!(parse_err(&["add-note", "--file", "a.txt"]).contains("--line"));
        assert!(
            parse_err(&["add-note", "--file", "a.txt", "--line", "nope", "-m", "x"])
                .contains("--line")
        );
        assert!(parse_err(&["add-note", "--file", "a.txt", "--line"]).contains("--line"));
        assert!(parse_err(&["add-note", "--file", "a.txt", "--line", "3"]).contains("--message"));

        assert_eq!(
            run_parse(&[
                "add-note",
                "--file",
                "a.txt",
                "--line",
                "3",
                "-m",
                "check this"
            ]),
            ReviewCommand::AddNote {
                repo: ".".to_owned(),
                file: "a.txt".to_owned(),
                line: 3,
                side: NoteSide::New,
                message: "check this".to_owned(),
            }
        );
    }

    #[test]
    fn status_mark_and_unmark_parse() {
        assert_eq!(
            run_parse(&["status", "--format", "json"]),
            ReviewCommand::Status {
                repo: ".".to_owned(),
                format: ReviewOutputFormat::Json,
            }
        );
        assert_eq!(
            run_parse(&["mark", "--file", "a.txt", "--expected-commit", "abc"]),
            ReviewCommand::Mark {
                repo: ".".to_owned(),
                file: "a.txt".to_owned(),
                line: None,
                side: NoteSide::New,
                expected_commit: "abc".to_owned(),
            }
        );
        assert_eq!(
            run_parse(&[
                "mark",
                "--repo",
                "/tmp/repo",
                "--file",
                "a.txt",
                "--line",
                "4",
                "--side",
                "old",
                "--expected-commit",
                "abc",
            ]),
            ReviewCommand::Mark {
                repo: "/tmp/repo".to_owned(),
                file: "a.txt".to_owned(),
                line: Some(4),
                side: NoteSide::Old,
                expected_commit: "abc".to_owned(),
            }
        );
        assert!(parse_err(&["mark", "--file", "a.txt"]).contains("--expected-commit"));
        assert!(parse_err(&["mark", "--expected-commit", "abc"]).contains("--file"));
        assert!(
            parse_err(&[
                "mark",
                "--file",
                "a.txt",
                "--line",
                "x",
                "--expected-commit",
                "abc"
            ])
            .contains("--line")
        );
        assert_eq!(
            run_parse(&["unmark"]),
            ReviewCommand::Unmark {
                repo: ".".to_owned(),
                file: None,
            }
        );
        assert_eq!(
            run_parse(&["unmark", "--file", "a.txt"]),
            ReviewCommand::Unmark {
                repo: ".".to_owned(),
                file: Some("a.txt".to_owned()),
            }
        );
    }

    #[test]
    fn resolve_note_requires_an_id_and_defaults_the_repo() {
        let err = parse_err(&["resolve-note"]);
        assert!(err.contains("ID"), "expected missing-id error, got: {err}");
        assert_eq!(
            run_parse(&["resolve-note", "note-1"]),
            ReviewCommand::ResolveNote {
                id: "note-1".to_owned(),
                repo: ".".to_owned(),
            }
        );
    }

    #[test]
    fn invalid_values_and_extra_arguments_are_rejected() {
        let err = parse_err(&["notes", "--format", "xml"]);
        assert!(err.contains("xml"), "expected format error, got: {err}");
        let err = parse_err(&[
            "add-note", "--file", "a.txt", "--line", "3", "--side", "sideways", "-m", "x",
        ]);
        assert!(err.contains("sideways"), "expected side error, got: {err}");
        let err = parse_err(&["resolve-note", "note-1", "extra"]);
        assert!(
            err.contains("extra"),
            "expected extra-argument error, got: {err}"
        );
    }

    #[test]
    fn option_values_may_begin_with_a_dash() {
        assert_eq!(
            run_parse(&[
                "add-note",
                "--repo",
                "-x",
                "--file",
                "a.txt",
                "--line",
                "3",
                "-m",
                "- rename this"
            ]),
            ReviewCommand::AddNote {
                repo: "-x".to_owned(),
                file: "a.txt".to_owned(),
                line: 3,
                side: NoteSide::New,
                message: "- rename this".to_owned(),
            }
        );

        assert_eq!(
            run_parse(&["notes", "--repo", "-x"]),
            ReviewCommand::Notes {
                repo: "-x".to_owned(),
                format: ReviewOutputFormat::Text,
                include_resolved: false,
            }
        );
    }

    #[test]
    fn equals_form_works_for_short_aliases() {
        assert_eq!(
            run_parse(&["add-note", "--file", "a.txt", "--line", "3", "-m=hello"]),
            ReviewCommand::AddNote {
                repo: ".".to_owned(),
                file: "a.txt".to_owned(),
                line: 3,
                side: NoteSide::New,
                message: "hello".to_owned(),
            }
        );
    }

    #[test]
    fn double_dash_makes_the_rest_positional() {
        assert_eq!(
            run_parse(&["resolve-note", "--", "-note-1"]),
            ReviewCommand::ResolveNote {
                id: "-note-1".to_owned(),
                repo: ".".to_owned(),
            }
        );
    }

    #[test]
    fn help_is_printed_as_a_success_outcome() {
        for arguments in [
            &["--help"][..],
            &["notes", "--help"][..],
            &["add-note", "--help"][..],
        ] {
            match parse_review(&args(arguments)) {
                Ok(ReviewOutcome::Print(text)) => {
                    assert!(text.contains("Usage"), "expected usage, got: {text}")
                }
                other => panic!("expected print outcome, got: {other:?}"),
            }
        }
    }
}
