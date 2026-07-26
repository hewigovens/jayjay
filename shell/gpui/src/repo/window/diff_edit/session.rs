use std::collections::BTreeSet;

use gpui::Context;
use jayjay_core::diff::change_groups;

use super::state::{DiffEditCheckboxState, DiffEditState};
use super::state::{checkbox_state, hunk_supports_diff_edit, next_diff_edit_session};
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
        self.diff_edit.session = next_diff_edit_session();
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
        !self.diff_edit.select_all_pending.is_empty()
    }

    pub fn diff_edit_selected(&self, path: &str) -> BTreeSet<u32> {
        self.diff_edit
            .selected
            .get(path)
            .cloned()
            .unwrap_or_default()
    }

    pub fn diff_edit_file_state(&self, path: &str) -> DiffEditCheckboxState {
        let Some(loaded) = self.diff_edit.loaded_files.get(path) else {
            return DiffEditCheckboxState::None;
        };
        checkbox_state(self.diff_edit.selected.get(path), &loaded.changed)
    }

    pub fn toggle_diff_edit_display_line(
        &mut self,
        path: &str,
        display_line: u32,
        cx: &mut Context<Self>,
    ) {
        let Some(loaded) = self.diff_edit.loaded_files.get(path) else {
            return;
        };
        let Some(full_line) = loaded.display_to_full.get(&display_line).copied() else {
            return;
        };
        if !loaded.changed.contains(&full_line) {
            return;
        }
        let selected = self.diff_edit.selected.entry(path.to_owned()).or_default();
        if !selected.remove(&full_line) {
            selected.insert(full_line);
        }
        self.refresh_diff_edit_summary();
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
            .filter(|line| loaded.changed.contains(line))
            .collect();
        self.diff_edit
            .selected
            .entry(path.to_owned())
            .or_default()
            .extend(lines);
        self.refresh_diff_edit_summary();
        cx.notify();
    }

    pub fn toggle_diff_edit_file(&mut self, path: &str, cx: &mut Context<Self>) {
        let Some(loaded) = self.diff_edit.loaded_files.get(path) else {
            return;
        };
        let lines = loaded.changed.clone();
        if lines.is_empty() {
            return;
        }
        let selected = self.diff_edit.selected.entry(path.to_owned()).or_default();
        if lines.is_subset(selected) {
            selected.clear();
        } else {
            selected.extend(lines.iter().copied());
        }
        self.refresh_diff_edit_summary();
        cx.notify();
    }

    pub fn toggle_diff_edit_all(&mut self, cx: &mut Context<Self>) {
        if self.diff_edit_selecting_all() {
            return;
        }
        let has_selection = self
            .diff_edit
            .selected
            .values()
            .any(|lines| !lines.is_empty());
        if has_selection || !self.diff_edit.select_all_pending.is_empty() {
            self.diff_edit.select_all_pending.clear();
            for selected in self.diff_edit.selected.values_mut() {
                selected.clear();
            }
            self.refresh_diff_edit_summary();
            cx.notify();
            return;
        }
        let (paths, files) = self
            .vm
            .read(cx)
            .files
            .as_ref()
            .map(|files| {
                let paths = files
                    .iter()
                    .filter(|hunk| hunk_supports_diff_edit(hunk))
                    .map(|hunk| hunk.path.clone())
                    .collect();
                (paths, files.clone())
            })
            .unwrap_or_default();
        self.diff_edit.select_all_pending = paths;
        self.drain_select_all_pending();
        if self.diff_edit_selecting_all() {
            self.ensure_diff_edit_files(cx);
            self.vm
                .update(cx, |vm, cx| vm.preload_diffs_async(files, cx));
        }
        cx.notify();
    }

    fn drain_select_all_pending(&mut self) {
        let ready: Vec<String> = self
            .diff_edit
            .select_all_pending
            .iter()
            .filter(|path| {
                self.diff_edit.loaded_files.contains_key(*path)
                    || self.diff_edit.known_unsupported.contains(*path)
            })
            .cloned()
            .collect();
        for path in ready {
            self.diff_edit.select_all_pending.remove(&path);
            if let Some(loaded) = self.diff_edit.loaded_files.get(&path) {
                let lines = (*loaded.changed).clone();
                self.diff_edit.selected.insert(path, lines);
            }
        }
        self.refresh_diff_edit_summary();
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
            // A new session token kills every in-flight completion (card loads, stats) so a superseded compute can't reinstall old-epoch state over the cleared maps.
            self.diff_edit.session = next_diff_edit_session();
            // Old-epoch badges must not outlive the reset; the per-file pass rebuilds the folds from fresh stats.
            self.diff_edit.stats = None;
            self.diff_edit.loaded_files.clear();
            self.diff_edit.known_unsupported.clear();
            self.diff_edit.loading.clear();
            self.diff_edit.selected.clear();
            self.diff_edit.select_all_pending.clear();
            self.diff_edit.rows = None;
            self.refresh_diff_edit_summary();
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
