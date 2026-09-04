//! Single source RepoWindow/RepoViewModel state feeds into the pure `diff::rows` builder, so unified_body, find.rs's scroll targeting, and tests all see the same interleaved row list.

use std::sync::Arc;

use gpui::{App, px};
use jayjay_review::ReviewNoteStatus;

use super::RepoWindow;
use crate::app::fonts;
use crate::app::theme::theme;
use crate::diff::DiffRenderRows;
use crate::diff::wrap::wrap_cols_from_bounds;

impl RepoWindow {
    /// Filters `vm.review_notes` (which covers every file in the change) down to the selected hunk, or a file's diff would show another file's notes as its own rows/dots.
    pub(crate) fn notes_for_selected_hunk(&self, cx: &App) -> Vec<ReviewNoteStatus> {
        let vm = self.vm.read(cx);
        let Some(hunk) = vm.selected_hunk() else {
            return Vec::new();
        };
        if self.review_notes_context(hunk, cx).is_none() {
            return Vec::new();
        }
        vm.review_notes
            .iter()
            .filter(|status| {
                status.note.path == hunk.path && status.note.identity == hunk.review_identity
            })
            .cloned()
            .collect()
    }

    /// The single row list unified_body's rendering and find.rs's scroll targeting both read, so a note above a match can never desync the two; `None` while no diff is loaded yet.
    pub fn diff_render_rows(&self, cx: &App) -> Option<Arc<DiffRenderRows>> {
        let notes = self.notes_for_selected_hunk(cx);
        let fd = self.vm.read(cx).current_diff.clone()?;
        let advance = fonts::mono_advance(cx, px(theme(cx).font_size));
        let cols = wrap_cols_from_bounds(self.diff.unified_bounds.get(), advance);
        Some(self.diff.wrap_cache.borrow_mut().rows(&fd, cols, &notes))
    }
}
