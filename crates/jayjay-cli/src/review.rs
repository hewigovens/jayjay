use std::path::Path;

use jayjay_core::Repo;
use jayjay_review::{ReviewNoteStatus, ReviewStore};
use serde::Serialize;

use crate::args::{OutputFormat, ReviewCommand};
use crate::launcher::canonicalize;

#[derive(Serialize)]
struct NotesOutput {
    schema_version: u32,
    repo: String,
    change_id: String,
    notes: Vec<ReviewNoteStatus>,
}

pub(crate) fn run(command: ReviewCommand) -> Result<(), String> {
    match command {
        ReviewCommand::Notes {
            repo,
            format,
            include_resolved,
        } => list_notes(&repo, format, include_resolved),
        ReviewCommand::ResolveNote { id, repo } => resolve_note(&repo, &id),
    }
}

fn list_notes(repo: &str, format: OutputFormat, include_resolved: bool) -> Result<(), String> {
    let repo = open_repo(&canonicalize(repo))?;
    // The same provider the GUI reconciles through, so rename detection, LFS normalization, and change-group indices agree across surfaces.
    let report = repo
        .review_notes_report(&ReviewStore::load(), "@", include_resolved)
        .map_err(|error| error.to_string())?;
    match format {
        OutputFormat::Text => print_notes_text(&report.notes),
        OutputFormat::Json => {
            let output = NotesOutput {
                schema_version: 1,
                repo: repo.path().display().to_string(),
                change_id: report.change_id,
                notes: report.notes,
            };
            let text = serde_json::to_string_pretty(&output).map_err(|error| error.to_string())?;
            println!("{text}");
        }
    }
    Ok(())
}

fn resolve_note(repo: &str, id: &str) -> Result<(), String> {
    validate_note_id(id)?;
    let repo = open_repo(&canonicalize(repo))?;
    // The store is shared across repos; only resolve notes that belong to this repo's working-copy change so a copy-pasted id can't silently resolve someone else's note.
    let change_id = repo
        .show_summary("@")
        .map_err(|error| error.to_string())?
        .info
        .change_id
        .id;
    let mut store = ReviewStore::load();
    if store
        .list_notes(&change_id, true)
        .iter()
        .all(|note| note.id != id)
    {
        return Err(format!(
            "review note not found on the working-copy change: {id}"
        ));
    }
    let _ = store.resolve_note(id);
    println!("Resolved review note {id}");
    Ok(())
}

fn open_repo(path: &Path) -> Result<Repo, String> {
    let repo = Repo::open(path).map_err(|error| error.to_string())?;
    repo.refresh_working_copy()
        .map_err(|error| error.to_string())?;
    Ok(repo)
}

fn print_notes_text(notes: &[ReviewNoteStatus]) {
    if notes.is_empty() {
        println!("No review notes.");
        return;
    }

    for item in notes {
        let note = &item.note;
        let status = item.status.as_str();
        let body = note.body.lines().next().unwrap_or("").trim();
        println!(
            "{}:{} [{}] {} {}",
            note.path, note.line, status, note.id, body
        );
    }
}

fn validate_note_id(id: &str) -> Result<(), String> {
    let valid = !id.is_empty()
        && id.len() <= 80
        && id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_');
    if valid {
        Ok(())
    } else {
        Err("malformed review note id".to_string())
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
