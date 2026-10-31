use jayjay_primitives::{
    DiffHunk, JayError, JayResult, NoteAnchor, NoteEntry, NoteSide, ReviewDiffProvider,
    ReviewFileDiff, ReviewHunk, ReviewNoteStatus,
};
use jayjay_review::ReviewStore;
use jj_lib::hex_util::encode_reverse_hex;
use jj_lib::object_id::ObjectId;

use super::Repo;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ReviewNotesReport {
    pub change_id: String,
    pub notes: Vec<ReviewNoteStatus>,
}

/// The file diffs notes are reconciled against, so a remote session can reconcile on this machine from the host's diffs.
pub(crate) trait ReviewDiffSource {
    fn show_file(&self, rev: &str, path: &str) -> JayResult<DiffHunk>;
    fn show_file_rename(&self, rev: &str, old_path: &str, new_path: &str) -> JayResult<DiffHunk>;
}

pub(crate) struct CoreReviewDiffProvider<'a, S> {
    source: &'a S,
    rev: &'a str,
    hunks: Vec<ReviewHunk>,
}

impl<'a, S: ReviewDiffSource> CoreReviewDiffProvider<'a, S> {
    pub(crate) fn new(source: &'a S, rev: &'a str, diff: Vec<DiffHunk>) -> Self {
        Self {
            source,
            rev,
            hunks: diff.into_iter().map(ReviewHunk::from).collect(),
        }
    }
}

impl Repo {
    pub fn review_notes(
        &self,
        rev: &str,
        include_resolved: bool,
    ) -> JayResult<Vec<ReviewNoteStatus>> {
        Ok(self
            .review_notes_report(&ReviewStore::load(), rev, include_resolved)?
            .notes)
    }

    pub fn review_notes_report(
        &self,
        store: &ReviewStore,
        rev: &str,
        include_resolved: bool,
    ) -> JayResult<ReviewNotesReport> {
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
    ) -> JayResult<ReviewNotesReport> {
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

    pub(crate) fn review_note_anchor(
        &self,
        rev: &str,
        path: &str,
        side: NoteSide,
        line: u32,
    ) -> JayResult<NoteAnchor> {
        let change_id = self.resolve_change_id(rev)?;
        let provider = self.review_diff_provider(rev)?;
        jayjay_review::build_note_anchor(&provider, &change_id, path, side, line)?.ok_or_else(
            || {
                JayError::review(format!(
                    "{path}:{line} ({} side) is not a changed line in this change's diff",
                    side.as_str()
                ))
            },
        )
    }

    fn resolve_change_id(&self, rev: &str) -> JayResult<String> {
        let repo = self.get_repo();
        let commit = self.resolve_commit(&repo, rev)?;
        Ok(encode_reverse_hex(commit.change_id().as_bytes()))
    }

    // self and rev share lifetime 'a because CoreReviewDiffProvider borrows both into the same field lifetime.
    fn review_diff_provider<'a>(
        &'a self,
        rev: &'a str,
    ) -> JayResult<CoreReviewDiffProvider<'a, Self>> {
        Ok(CoreReviewDiffProvider::new(
            self,
            rev,
            self.show_summary(rev)?.diff,
        ))
    }
}

impl ReviewDiffSource for Repo {
    fn show_file(&self, rev: &str, path: &str) -> JayResult<DiffHunk> {
        Repo::show_file(self, rev, path)
    }
    fn show_file_rename(&self, rev: &str, old_path: &str, new_path: &str) -> JayResult<DiffHunk> {
        Repo::show_file_rename(self, rev, old_path, new_path)
    }
}

impl<S: ReviewDiffSource> ReviewDiffProvider for CoreReviewDiffProvider<'_, S> {
    fn review_hunks(&self) -> JayResult<Vec<ReviewHunk>> {
        Ok(self.hunks.clone())
    }

    fn review_file_diff(&self, hunk: &ReviewHunk) -> JayResult<ReviewFileDiff> {
        let source = hunk
            .old_path
            .as_deref()
            .filter(|_| hunk.hunk_type.has_source());
        let diff = match source {
            Some(old_path) => self
                .source
                .show_file_rename(self.rev, old_path, &hunk.path)?,
            None => self.source.show_file(self.rev, &hunk.path)?,
        };
        Ok(ReviewFileDiff::from(&diff))
    }
}
