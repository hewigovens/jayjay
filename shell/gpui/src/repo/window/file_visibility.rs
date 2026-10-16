//! File-list visibility filtering: the Git LFS/submodule settings, the filename query and the independent hide-reviewed and noted-files-only toggles share one index calculation.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use gpui::{App, Context, Window};
use jayjay_core::DiffHunk;
use jayjay_review::ReviewFileRollup;

use super::RepoWindow;
use crate::diff::file_status::{is_lfs, is_submodule};
use crate::ui::input::LineInput;

pub(crate) struct HiddenGitFiles {
    pub(crate) lfs: usize,
    pub(crate) submodules: usize,
}

fn lfs_hidden(hunk: &DiffHunk, hide_lfs: bool, tracked_lfs_paths: &HashSet<String>) -> bool {
    hide_lfs && (is_lfs(hunk) || tracked_lfs_paths.contains(&hunk.path))
}

fn submodule_hidden(hunk: &DiffHunk, submodule_support: bool) -> bool {
    !submodule_support && is_submodule(hunk)
}

impl RepoWindow {
    /// `active_note_counts` gates the noted-files filter the same way `change_id` gates hide-reviewed; the filename query applies in every file-list mode.
    pub(crate) fn visible_file_indices(
        &self,
        files: &[DiffHunk],
        show_review: bool,
        active_note_counts: Option<&HashMap<String, usize>>,
        review_rollups: Option<&HashMap<String, ReviewFileRollup>>,
        tracked_lfs_paths: &HashSet<String>,
    ) -> Vec<usize> {
        let hide_reviewed = show_review && self.file_column.hide_reviewed;
        let notes_only = show_review && self.file_column.notes_only;
        let hide_lfs = self.file_column.hide_git_lfs;
        let submodule_support = self.file_column.enable_git_submodule_support;
        let query = self
            .file_column
            .filter
            .as_ref()
            .map(LineInput::text)
            .unwrap_or_default()
            .to_lowercase();
        if query.is_empty() && !hide_reviewed && !notes_only && !hide_lfs && submodule_support {
            return (0..files.len()).collect();
        }
        files
            .iter()
            .enumerate()
            .filter(|(_, hunk)| {
                if lfs_hidden(hunk, hide_lfs, tracked_lfs_paths)
                    || submodule_hidden(hunk, submodule_support)
                {
                    return false;
                }
                if !query.is_empty() && !hunk.path.to_lowercase().contains(&query) {
                    return false;
                }
                if hide_reviewed
                    && review_rollups.is_some_and(|rollups| {
                        rollups.get(&hunk.path) == Some(&ReviewFileRollup::Reviewed)
                    })
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
        let rollups = change_id
            .filter(|_| show_review)
            .map(|change_id| {
                let vm = self.vm.read(cx);
                self.review_rollups_with_vm(change_id, files.iter(), vm)
            })
            .unwrap_or_default();
        let lfs_paths = self.vm.read(cx).tracked_lfs_paths.clone();
        self.visible_file_indices(
            files,
            show_review,
            Some(&counts),
            Some(&rollups),
            &lfs_paths,
        )
    }

    pub(crate) fn hidden_only_files(
        &self,
        files: &[DiffHunk],
        tracked_lfs_paths: &HashSet<String>,
    ) -> Option<HiddenGitFiles> {
        let (mut lfs, mut submodules, mut visible) = (0, 0, 0);
        for hunk in files {
            let lfs_hidden = lfs_hidden(hunk, self.file_column.hide_git_lfs, tracked_lfs_paths);
            let submodule_hidden =
                submodule_hidden(hunk, self.file_column.enable_git_submodule_support);
            lfs += usize::from(lfs_hidden);
            submodules += usize::from(submodule_hidden);
            visible += usize::from(!lfs_hidden && !submodule_hidden);
        }
        (!files.is_empty() && visible == 0).then_some(HiddenGitFiles { lfs, submodules })
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

    pub(crate) fn toggle_hide_reviewed_files(&mut self, cx: &mut Context<Self>) {
        crate::app::config::update(cx, |c| c.diff.hide_reviewed_files ^= true);
    }

    pub fn toggle_notes_only_files(&mut self, cx: &mut Context<Self>) {
        self.file_column.notes_only ^= true;
        self.reconcile_file_selection(cx);
        cx.notify();
    }

    pub(crate) fn file_filter_input(view: &mut Self) -> Option<&mut LineInput> {
        view.file_column.filter.as_mut()
    }

    pub(crate) fn toggle_file_filter(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.file_column.filter.is_some() {
            self.dismiss_file_filter(window, cx);
            return;
        }
        self.file_column.filter = Some(LineInput::default());
        self.file_filter_focus.focus(window, cx);
        LineInput::show_for_owner(self, cx, Self::file_filter_input);
        cx.notify();
    }

    pub(crate) fn close_file_filter(&mut self, cx: &mut Context<Self>) {
        LineInput::hide_for_owner(self, cx, Self::file_filter_input);
        self.file_column.filter = None;
        cx.notify();
    }

    pub(crate) fn dismiss_file_filter(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.close_file_filter(cx);
        self.focus_handle.focus(window, cx);
        self.focus_file_list(cx);
    }

    pub(crate) fn activate_file_filter(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.file_filter_focus.focus(window, cx);
        LineInput::show_for_owner(self, cx, Self::file_filter_input);
        cx.notify();
    }

    pub(crate) fn handle_file_filter_key(
        &mut self,
        event: &gpui::KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        // Escape arrives as the window's `Dismiss` action instead; see `render/interaction.rs`.
        if event.keystroke.key == "enter" {
            self.focus_handle.focus(window, cx);
            self.focus_file_list(cx);
            return true;
        }
        let Some(input) = self.file_column.filter.as_mut() else {
            return false;
        };
        let result = input.handle_key(event, cx);
        if result.handled {
            LineInput::show_for_owner(self, cx, Self::file_filter_input);
            if result.changed {
                self.reconcile_file_selection(cx);
            }
            cx.notify();
        }
        result.handled
    }

    pub(crate) fn reconcile_file_selection(&mut self, cx: &mut Context<Self>) {
        self.prune_file_multi_select(cx);
        let (show_review, change_id) = self.review_file_context(cx);
        let vm = self.vm.read(cx);
        let (files, selected) = (vm.files.clone(), vm.selected_file_ix);
        let Some(files) = files else { return };
        let visible = Arc::new(self.visible_indices(&files, change_id.as_deref(), show_review, cx));
        if selected.is_some_and(|ix| visible.contains(&ix)) {
            return;
        }
        let still_selected = self.multi_selected_hunk_indices();
        let next = if let Some(&next) = visible.iter().find(|ix| still_selected.contains(ix)) {
            self.focus_file(next, cx);
            next
        } else if let Some(&next) = visible.first() {
            self.select_file(next, cx);
            next
        } else {
            self.vm.update(cx, |vm, cx| vm.clear_file_selection(cx));
            return;
        };
        let tree_mode = crate::app::config::current(cx).diff.tree_file_list;
        self.reveal_file(next, &files, &visible, tree_mode);
    }
}
