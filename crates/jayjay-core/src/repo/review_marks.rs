use std::collections::HashSet;
use std::fmt::Write as _;
use std::path::Path;

use jayjay_primitives::{NoteSide, ReviewFileDiff, ReviewFileRollup, ReviewMarkSource};
use jayjay_review::{ReviewStore, change_group_index};
use serde::Serialize;

use super::review_note_output::{ReviewOutputFormat, canonicalize, open_repo};
use super::review_snapshot::{review_display_group_map_from_hunk, review_snapshot_from_hunk};
use crate::types::*;

#[derive(Serialize)]
struct StatusOutput {
    schema_version: u32,
    repo: String,
    change_id: String,
    commit_id: String,
    files: Vec<FileStatus>,
    reviewed: usize,
    total: usize,
}

#[derive(Serialize)]
struct FileStatus {
    path: String,
    status: ReviewFileRollup,
    source: ReviewMarkSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    reviewed_groups: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    total_groups: Option<usize>,
}

pub fn mark_review_file(
    repo: &Path,
    file: &str,
    line: Option<u32>,
    side: NoteSide,
    expected_commit: &str,
) -> CoreResult<String> {
    let repo = open_repo(&canonicalize(repo))?;
    let detail = repo.show_summary("@")?;
    let commit_id = detail.info.commit_id.id;
    if commit_id != expected_commit {
        return Err(CoreError::review(format!(
            "Working copy changed since inspection (expected {expected_commit}, current {commit_id}); inspect the current commit before marking"
        )));
    }
    let change_id = detail.info.change_id.id;
    let listed = detail
        .diff
        .iter()
        .find(|hunk| hunk.path == file)
        .ok_or_else(|| CoreError::review(format!("{file} is not part of this change's diff")))?;
    let hunk = repo.load_review_hunk(&commit_id, &listed.path, listed.old_path.as_deref())?;
    if hunk.review_identity.is_empty() {
        return Err(CoreError::review(format!(
            "{file} cannot carry review marks"
        )));
    }
    let snapshot = review_snapshot_from_hunk(&hunk);
    let mut store = ReviewStore::load();
    let Some(line) = line else {
        store
            .mark_reviewed_as(
                &change_id,
                file,
                &hunk.review_identity,
                Some(&snapshot),
                ReviewMarkSource::Agent,
            )
            .map_err(|error| CoreError::review(format!("Could not save review marks: {error}")))?;
        return Ok(format!("Marked {file} reviewed\n"));
    };
    if snapshot.fingerprints.is_empty() {
        return Err(CoreError::review(format!(
            "{file} has no stable change groups; mark the whole file instead"
        )));
    }
    let display_index = change_group_index(file, &ReviewFileDiff::from(&hunk), side, line)
        .ok_or_else(|| {
            CoreError::review(format!(
                "{file}:{line} ({} side) is not a changed line in this change's diff",
                side.as_str()
            ))
        })?;
    let mapping = review_display_group_map_from_hunk(&hunk, false);
    let indices = mapping
        .get(display_index as usize)
        .filter(|indices| !indices.is_empty())
        .ok_or_else(|| {
            CoreError::review(format!(
                "{file}:{line} has no stable change group; mark the whole file instead"
            ))
        })?;
    store
        .mark_groups_reviewed_as(
            &change_id,
            file,
            &hunk.review_identity,
            &snapshot,
            indices,
            ReviewMarkSource::Agent,
        )
        .map_err(|error| CoreError::review(format!("Could not save review marks: {error}")))?;
    let state = store.file_review_state(&change_id, file, &hunk.review_identity, Some(&snapshot));
    Ok(format!(
        "Marked {file}:{line} reviewed ({}/{} groups)\n",
        state.reviewed_indices().len(),
        state.group_states().len()
    ))
}

pub fn unmark_review_files(repo: &Path, file: Option<&str>) -> CoreResult<String> {
    let repo = open_repo(&canonicalize(repo))?;
    let change_id = repo.show_summary("@")?.info.change_id.id;
    let cleared = ReviewStore::load()
        .unmark_owned_by(&change_id, file, ReviewMarkSource::Agent)
        .map_err(|error| CoreError::review(format!("Could not save review marks: {error}")))?;
    if let Some(file) = file {
        return if cleared.is_empty() {
            Err(CoreError::review(format!(
                "{file} has no agent review mark on this change"
            )))
        } else {
            Ok(format!("Unmarked {file}\n"))
        };
    }
    let mut output = format!("Unmarked {} agent-reviewed files\n", cleared.len());
    for path in cleared {
        writeln!(output, "  {path}").expect("write to String");
    }
    Ok(output)
}

pub fn review_status_output(repo: &Path, format: ReviewOutputFormat) -> CoreResult<String> {
    let repo = open_repo(&canonicalize(repo))?;
    let detail = repo.show_summary("@")?;
    let change_id = detail.info.change_id.id;
    let commit_id = detail.info.commit_id.id;
    let store = ReviewStore::load();
    let agent_marked: HashSet<String> = store
        .paths_owned_by(&change_id, ReviewMarkSource::Agent)
        .into_iter()
        .collect();
    let mut files = Vec::with_capacity(detail.diff.len());
    for hunk in &detail.diff {
        let mut state =
            store.file_review_state(&change_id, &hunk.path, &hunk.review_identity, None);
        if state.has_changed_since_review() {
            let snapshot =
                repo.review_file_snapshot(&commit_id, &hunk.path, hunk.old_path.as_deref())?;
            state = store.file_review_state(
                &change_id,
                &hunk.path,
                &hunk.review_identity,
                Some(&snapshot),
            );
        }
        let groups = state.group_states();
        files.push(FileStatus {
            path: hunk.path.clone(),
            status: state.rollup(),
            source: if agent_marked.contains(&hunk.path) {
                ReviewMarkSource::Agent
            } else {
                ReviewMarkSource::User
            },
            reviewed_groups: (!groups.is_empty()).then(|| state.reviewed_indices().len()),
            total_groups: (!groups.is_empty()).then_some(groups.len()),
        });
    }
    let output = StatusOutput {
        schema_version: 1,
        repo: repo.path().display().to_string(),
        change_id,
        commit_id,
        reviewed: files
            .iter()
            .filter(|file| file.status == ReviewFileRollup::Reviewed)
            .count(),
        total: files.len(),
        files,
    };
    match format {
        ReviewOutputFormat::Text => Ok(output.text()),
        ReviewOutputFormat::Json => {
            let mut text = serde_json::to_string_pretty(&output)
                .map_err(|error| CoreError::internal(error.to_string()))?;
            text.push('\n');
            Ok(text)
        }
    }
}

impl StatusOutput {
    fn text(&self) -> String {
        let mut text = format!("Commit {}\n", self.commit_id);
        for file in &self.files {
            let groups = match (file.reviewed_groups, file.total_groups) {
                (Some(reviewed), Some(total)) if file.status == ReviewFileRollup::Partial => {
                    format!("{reviewed}/{total}")
                }
                _ => String::new(),
            };
            let source = match file.source {
                ReviewMarkSource::Agent => "  (agent)",
                ReviewMarkSource::User => "",
            };
            writeln!(
                text,
                "{:<21}{groups:<5}{}{source}",
                file.status.as_str(),
                file.path
            )
            .expect("write to String");
        }
        let by_agent = self
            .files
            .iter()
            .filter(|file| file.source == ReviewMarkSource::Agent)
            .count();
        let partial = self
            .files
            .iter()
            .filter(|file| file.status == ReviewFileRollup::Partial)
            .count();
        writeln!(
            text,
            "\n{} of {} files reviewed ({by_agent} by agent), {partial} partial",
            self.reviewed, self.total
        )
        .expect("write to String");
        text
    }
}
