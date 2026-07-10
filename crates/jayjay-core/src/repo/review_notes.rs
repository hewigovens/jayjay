use jayjay_primitives::{
    JayJayError, NoteAnchor, NoteEntry, NoteSide, ReviewDiffProvider, ReviewFileDiff, ReviewHunk,
    ReviewNoteStatus, ReviewResult,
};
use jayjay_review::ReviewStore;
use jj_lib::hex_util::encode_reverse_hex;
use jj_lib::object_id::ObjectId;

use crate::types::*;

use super::Repo;

pub struct ReviewNotesReport {
    pub change_id: String,
    pub notes: Vec<ReviewNoteStatus>,
}

struct CoreReviewDiffProvider<'a> {
    repo: &'a Repo,
    rev: &'a str,
    hunks: Vec<ReviewHunk>,
}

impl Repo {
    pub fn review_notes(
        &self,
        rev: &str,
        include_resolved: bool,
    ) -> CoreResult<Vec<ReviewNoteStatus>> {
        Ok(self
            .review_notes_report(&ReviewStore::load(), rev, include_resolved)?
            .notes)
    }

    pub fn review_notes_report(
        &self,
        store: &ReviewStore,
        rev: &str,
        include_resolved: bool,
    ) -> CoreResult<ReviewNotesReport> {
        let change_id = self.resolve_change_id(rev)?;
        // Every review-state change triggers a refresh; skip the summary walk (materialize + hash all changed files) when the change has no notes.
        if store.list_notes(&change_id, include_resolved).is_empty() {
            return Ok(ReviewNotesReport {
                change_id,
                notes: vec![],
            });
        }

        let provider = self.review_diff_provider(rev)?;
        let notes = store.reconcile(&provider, &change_id, include_resolved)?;
        Ok(ReviewNotesReport { change_id, notes })
    }

    // Send-safe entry point for shells (GPUI's Rc<RefCell<_>> ReviewStore) that can't move the store itself into a background task; skips the show_summary walk when notes is empty, same as review_notes_report.
    pub fn reconcile_review_notes(
        &self,
        notes: Vec<NoteEntry>,
        rev: &str,
    ) -> CoreResult<ReviewNotesReport> {
        let change_id = self.resolve_change_id(rev)?;
        if notes.is_empty() {
            return Ok(ReviewNotesReport {
                change_id,
                notes: vec![],
            });
        }

        let provider = self.review_diff_provider(rev)?;
        let notes = jayjay_review::reconcile_notes(notes, &provider)?;
        Ok(ReviewNotesReport { change_id, notes })
    }

    pub fn review_note_anchor(
        &self,
        rev: &str,
        path: &str,
        side: NoteSide,
        line: u32,
    ) -> CoreResult<NoteAnchor> {
        let change_id = self.resolve_change_id(rev)?;
        let provider = self.review_diff_provider(rev)?;
        jayjay_review::build_note_anchor(&provider, &change_id, path, side, line)?.ok_or_else(
            || {
                let side = match side {
                    NoteSide::New => "new",
                    NoteSide::Old => "old",
                };
                JayJayError::review(format!(
                    "{path}:{line} ({side} side) is not a changed line in this change's diff"
                ))
            },
        )
    }

    fn resolve_change_id(&self, rev: &str) -> CoreResult<String> {
        let repo = self.get_repo();
        let commit = self.resolve_commit(&repo, rev)?;
        Ok(encode_reverse_hex(commit.change_id().as_bytes()))
    }

    // self and rev share lifetime 'a because CoreReviewDiffProvider borrows both into the same field lifetime.
    fn review_diff_provider<'a>(&'a self, rev: &'a str) -> CoreResult<CoreReviewDiffProvider<'a>> {
        let summary = self.show_summary(rev)?;
        Ok(CoreReviewDiffProvider {
            repo: self,
            rev,
            hunks: summary.diff.into_iter().map(ReviewHunk::from).collect(),
        })
    }
}

impl ReviewDiffProvider for CoreReviewDiffProvider<'_> {
    fn review_hunks(&self) -> ReviewResult<Vec<ReviewHunk>> {
        Ok(self.hunks.clone())
    }

    fn review_file_diff(&self, hunk: &ReviewHunk) -> ReviewResult<ReviewFileDiff> {
        let renamed_from = hunk
            .old_path
            .as_deref()
            .filter(|_| hunk.hunk_type == HunkType::Renamed);
        let diff = match renamed_from {
            Some(old_path) => self.repo.show_file_rename(self.rev, old_path, &hunk.path)?,
            None => self.repo.show_file(self.rev, &hunk.path)?,
        };
        Ok(ReviewFileDiff {
            old_content: diff.old.content,
            new_content: diff.new.content,
        })
    }
}
