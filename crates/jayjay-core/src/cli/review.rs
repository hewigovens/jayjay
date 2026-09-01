use std::path::Path;

use crate::{
    CoreResult, NoteSide, ReviewNoteOutputFormat, add_review_note, resolve_review_note,
    review_notes_output,
};

use super::parser::ArgParser;

#[derive(Debug, PartialEq)]
pub(super) enum ReviewCommand {
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
    pub(super) fn parse(arguments: &[String]) -> Result<Self, String> {
        let (subcommand, rest) = arguments
            .split_first()
            .ok_or_else(|| "missing review subcommand".to_owned())?;
        let mut parser = ArgParser::new(rest);
        match subcommand.as_str() {
            "notes" => Self::parse_notes(&mut parser),
            "resolve-note" => Self::parse_resolve_note(&mut parser),
            "add-note" => Self::parse_add_note(&mut parser),
            other => Err(format!("unknown review subcommand: {other}")),
        }
    }

    pub(super) fn run(&self) -> CoreResult<String> {
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
            .unwrap_or_else(|| "text".to_owned());
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
            .ok_or_else(|| "missing review note id".to_owned())?;
        parser.finish()?;
        Ok(Self::ResolveNote { id, repo })
    }

    fn parse_add_note(parser: &mut ArgParser) -> Result<Self, String> {
        let repo = default_repo(parser)?;
        let file = parser
            .option("--file", None)?
            .ok_or_else(|| "missing --file".to_owned())?;
        let line = parser
            .option("--line", None)?
            .and_then(|value| value.parse::<u32>().ok())
            .ok_or_else(|| "missing or invalid --line".to_owned())?;
        let side = parser
            .option("--side", None)?
            .unwrap_or_else(|| "new".to_owned());
        let side = match side.as_str() {
            "new" => NoteSide::New,
            "old" => NoteSide::Old,
            _ => return Err(format!("unsupported review note side: {side}")),
        };
        let message = parser
            .option("--message", Some("-m"))?
            .ok_or_else(|| "missing --message".to_owned())?;
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
        .unwrap_or_else(|| ".".to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::args;

    #[test]
    fn notes_defaults_and_options_parse() {
        assert_eq!(
            ReviewCommand::parse(&args(&["notes"])).expect("parses"),
            ReviewCommand::Notes {
                repo: ".".to_owned(),
                format: ReviewNoteOutputFormat::Text,
                include_resolved: false,
            }
        );

        assert_eq!(
            ReviewCommand::parse(&args(&[
                "notes",
                "--repo",
                "/tmp/repo",
                "--format",
                "json",
                "--include-resolved",
            ]))
            .expect("parses"),
            ReviewCommand::Notes {
                repo: "/tmp/repo".to_owned(),
                format: ReviewNoteOutputFormat::Json,
                include_resolved: true,
            }
        );
    }

    #[test]
    fn add_note_requires_anchor_and_message() {
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
            ReviewCommand::parse(&args(&["add-note", "--file", "a.txt", "--line"])).unwrap_err(),
            "missing value for --line"
        );
        assert_eq!(
            ReviewCommand::parse(&args(&["add-note", "--file", "a.txt", "--line", "3"]))
                .unwrap_err(),
            "missing --message"
        );

        assert_eq!(
            ReviewCommand::parse(&args(&[
                "add-note",
                "--file",
                "a.txt",
                "--line",
                "3",
                "-m",
                "check this",
            ]))
            .expect("parses"),
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
    fn resolve_note_requires_an_id_and_defaults_the_repo() {
        assert_eq!(
            ReviewCommand::parse(&args(&["resolve-note"])).unwrap_err(),
            "missing review note id"
        );
        assert_eq!(
            ReviewCommand::parse(&args(&["resolve-note", "note-1"])).expect("parses"),
            ReviewCommand::ResolveNote {
                id: "note-1".to_owned(),
                repo: ".".to_owned(),
            }
        );
    }

    #[test]
    fn invalid_values_and_extra_arguments_are_rejected() {
        assert_eq!(
            ReviewCommand::parse(&args(&["notes", "--format", "xml"])).unwrap_err(),
            "unsupported review notes format: xml"
        );
        assert_eq!(
            ReviewCommand::parse(&args(&[
                "add-note", "--file", "a.txt", "--line", "3", "--side", "sideways", "-m", "x"
            ]))
            .unwrap_err(),
            "unsupported review note side: sideways"
        );
        assert_eq!(
            ReviewCommand::parse(&args(&["resolve-note", "note-1", "extra"])).unwrap_err(),
            "unexpected argument: extra"
        );
    }
}
