use std::collections::BTreeSet;

use gpui::{Context, KeyDownEvent};
use jayjay_core::{DiffEditDestination, DiffEditFileSelection, DiffEditRange, HunkType};

use super::RepoWindow;
use super::diff_edit_state::hunk_supports_diff_edit;
use super::diff_edit_view::DiffEditSnapshot;
use crate::repo::view_model::mutations::DiffEditApplyRequest;
use crate::ui::input::LineInput;

const EMPTY_SELECTION_MESSAGE: &str =
    "Select at least one file, hunk, or line before applying diff edit.";

impl RepoWindow {
    pub fn diff_edit_selection_summary(&self) -> (usize, usize) {
        self.diff_edit.summary
    }

    pub(super) fn diff_edit_selection_text(&self) -> String {
        let (lines, files) = self.diff_edit_selection_summary();
        if lines == 0 {
            return "Select files, hunks, or line ranges to edit".into();
        }
        format!(
            "{files} {}, {lines} {} selected",
            if files == 1 { "file" } else { "files" },
            if lines == 1 { "line" } else { "lines" }
        )
    }

    pub(super) fn diff_edit_should_deselect(&self) -> bool {
        self.diff_edit.selecting_all
            || !self.diff_edit.select_all_pending.is_empty()
            || self
                .diff_edit
                .selected
                .values()
                .any(|lines| !lines.is_empty())
    }

    pub(super) fn diff_edit_is_working_copy(&self, cx: &Context<Self>) -> bool {
        self.vm
            .read(cx)
            .selected_change_for_file_ops()
            .is_some_and(|change| change.is_working_copy)
    }

    pub fn diff_edit_snapshot(&self) -> DiffEditSnapshot {
        let working_copy = self.diff_edit.working_copy;
        let (selected_lines, selected_files) = self.diff_edit_selection_summary();
        let destinations = if working_copy {
            vec![DiffEditDestination::RemoveFromSource]
        } else {
            vec![
                DiffEditDestination::NewChild,
                DiffEditDestination::NewParallel,
                DiffEditDestination::MoveToWorkingCopy,
                DiffEditDestination::RemoveFromSource,
            ]
        };
        DiffEditSnapshot {
            active: self.diff_edit.active,
            working_copy,
            description: self.diff_edit.message.text().to_owned(),
            destinations,
            selected_files,
            selected_lines,
        }
    }

    pub(super) fn activate_diff_edit_message(&mut self, cx: &mut Context<Self>) {
        self.diff_edit.message_active = true;
        LineInput::show_for_owner(self, cx, Self::diff_edit_message_input);
        cx.notify();
    }

    pub(super) fn handle_diff_edit_key(
        &mut self,
        event: &KeyDownEvent,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.diff_edit.active || !self.diff_edit.message_active {
            return false;
        }
        if event.keystroke.key == "escape" {
            self.diff_edit.message_active = false;
            LineInput::hide_for_owner(self, cx, Self::diff_edit_message_input);
        } else if self.diff_edit.message.handle_key(event, cx).changed {
            cx.notify();
        }
        true
    }

    pub fn set_diff_edit_message(&mut self, message: impl Into<String>, cx: &mut Context<Self>) {
        self.diff_edit.message.set_text(message);
        cx.notify();
    }

    pub fn start_diff_edit_apply(
        &mut self,
        destination: DiffEditDestination,
        cx: &mut Context<Self>,
    ) {
        if self.diff_edit_selection_summary().0 == 0 {
            self.show_toast(EMPTY_SELECTION_MESSAGE, cx);
            return;
        }
        let message = self.diff_edit.message.text().to_owned();
        self.apply_diff_edit(destination, message, cx);
    }

    fn apply_diff_edit(
        &mut self,
        destination: DiffEditDestination,
        message: String,
        cx: &mut Context<Self>,
    ) {
        let Some(request) = self.build_diff_edit_request(destination, message, cx) else {
            self.show_toast(EMPTY_SELECTION_MESSAGE, cx);
            return;
        };
        self.vm
            .update(cx, |vm, cx| vm.apply_diff_edit(request, cx))
            .detach();
        self.exit_diff_edit(cx);
    }

    fn build_diff_edit_request(
        &self,
        destination: DiffEditDestination,
        message: String,
        cx: &Context<Self>,
    ) -> Option<DiffEditApplyRequest> {
        let vm = self.vm.read(cx);
        let change = vm.selected_change_for_file_ops()?;
        if self.diff_edit.change_id.as_deref() != Some(change.change_id.id.as_str()) {
            return None;
        }
        let hunks = vm.files.as_ref()?;
        let inverse = destination == DiffEditDestination::RemoveFromSource;
        let mut selections = Vec::new();
        for hunk in hunks.iter().filter(|hunk| hunk_supports_diff_edit(hunk)) {
            let Some(loaded) = self.diff_edit.loaded_files.get(&hunk.path) else {
                continue;
            };
            let selected = self
                .diff_edit
                .selected
                .get(&hunk.path)
                .cloned()
                .unwrap_or_default();
            let lines = if inverse {
                loaded.changed.difference(&selected).copied().collect()
            } else {
                selected
            };
            if let Some(selection) = file_selection(hunk, loaded, &lines) {
                selections.push(selection);
            }
        }
        if selections.is_empty() {
            return None;
        }
        Some(DiffEditApplyRequest {
            rev: crate::repo::revset::change_revision(change),
            destination,
            selections,
            message,
            ignore_whitespace: vm.ignore_whitespace,
            restore_path: vm.selected_hunk()?.path.clone(),
        })
    }

    fn diff_edit_message_input(view: &mut Self) -> Option<&mut LineInput> {
        view.diff_edit.active.then_some(&mut view.diff_edit.message)
    }
}

fn file_selection(
    hunk: &jayjay_core::DiffHunk,
    loaded: &super::diff_edit_state::DiffEditLoadedFile,
    lines: &BTreeSet<u32>,
) -> Option<DiffEditFileSelection> {
    (!lines.is_empty()).then(|| DiffEditFileSelection {
        path: hunk.path.clone(),
        old_path: hunk.old_path.clone(),
        old_content: (hunk.hunk_type != HunkType::Added).then(|| loaded.old_content.to_string()),
        new_content: (hunk.hunk_type != HunkType::Removed).then(|| loaded.new_content.to_string()),
        hunk_type: hunk.hunk_type,
        line_ranges: contiguous_ranges(lines),
    })
}

fn contiguous_ranges(lines: &BTreeSet<u32>) -> Vec<DiffEditRange> {
    let mut ranges = Vec::new();
    for line in lines.iter().copied() {
        match ranges.last_mut() {
            Some(DiffEditRange { end_line, .. }) if *end_line + 1 == line => *end_line = line,
            _ => ranges.push(DiffEditRange {
                start_line: line,
                end_line: line,
            }),
        }
    }
    ranges
}
