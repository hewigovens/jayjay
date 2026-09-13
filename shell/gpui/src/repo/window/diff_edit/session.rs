use std::collections::BTreeSet;

use gpui::Context;
use jayjay_core::diff::change_groups;
use jayjay_core::diff_edit::DiffEditCheckbox;

use super::state::DiffEditState;
use super::state::{hunk_supports_diff_edit, next_diff_edit_epoch};
use crate::repo::window::{RepoWindow, TextModalAction};

impl RepoWindow {
    pub fn enter_diff_edit(&mut self, cx: &mut Context<Self>) {
        if self.diff_edit.active || !self.can_enter_diff_edit(cx) {
            return;
        }
        let (change_id, description, working_copy) = self
            .vm
            .read(cx)
            .selected_change_for_file_ops()
            .map(|change| {
                (
                    change.change_id.id.clone(),
                    change.description.clone(),
                    change.is_working_copy,
                )
            })
            .unwrap();
        self.diff_edit.active = true;
        self.diff_edit.focus_pending = true;
        // Diff edit owns the window's keys while it is open, so the Tab cycle hands its control back.
        self.focused_control = None;
        self.diff_edit.epoch = next_diff_edit_epoch();
        self.diff_edit.change_id = Some(change_id);
        self.diff_edit.working_copy = working_copy;
        self.diff_edit.message = description;
        self.diff_edit.loaded_ignore_whitespace = self.vm.read(cx).ignore_whitespace;
        self.diff_edit.loaded_commit = self.selected_commit_id(cx);
        self.seed_diff_edit_collapse(cx);
        self.spawn_diff_edit_stats(cx);
        self.ensure_diff_edit_files(cx);
        if let Some(files) = self.vm.read(cx).files.clone() {
            self.vm
                .update(cx, |vm, cx| vm.preload_diffs_async(files, cx));
        }
        cx.notify();
    }

    pub fn exit_diff_edit(&mut self, cx: &mut Context<Self>) {
        if self.diff_edit.active {
            if self.text_modal.as_ref().is_some_and(|modal| {
                matches!(&modal.action, TextModalAction::DiffEditDescription { .. })
            }) {
                self.text_modal = None;
            }
            self.diff_edit = DiffEditState::default();
            cx.notify();
        }
    }

    pub fn diff_edit_active(&self) -> bool {
        self.diff_edit.active
    }

    pub fn diff_edit_selecting_all(&self) -> bool {
        self.diff_edit.session.is_selecting_all()
    }

    pub fn diff_edit_selected(&self, path: &str) -> BTreeSet<u32> {
        self.diff_edit
            .session
            .selected_lines(path)
            .into_iter()
            .collect()
    }

    pub fn diff_edit_file_state(&self, path: &str) -> DiffEditCheckbox {
        self.diff_edit.session.checkbox(path)
    }

    pub fn toggle_diff_edit_display_line(
        &mut self,
        path: &str,
        display_line: u32,
        cx: &mut Context<Self>,
    ) {
        let Some(full_line) = self
            .diff_edit
            .loaded_files
            .get(path)
            .and_then(|loaded| loaded.display_to_full.get(&display_line).copied())
        else {
            return;
        };
        self.diff_edit.session.toggle_line(path, full_line);
        cx.notify();
    }

    pub fn select_diff_edit_display_group(
        &mut self,
        path: &str,
        display_line: u32,
        cx: &mut Context<Self>,
    ) {
        let Some(loaded) = self.diff_edit.loaded_files.get(path) else {
            return;
        };
        let Some(group) = change_groups(&loaded.display_diff.lines)
            .into_iter()
            .find(|group| (group.start_line..=group.end_line).contains(&display_line))
        else {
            return;
        };
        let lines: Vec<u32> = (group.start_line..=group.end_line)
            .filter_map(|line| loaded.display_to_full.get(&line).copied())
            .collect();
        self.diff_edit.session.select_lines(path, &lines);
        cx.notify();
    }

    pub fn toggle_diff_edit_file(&mut self, path: &str, cx: &mut Context<Self>) {
        self.diff_edit.session.toggle_file(path);
        cx.notify();
    }

    pub fn toggle_diff_edit_all(&mut self, cx: &mut Context<Self>) {
        if self.diff_edit_selecting_all() {
            return;
        }
        // A card already known to be unsupported would otherwise stay pending forever and keep Select All spinning.
        let paths: Vec<String> = self
            .vm
            .read(cx)
            .files
            .as_ref()
            .map(|files| {
                files
                    .iter()
                    .filter(|hunk| {
                        hunk_supports_diff_edit(hunk)
                            && !self.diff_edit.known_unsupported.contains(&hunk.path)
                    })
                    .map(|hunk| hunk.path.clone())
                    .collect()
            })
            .unwrap_or_default();
        if !self.diff_edit.session.toggle_all(&paths).is_empty() {
            self.ensure_diff_edit_files(cx);
            if let Some(files) = self.vm.read(cx).files.clone() {
                self.vm
                    .update(cx, |vm, cx| vm.preload_diffs_async(files, cx));
            }
        }
        cx.notify();
    }

    pub(crate) fn sync_diff_edit_loaded_files(&mut self, cx: &mut Context<Self>) {
        if !self.diff_edit.active {
            return;
        }
        if !self.diff_edit_change_is_current(cx) {
            self.exit_diff_edit(cx);
            return;
        }
        // A whitespace-mode change or an amend invalidates every loaded diff AND the selections: selected values are full-diff row indices that silently remap when rows shift, and apply would submit them under the new state.
        let ignore_whitespace = self.vm.read(cx).ignore_whitespace;
        let commit = self.selected_commit_id(cx);
        if self.diff_edit.loaded_ignore_whitespace != ignore_whitespace
            || (commit.is_some() && self.diff_edit.loaded_commit != commit)
        {
            self.diff_edit.loaded_ignore_whitespace = ignore_whitespace;
            self.diff_edit.loaded_commit = commit;
            // A new epoch kills every in-flight completion (card loads, stats) so a superseded compute can't reinstall old-epoch state over the cleared maps.
            self.diff_edit.epoch = next_diff_edit_epoch();
            // Old-epoch badges must not outlive the reset; the per-file pass rebuilds the folds from fresh stats.
            self.diff_edit.stats = None;
            self.diff_edit.session.unload();
            self.diff_edit.loaded_files.clear();
            self.diff_edit.known_unsupported.clear();
            self.diff_edit.loading.clear();
            self.diff_edit.rows = None;
            self.spawn_diff_edit_stats(cx);
            if let Some(files) = self.vm.read(cx).files.clone() {
                self.vm
                    .update(cx, |vm, cx| vm.preload_diffs_async(files, cx));
            }
        }
        self.ensure_diff_edit_files(cx);
    }

    pub(crate) fn sync_diff_edit_change(&mut self, cx: &mut Context<Self>) {
        if self.diff_edit.active && !self.diff_edit_change_is_current(cx) {
            self.exit_diff_edit(cx);
        }
    }

    pub(crate) fn can_enter_diff_edit(&self, cx: &Context<Self>) -> bool {
        self.vm
            .read(cx)
            .selected_change_for_file_ops()
            .is_some_and(|change| !change.has_conflict && !change.is_empty && !change.is_immutable)
    }

    fn diff_edit_change_is_current(&self, cx: &Context<Self>) -> bool {
        self.vm
            .read(cx)
            .selected_change_for_file_ops()
            .is_some_and(|change| {
                self.diff_edit.change_id.as_deref() == Some(change.change_id.id.as_str())
            })
    }
}
