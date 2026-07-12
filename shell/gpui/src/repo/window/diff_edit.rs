use std::collections::BTreeSet;

use gpui::Context;
use jayjay_core::diff::change_groups;

use super::diff_edit_state::{checkbox_state, hunk_supports_diff_edit, next_diff_edit_session};
use super::{DiffEditCheckboxState, DiffEditState, RepoWindow};

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
        self.diff_edit.session = next_diff_edit_session();
        self.diff_edit.change_id = Some(change_id);
        self.diff_edit.working_copy = working_copy;
        self.diff_edit.message.set_text(description);
        self.ensure_diff_edit_files(cx);
        if let Some(files) = self.vm.read(cx).files.clone() {
            self.vm
                .update(cx, |vm, cx| vm.preload_diffs_async(files, cx));
        }
        cx.notify();
    }

    pub fn exit_diff_edit(&mut self, cx: &mut Context<Self>) {
        if self.diff_edit.active {
            self.diff_edit = DiffEditState::default();
            cx.notify();
        }
    }

    pub fn diff_edit_active(&self) -> bool {
        self.diff_edit.active
    }

    pub fn diff_edit_selecting_all(&self) -> bool {
        self.diff_edit.selecting_all
    }

    pub fn diff_edit_selected(&self, path: &str) -> BTreeSet<u32> {
        self.diff_edit
            .selected
            .get(path)
            .cloned()
            .unwrap_or_default()
    }

    pub fn diff_edit_file_state(
        &mut self,
        path: &str,
        _cx: &Context<Self>,
    ) -> DiffEditCheckboxState {
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
        if self.diff_edit.selecting_all {
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
        self.diff_edit.selecting_all = true;
        self.diff_edit.select_all_pending = paths;
        self.drain_select_all_pending();
        if self.diff_edit.selecting_all {
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
        if self.diff_edit.select_all_pending.is_empty() {
            self.diff_edit.selecting_all = false;
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
