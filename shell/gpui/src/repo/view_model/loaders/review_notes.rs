use std::collections::HashMap;
use std::sync::Arc;

use gpui::Context;
use jayjay_review::{NoteEntry, NoteStatus, ReviewNoteStatus};

use super::super::RepoViewModel;

impl RepoViewModel {
    /// Cached: recomputed only in `set_review_notes`, not on every render that reads it.
    pub fn active_note_counts(&self) -> Arc<HashMap<String, usize>> {
        self.active_note_counts_cache.clone()
    }

    pub fn stale_or_orphaned_notes(&self) -> Vec<ReviewNoteStatus> {
        self.review_notes
            .iter()
            .filter(|s| matches!(s.status, NoteStatus::Stale | NoteStatus::Orphaned))
            .cloned()
            .collect()
    }

    /// Takes notes as an owned parameter rather than reading the shared store: `SharedReviewStore` is `Rc<RefCell<_>>` (not `Send`), so only this diff-walk half can run off the main thread.
    pub(in crate::repo) fn load_review_notes(
        &mut self,
        notes: Vec<NoteEntry>,
        cx: &mut Context<Self>,
    ) {
        self.loading.review_notes_gen = self.loading.review_notes_gen.wrapping_add(1);
        let generation = self.loading.review_notes_gen;

        if notes.is_empty() || !self.shows_review_controls() {
            return self.clear_review_notes(cx);
        }
        let (Some(repo), Some(rev)) = (self.repo.clone(), self.selected_revision()) else {
            return self.clear_review_notes(cx);
        };

        Self::background_update(
            cx,
            async move { repo.reconcile_review_notes(notes, &rev) },
            move |vm, result, cx| {
                if vm.loading.review_notes_gen != generation {
                    return;
                }
                // Keep last known statuses on a transient read error — clearing would silently hide the stale-notes banner and every gutter dot.
                if let Ok(report) = result {
                    vm.set_review_notes(report.notes);
                }
                cx.notify();
            },
        );
    }

    fn clear_review_notes(&mut self, cx: &mut Context<Self>) {
        self.set_review_notes(Vec::new());
        cx.notify();
    }

    /// The only place `review_notes` is written, keeping it and `active_note_counts_cache` in lockstep — writing either separately would desync readers from the cache.
    fn set_review_notes(&mut self, notes: Vec<ReviewNoteStatus>) {
        let mut counts = HashMap::new();
        for status in &notes {
            if status.status == NoteStatus::Current {
                *counts.entry(status.note.path.clone()).or_insert(0) += 1;
            }
        }
        self.review_notes = notes;
        self.active_note_counts_cache = Arc::new(counts);
    }
}
