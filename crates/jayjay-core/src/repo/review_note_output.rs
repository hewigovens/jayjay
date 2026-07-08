use std::env;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use jayjay_primitives::{NoteSide, ReviewNoteStatus};
use jayjay_review::ReviewStore;
use serde::Serialize;

use crate::types::{CoreError, CoreResult};

use super::Repo;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewNoteOutputFormat {
    Text,
    Json,
}

#[derive(Serialize)]
struct NotesOutput {
    schema_version: u32,
    repo: String,
    change_id: String,
    notes: Vec<ReviewNoteStatus>,
}

pub fn add_review_note(
    repo: &Path,
    file: &str,
    line: u32,
    side: NoteSide,
    message: &str,
) -> CoreResult<String> {
    let body = message.trim();
    if body.is_empty() {
        return Err(CoreError::internal("note body is empty"));
    }
    let repo = open_repo(&canonicalize(repo))?;
    // The same anchor the GUI would record, so the note shows a marker and bubble there and reconciles Current here.
    let anchor = repo.review_note_anchor("@", file, side, line)?;
    let mut store = ReviewStore::load();
    let note = store.add_note(anchor, body);
    Ok(format!(
        "Added review note {} at {}:{}\n",
        note.id, note.path, note.line
    ))
}

pub fn review_notes_output(
    repo: &Path,
    format: ReviewNoteOutputFormat,
    include_resolved: bool,
) -> CoreResult<String> {
    let repo = open_repo(&canonicalize(repo))?;
    // The same provider the GUI reconciles through, so rename detection, LFS normalization, and change-group indices agree across surfaces.
    let report = repo.review_notes_report(&ReviewStore::load(), "@", include_resolved)?;
    match format {
        ReviewNoteOutputFormat::Text => Ok(notes_text(&report.notes)),
        ReviewNoteOutputFormat::Json => {
            let output = NotesOutput {
                schema_version: 1,
                repo: repo.path().display().to_string(),
                change_id: report.change_id,
                notes: report.notes,
            };
            let mut text = serde_json::to_string_pretty(&output)
                .map_err(|error| CoreError::internal(error.to_string()))?;
            text.push('\n');
            Ok(text)
        }
    }
}

pub fn resolve_review_note(repo: &Path, id: &str) -> CoreResult<String> {
    validate_note_id(id)?;
    let repo = open_repo(&canonicalize(repo))?;
    // The store is shared across repos; only resolve notes that belong to this repo's working-copy change so a copy-pasted id can't silently resolve someone else's note.
    let change_id = repo.show_summary("@")?.info.change_id.id;
    let mut store = ReviewStore::load();
    if store
        .list_notes(&change_id, true)
        .iter()
        .all(|note| note.id != id)
    {
        return Err(CoreError::internal(format!(
            "review note not found on the working-copy change: {id}"
        )));
    }
    let _ = store.resolve_note(id);
    Ok(format!("Resolved review note {id}\n"))
}

fn open_repo(path: &Path) -> CoreResult<Repo> {
    let repo = Repo::open(path)?;
    repo.refresh_working_copy()?;
    Ok(repo)
}

fn canonicalize(path: &Path) -> PathBuf {
    let abs = if path.is_absolute() {
        path.to_owned()
    } else {
        env::current_dir()
            .map(|cwd| cwd.join(path))
            .unwrap_or_else(|_| path.to_owned())
    };
    abs.canonicalize().unwrap_or(abs)
}

/// Complete enough for agents to act on without JSON parsing: full body and anchor line, one indented block per note.
fn notes_text(notes: &[ReviewNoteStatus]) -> String {
    if notes.is_empty() {
        return "No review notes.\n".to_string();
    }

    let mut output = String::new();
    for (index, item) in notes.iter().enumerate() {
        if index > 0 {
            output.push('\n');
        }
        let note = &item.note;
        let side = match note.side {
            NoteSide::New => "",
            NoteSide::Old => " (old side)",
        };
        writeln!(
            output,
            "{}:{}{} [{}] {}",
            note.path,
            note.line,
            side,
            item.status.as_str(),
            note.id
        )
        .expect("write to String");
        let excerpt = note.anchor_excerpt.trim();
        if !excerpt.is_empty() {
            writeln!(output, "  anchor: {excerpt}").expect("write to String");
        }
        for line in note.body.lines() {
            writeln!(output, "  {line}").expect("write to String");
        }
    }
    output
}

fn validate_note_id(id: &str) -> CoreResult<()> {
    let valid = !id.is_empty()
        && id.len() <= 80
        && id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_');
    if valid {
        Ok(())
    } else {
        Err(CoreError::internal("malformed review note id"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_bounded_uuid_like_note_ids() {
        assert!(validate_note_id("9df19c12-7e4f-47b8-a414-0df1a1e61240").is_ok());
        assert!(validate_note_id("note_1").is_ok());
    }

    #[test]
    fn rejects_malformed_note_ids() {
        assert!(validate_note_id("").is_err());
        assert!(validate_note_id("glob:foo").is_err());
        assert!(validate_note_id("../note").is_err());
        assert!(validate_note_id(&"a".repeat(81)).is_err());
    }
}
