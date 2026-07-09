use std::collections::HashMap;

use jayjay_primitives::{
    NoteEntry, NoteStatus, ReviewDiffProvider, ReviewHunk, ReviewNoteStatus, ReviewResult,
};
use jj_diff::change_group_for_anchor;

use crate::replay::{ReplayCache, diff_side};
use crate::store::ReviewStore;

impl ReviewStore {
    pub fn reconcile(
        &self,
        provider: &impl ReviewDiffProvider,
        change_id: &str,
        include_resolved: bool,
    ) -> ReviewResult<Vec<ReviewNoteStatus>> {
        reconcile_notes(self.list_notes(change_id, include_resolved), provider)
    }
}

// Split from ReviewStore::reconcile so callers with an owned, Send-safe note snapshot (e.g. GPUI's RepoWindow, whose Rc<RefCell<ReviewStore>> can't cross into a background task) can reconcile without a whole ReviewStore.
pub fn reconcile_notes(
    notes: Vec<NoteEntry>,
    provider: &impl ReviewDiffProvider,
) -> ReviewResult<Vec<ReviewNoteStatus>> {
    if notes.is_empty() {
        return Ok(vec![]);
    }

    let hunks: HashMap<String, ReviewHunk> = provider
        .review_hunks()?
        .into_iter()
        .map(|hunk| (hunk.path.clone(), hunk))
        .collect();
    let mut cache = ReplayCache::default();

    Ok(notes
        .into_iter()
        .map(|note| status_for_note(&note, provider, &hunks, &mut cache))
        .collect())
}

fn status_for_note(
    note: &NoteEntry,
    provider: &impl ReviewDiffProvider,
    hunks: &HashMap<String, ReviewHunk>,
    cache: &mut ReplayCache,
) -> ReviewNoteStatus {
    if note.resolved {
        return ReviewNoteStatus::new(note, NoteStatus::Resolved, None);
    }
    let Some(hunk) = hunks.get(&note.path) else {
        return ReviewNoteStatus::new(note, NoteStatus::Orphaned, None);
    };
    if hunk.review_identity != note.identity {
        return ReviewNoteStatus::new(note, NoteStatus::Stale, None);
    }

    let Some(lines) = cache.display_lines(note, provider, hunk) else {
        return ReviewNoteStatus::new(note, NoteStatus::Stale, None);
    };
    let group =
        change_group_for_anchor(lines, diff_side(note.side), note.line, &note.anchor_excerpt);
    match group {
        Some(group) => ReviewNoteStatus::new(note, NoteStatus::Current, Some(group.index)),
        None => ReviewNoteStatus::new(note, NoteStatus::Stale, None),
    }
}
