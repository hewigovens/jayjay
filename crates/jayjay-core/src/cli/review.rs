use std::path::Path;

use crate::{
    CoreResult, NoteSide, ReviewOutputFormat, add_review_note, mark_review_file,
    resolve_review_note, review_notes_output, review_status_output, unmark_review_files,
};

#[derive(Debug, PartialEq)]
pub(super) enum ReviewCommand {
    Notes {
        repo: String,
        format: ReviewOutputFormat,
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
    Status {
        repo: String,
        format: ReviewOutputFormat,
    },
    Mark {
        repo: String,
        file: String,
        line: Option<u32>,
        side: NoteSide,
        expected_commit: String,
    },
    Unmark {
        repo: String,
        file: Option<String>,
    },
}

impl ReviewCommand {
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
            Self::Status { repo, format } => review_status_output(Path::new(repo), *format),
            Self::Mark {
                repo,
                file,
                line,
                side,
                expected_commit,
            } => mark_review_file(Path::new(repo), file, *line, *side, expected_commit),
            Self::Unmark { repo, file } => unmark_review_files(Path::new(repo), file.as_deref()),
        }
    }
}
