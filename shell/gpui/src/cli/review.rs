use std::path::Path;

use jayjay_core::{
    CoreResult, NoteSide, ReviewNoteOutputFormat, add_review_note, resolve_review_note,
    review_notes_output,
};

use super::parser::ArgParser;

/// Must expose the same subcommands, flags, and defaults as the SwiftUI shell's `ReviewCommand` so `jayjay review ...` behaves identically regardless of which shell binary handles it.
#[derive(Debug)]
pub enum ReviewCommand {
    Notes {
        repo: String,
        format: ReviewNoteOutputFormat,
        include_resolved: bool,
    },
    ResolveNote {
        id: String,
        repo: String,
    },
    AddNote {
        repo: String,
        file: String,
        line: u32,
        side: NoteSide,
        message: String,
    },
}

impl ReviewCommand {
    pub fn parse(arguments: &[String]) -> Result<Self, String> {
        let (subcommand, rest) = arguments
            .split_first()
            .ok_or_else(|| "missing review subcommand".to_string())?;
        let mut parser = ArgParser::new(rest);
        match subcommand.as_str() {
            "notes" => Self::parse_notes(&mut parser),
            "resolve-note" => Self::parse_resolve_note(&mut parser),
            "add-note" => Self::parse_add_note(&mut parser),
            other => Err(format!("unknown review subcommand: {other}")),
        }
    }

    pub fn run(&self) -> CoreResult<String> {
        match self {
            Self::Notes {
                repo,
                format,
                include_resolved,
            } => review_notes_output(Path::new(repo), *format, *include_resolved),
            Self::ResolveNote { id, repo } => resolve_review_note(Path::new(repo), id),
            Self::AddNote {
                repo,
                file,
                line,
                side,
                message,
            } => add_review_note(Path::new(repo), file, *line, *side, message),
        }
    }

    fn parse_notes(parser: &mut ArgParser) -> Result<Self, String> {
        let repo = default_repo(parser)?;
        let format = parser
            .option("--format", None)?
            .unwrap_or_else(|| "text".to_string());
        let format = match format.as_str() {
            "text" => ReviewNoteOutputFormat::Text,
            "json" => ReviewNoteOutputFormat::Json,
            _ => return Err(format!("unsupported review notes format: {format}")),
        };
        let include_resolved = parser.flag("--include-resolved");
        parser.finish()?;
        Ok(Self::Notes {
            repo,
            format,
            include_resolved,
        })
    }

    fn parse_resolve_note(parser: &mut ArgParser) -> Result<Self, String> {
        let repo = default_repo(parser)?;
        let id = parser
            .positional()
            .ok_or_else(|| "missing review note id".to_string())?;
        parser.finish()?;
        Ok(Self::ResolveNote { id, repo })
    }

    fn parse_add_note(parser: &mut ArgParser) -> Result<Self, String> {
        let repo = default_repo(parser)?;
        let file = parser
            .option("--file", None)?
            .ok_or_else(|| "missing --file".to_string())?;
        // `?` propagates `option`'s "missing value for --line" first; only an absent/non-numeric value falls through to "missing or invalid --line" below.
        let line = parser.option("--line", None)?;
        let line = line
            .and_then(|value| value.parse::<u32>().ok())
            .ok_or_else(|| "missing or invalid --line".to_string())?;
        let side = parser
            .option("--side", None)?
            .unwrap_or_else(|| "new".to_string());
        let side = match side.as_str() {
            "new" => NoteSide::New,
            "old" => NoteSide::Old,
            _ => return Err(format!("unsupported review note side: {side}")),
        };
        let message = parser
            .option("--message", Some("-m"))?
            .ok_or_else(|| "missing --message".to_string())?;
        parser.finish()?;
        Ok(Self::AddNote {
            repo,
            file,
            line,
            side,
            message,
        })
    }
}

fn default_repo(parser: &mut ArgParser) -> Result<String, String> {
    Ok(parser
        .option("--repo", None)?
        .unwrap_or_else(|| ".".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn notes_defaults_repo_dot_format_text_no_resolved() {
        let command = ReviewCommand::parse(&args(&["notes"])).expect("parses");
        match command {
            ReviewCommand::Notes {
                repo,
                format,
                include_resolved,
            } => {
                assert_eq!(repo, ".");
                assert_eq!(format, ReviewNoteOutputFormat::Text);
                assert!(!include_resolved);
            }
            _ => panic!("expected Notes"),
        }
    }

    #[test]
    fn notes_accepts_json_format_and_include_resolved() {
        let command = ReviewCommand::parse(&args(&[
            "notes",
            "--repo",
            "/tmp/repo",
            "--format",
            "json",
            "--include-resolved",
        ]))
        .expect("parses");
        match command {
            ReviewCommand::Notes {
                repo,
                format,
                include_resolved,
            } => {
                assert_eq!(repo, "/tmp/repo");
                assert_eq!(format, ReviewNoteOutputFormat::Json);
                assert!(include_resolved);
            }
            _ => panic!("expected Notes"),
        }
    }

    #[test]
    fn notes_rejects_unsupported_format() {
        let error = ReviewCommand::parse(&args(&["notes", "--format", "xml"])).unwrap_err();
        assert_eq!(error, "unsupported review notes format: xml");
    }

    #[test]
    fn resolve_note_requires_positional_id() {
        let error = ReviewCommand::parse(&args(&["resolve-note"])).unwrap_err();
        assert_eq!(error, "missing review note id");

        let command = ReviewCommand::parse(&args(&["resolve-note", "note-1"])).expect("parses");
        match command {
            ReviewCommand::ResolveNote { id, repo } => {
                assert_eq!(id, "note-1");
                assert_eq!(repo, ".");
            }
            _ => panic!("expected ResolveNote"),
        }
    }

    #[test]
    fn add_note_requires_file_line_and_message() {
        assert_eq!(
            ReviewCommand::parse(&args(&["add-note"])).unwrap_err(),
            "missing --file"
        );
        assert_eq!(
            ReviewCommand::parse(&args(&["add-note", "--file", "a.txt"])).unwrap_err(),
            "missing or invalid --line"
        );
        assert_eq!(
            ReviewCommand::parse(&args(&["add-note", "--file", "a.txt", "--line", "nope"]))
                .unwrap_err(),
            "missing or invalid --line"
        );
        assert_eq!(
            ReviewCommand::parse(&args(&["add-note", "--file", "a.txt", "--line", "3"]))
                .unwrap_err(),
            "missing --message"
        );
    }

    #[test]
    fn add_note_defaults_side_new_and_accepts_message_alias() {
        let command = ReviewCommand::parse(&args(&[
            "add-note",
            "--file",
            "a.txt",
            "--line",
            "3",
            "-m",
            "check this",
        ]))
        .expect("parses");
        match command {
            ReviewCommand::AddNote {
                file,
                line,
                side,
                message,
                ..
            } => {
                assert_eq!(file, "a.txt");
                assert_eq!(line, 3);
                assert_eq!(side, NoteSide::New);
                assert_eq!(message, "check this");
            }
            _ => panic!("expected AddNote"),
        }
    }

    #[test]
    fn add_note_rejects_unsupported_side() {
        let error = ReviewCommand::parse(&args(&[
            "add-note", "--file", "a.txt", "--line", "3", "--side", "sideways", "-m", "x",
        ]))
        .unwrap_err();
        assert_eq!(error, "unsupported review note side: sideways");
    }

    #[test]
    fn unknown_and_missing_subcommand_are_rejected() {
        assert_eq!(
            ReviewCommand::parse(&args(&[])).unwrap_err(),
            "missing review subcommand"
        );
        assert_eq!(
            ReviewCommand::parse(&args(&["bogus"])).unwrap_err(),
            "unknown review subcommand: bogus"
        );
    }

    #[test]
    fn trailing_unexpected_argument_is_rejected() {
        let error = ReviewCommand::parse(&args(&["resolve-note", "note-1", "extra"])).unwrap_err();
        assert_eq!(error, "unexpected argument: extra");
    }
}
