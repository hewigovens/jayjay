use std::io;
use std::path::Path;

use jayjay_primitives::{
    ChangeDetail, DiffHunk, JayError, JayResult, NoteEntry, ReviewFileDiff, ReviewMarkSource,
    ReviewNoteStatus,
};
use jayjay_review::ReviewStore;
use jj_diff::ReviewFileSnapshot;
use jj_lib::object_id::ObjectId;

use super::super::Repo;
use super::super::review_snapshot::{
    review_display_group_map_from_hunk, review_snapshot_from_hunk,
};
use super::super::support::{block_on_result, load_workspace_internal};
use super::models::{
    WorkingCopyReviewFile, WorkingCopyReviewMark, WorkingCopyReviewNote, WorkingCopyReviewStatus,
};

/// An immutable view of @, or of another mutable change, that neither snapshots files nor reconciles operation heads.
pub struct WorkingCopyReview {
    repo: Repo,
    detail: ChangeDetail,
    rev: String,
}

impl WorkingCopyReview {
    pub fn open(path: &Path) -> JayResult<Self> {
        Self::open_rev(path, "@")
    }

    /// Writes are keyed by this change and refused once it is rewritten; immutable changes and revsets are refused.
    pub fn open_change(path: &Path, change_id: &str) -> JayResult<Self> {
        if change_id.is_empty() || !change_id.chars().all(|c| c.is_ascii_lowercase()) {
            return Err(JayError::review("Name the change by its change id"));
        }
        let mut review = Self::open_rev(path, &format!("change_id({change_id})"))?;
        if review.detail.info.is_immutable {
            return Err(JayError::review("Immutable changes are read-only"));
        }
        review.rev = format!("change_id({})", review.change_id());
        Ok(review)
    }

    fn open_rev(path: &Path, rev: &str) -> JayResult<Self> {
        let repo = Self::load_repository(path)?;
        let detail = repo.show_summary(rev)?;
        Ok(Self {
            repo,
            detail,
            rev: rev.to_owned(),
        })
    }

    fn load_repository(path: &Path) -> JayResult<Repo> {
        let workspace = load_workspace_internal(path, "open review workspace")?;
        let loader = workspace.repo_loader();
        let heads = block_on_result(
            "read review operation heads",
            loader.op_heads_store().get_op_heads(),
        )?;
        let [head] = heads.as_slice() else {
            return Err(JayError::review(
                "Concurrent repository operations; retry after they are reconciled",
            ));
        };
        let operation = block_on_result("read review operation", loader.load_operation(head))?;
        let loaded = block_on_result("load review operation", loader.load_at(&operation))?;
        Ok(Repo::from_workspace(workspace, loaded))
    }

    pub fn commit_id(&self) -> &str {
        &self.detail.info.commit_id.id
    }

    pub fn change_id(&self) -> &str {
        &self.detail.info.change_id.id
    }

    pub fn files(&self) -> &[DiffHunk] {
        &self.detail.diff
    }

    pub fn load_file(&self, path: &str) -> JayResult<DiffHunk> {
        let listed = self
            .detail
            .diff
            .iter()
            .find(|file| file.path == path)
            .ok_or_else(|| JayError::review(format!("{path} is not part of this change's diff")))?;
        self.repo.load_listed_review_hunk(self.commit_id(), listed)
    }

    pub fn status(
        &self,
        store: &ReviewStore,
        ignore_whitespace: bool,
    ) -> JayResult<WorkingCopyReviewStatus> {
        let agent_paths = store.paths_owned_by(self.change_id(), ReviewMarkSource::Agent);
        let files = self
            .files()
            .iter()
            .map(|listed| self.file_status(store, listed, ignore_whitespace, &agent_paths))
            .collect::<JayResult<Vec<_>>>()?;
        Ok(WorkingCopyReviewStatus {
            change: self.detail.info.clone(),
            files,
        })
    }

    fn file_status(
        &self,
        store: &ReviewStore,
        listed: &DiffHunk,
        ignore_whitespace: bool,
        agent_paths: &[String],
    ) -> JayResult<WorkingCopyReviewFile> {
        let diff = self
            .repo
            .load_listed_review_hunk(self.commit_id(), listed)?;
        let snapshot = review_snapshot_from_hunk(&diff);
        let display_groups = review_display_group_map_from_hunk(&diff, ignore_whitespace);
        let state = store.file_review_state(
            self.change_id(),
            &diff.path,
            &diff.review_identity,
            Some(&snapshot),
        );
        let group_states = store.display_hunk_states(
            self.change_id(),
            &diff.path,
            &diff.review_identity,
            &snapshot,
            &display_groups,
        );
        let source = if agent_paths.contains(&diff.path) {
            ReviewMarkSource::Agent
        } else {
            ReviewMarkSource::User
        };
        Ok(WorkingCopyReviewFile {
            diff,
            rollup: state.rollup(),
            source,
            snapshot,
            display_groups,
            group_states,
        })
    }

    pub fn set_reviewed(
        &self,
        store: &mut ReviewStore,
        mark: &WorkingCopyReviewMark,
    ) -> JayResult<()> {
        if mark.expected_commit != self.commit_id() {
            return Err(JayError::review(
                "Expected commit does not match the inspected review",
            ));
        }
        let hunk = self.load_file(&mark.path)?;
        if hunk.review_identity.is_empty() {
            return Err(JayError::review("This file cannot carry review marks"));
        }
        let snapshot = review_snapshot_from_hunk(&hunk);
        let mapping = review_display_group_map_from_hunk(&hunk, mark.ignore_whitespace);
        let indices = mark
            .display_group
            .map(|group| {
                mapping
                    .get(group as usize)
                    .filter(|indices| !indices.is_empty())
                    .map(Vec::as_slice)
                    .ok_or_else(|| JayError::review("Unknown review group; refresh the diff"))
            })
            .transpose()?;
        self.ensure_unchanged(&mark.expected_commit)?;
        store
            .transaction(|store| self.apply_mark(store, mark, &hunk, &snapshot, indices))
            .map_err(|error| JayError::review(format!("Could not save review marks: {error}")))
    }

    fn apply_mark(
        &self,
        store: &mut ReviewStore,
        mark: &WorkingCopyReviewMark,
        hunk: &DiffHunk,
        snapshot: &ReviewFileSnapshot,
        indices: Option<&[u32]>,
    ) -> io::Result<()> {
        let (change_id, path, identity) = (self.change_id(), &mark.path, &hunk.review_identity);
        match (indices, mark.reviewed) {
            (Some(indices), true) => store.mark_groups_reviewed_as(
                change_id,
                path,
                identity,
                snapshot,
                indices,
                ReviewMarkSource::User,
            ),
            (Some(indices), false) => {
                for &index in indices {
                    store.mark_hunk_unreviewed_snapshot(change_id, path, identity, snapshot, index);
                }
                Ok(())
            }
            (None, true) => store.mark_reviewed_as(
                change_id,
                path,
                identity,
                Some(snapshot),
                ReviewMarkSource::User,
            ),
            (None, false) => {
                store.mark_unreviewed(change_id, path);
                Ok(())
            }
        }
    }

    pub fn notes(&self, store: &ReviewStore) -> JayResult<Vec<ReviewNoteStatus>> {
        Ok(self
            .repo
            .review_notes_report(store, self.commit_id(), true)?
            .notes)
    }

    pub fn add_note(
        &self,
        store: &mut ReviewStore,
        note: &WorkingCopyReviewNote,
    ) -> JayResult<NoteEntry> {
        if note.expected_commit != self.commit_id() {
            return Err(JayError::review(
                "Expected commit does not match the inspected review",
            ));
        }
        let body = jayjay_review::checked_note_body(&note.body)?;
        let hunk = self.load_file(&note.path)?;
        let anchor = jayjay_review::note_anchor(
            self.change_id(),
            &hunk.path,
            &hunk.review_identity,
            &ReviewFileDiff::from(&hunk),
            note.side,
            note.line,
            note.ignore_whitespace,
        )
        .ok_or_else(|| JayError::review("Notes must anchor to a changed line"))?;
        self.ensure_unchanged(&note.expected_commit)?;
        store
            .transaction(|store| Ok(store.add_note(anchor, body)))
            .map_err(|error| JayError::review(format!("Could not save the review note: {error}")))
    }

    pub fn resolve_note(&self, store: &mut ReviewStore, id: &str) -> JayResult<NoteEntry> {
        let change_id = self.change_id();
        store
            .transaction(|store| {
                if !store
                    .list_notes(change_id, true)
                    .iter()
                    .any(|note| note.id == id)
                {
                    return Err(io::Error::other("Unknown review note"));
                }
                store
                    .resolve_note(id)
                    .ok_or_else(|| io::Error::other("Unknown review note"))
            })
            .map_err(|error| {
                JayError::review(format!("Could not resolve the review note: {error}"))
            })
    }

    // Checked before taking the store lock so slow repository loads never block other store readers or writers; a rewrite after this check still reconciles as changed since review.
    fn ensure_unchanged(&self, expected_commit: &str) -> JayResult<()> {
        let current = Self::load_repository(self.repo.path())?;
        let current_commit = current
            .resolve_commit(&current.get_repo(), &self.rev)?
            .id()
            .hex();
        if current_commit != expected_commit {
            return Err(JayError::review(format!(
                "{} changed since inspection (expected {expected_commit}, current {current_commit}); refresh before marking",
                if self.rev == "@" {
                    "Working copy"
                } else {
                    "The change"
                }
            )));
        }
        Ok(())
    }
}
