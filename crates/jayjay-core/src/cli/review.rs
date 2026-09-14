use std::path::Path;

use crate::{
    CoreResult, NoteSide, ReviewNoteOutputFormat, add_review_note, resolve_review_note,
    review_notes_output,
};

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
}
