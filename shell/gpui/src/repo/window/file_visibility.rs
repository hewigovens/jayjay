//! File-list visibility filtering: the two independent toggles beside the file-column header (hide-reviewed, noted-files-only) and the shared index math both apply.

use std::collections::HashMap;

use gpui::{App, Context, ScrollStrategy};
use jayjay_core::DiffHunk;

use super::RepoWindow;

impl RepoWindow {
    /// `active_note_counts` gates the noted-files filter the same way `change_id` gates hide-reviewed: both filters only apply under `show_review`, and each is a no-op unless its own toggle is also on.
    pub(crate) fn visible_file_indices(
        &self,
        files: &[DiffHunk],
        change_id: Option<&str>,
        show_review: bool,
        active_note_counts: Option<&HashMap<String, usize>>,
    ) -> Vec<usize> {
        let hide_reviewed = show_review && self.file_column.hide_reviewed;
        let notes_only = show_review && self.file_column.notes_only;
        if !hide_reviewed && !notes_only {
            return (0..files.len()).collect();
        }
        files
            .iter()
            .enumerate()
            .filter(|(_, hunk)| {
                if hide_reviewed
                    && change_id
                        .is_some_and(|cid| self.is_reviewed(cid, &hunk.path, &hunk.review_identity))
                {
                    return false;
                }
                if notes_only
                    && !active_note_counts.is_some_and(|counts| counts.contains_key(&hunk.path))
                {
                    return false;
                }
                true
            })
            .map(|(ix, _)| ix)
            .collect()
    }

    pub(crate) fn visible_indices(
        &self,
        files: &[DiffHunk],
        change_id: Option<&str>,
        show_review: bool,
        cx: &App,
    ) -> Vec<usize> {
        let counts = self.vm.read(cx).active_note_counts();
        self.visible_file_indices(files, change_id, show_review, Some(&counts))
    }

    /// `pub` wrapper around `visible_indices` so the separate `tests/` crate can assert the filter's effect without reaching `pub(crate)` state.
    pub fn visible_file_paths(&self, cx: &App) -> Vec<String> {
        let Some(files) = self.vm.read(cx).files.clone() else {
            return Vec::new();
        };
        let (show_review, change_id) = self.review_file_context(cx);
        self.visible_indices(&files, change_id.as_deref(), show_review, cx)
            .into_iter()
            .map(|ix| files[ix].path.clone())
            .collect()
    }

    /// Resolving or deleting the last active note must drop the notes-only filter too, or the list would pin to empty with no control left to clear it.
    pub(crate) fn clear_notes_only_if_empty(&mut self, cx: &Context<Self>) {
        if self.file_column.notes_only && self.vm.read(cx).active_note_counts().is_empty() {
            self.file_column.notes_only = false;
        }
    }

    pub fn toggle_hide_reviewed_files(&mut self, cx: &mut Context<Self>) {
        self.toggle_file_filter(cx, |fc| &mut fc.hide_reviewed);
    }

    pub fn toggle_notes_only_files(&mut self, cx: &mut Context<Self>) {
        self.toggle_file_filter(cx, |fc| &mut fc.notes_only);
    }

    fn toggle_file_filter(
        &mut self,
        cx: &mut Context<Self>,
        field: impl FnOnce(&mut super::view::FileColumnUiState) -> &mut bool,
    ) {
        let flag = field(&mut self.file_column);
        *flag ^= true;
        if *flag {
            self.jump_to_first_visible_file_if_current_is_hidden(cx);
        }
        cx.notify();
    }

    /// If enabling a filter hides the current file, jumps to the first still-visible one; skips its own `cx.notify()` here since `select_file`/`scroll_to_item` already notify.
    fn jump_to_first_visible_file_if_current_is_hidden(&mut self, cx: &mut Context<Self>) {
        let (show_review, change_id) = self.review_file_context(cx);
        let vm = self.vm.read(cx);
        let (files, selected) = (vm.files.clone(), vm.selected_file_ix);
        let visible = files
            .map(|files| self.visible_indices(&files, change_id.as_deref(), show_review, cx))
            .unwrap_or_default();
        if selected.is_some_and(|ix| !visible.contains(&ix))
            && let Some(next) = visible.first().copied()
        {
            self.select_file(next, cx);
            self.scrolls.files.scroll_to_item(0, ScrollStrategy::Top);
        }
    }
}
