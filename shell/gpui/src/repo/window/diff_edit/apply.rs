use std::collections::BTreeSet;

use gpui::Context;
use jayjay_core::external_tools::diff_edit_ranges;
use jayjay_core::{DiffEditDestination, DiffEditFileSelection, HunkType};

use super::state::hunk_supports_diff_edit;
use super::view::DiffEditSnapshot;
use crate::repo::view_model::mutations::DiffEditApplyRequest;
use crate::repo::window::RepoWindow;

const EMPTY_SELECTION_MESSAGE: &str =
    "Select at least one file, hunk, or line before applying diff edit.";
const SELECTION_STILL_LOADING_MESSAGE: &str =
    "Wait for Select All to finish loading before applying diff edit.";
const FILES_STILL_LOADING_MESSAGE: &str =
    "Wait for all editable files to finish loading before applying diff edit.";

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
        !self.diff_edit.select_all_pending.is_empty()
            || self
                .diff_edit
                .selected
                .values()
                .any(|lines| !lines.is_empty())
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
            description: self.diff_edit.message.clone(),
            destinations,
            selected_files,
            selected_lines,
        }
    }

    pub fn set_diff_edit_message(&mut self, message: impl Into<String>, cx: &mut Context<Self>) {
        self.diff_edit.message = message.into();
        cx.notify();
    }

    /// (change-id subtitle, current message, session) for the description modal; None while inactive or on the working copy.
    pub(crate) fn diff_edit_description_context(&self) -> Option<(String, String, u64)> {
        if !self.diff_edit.active || self.diff_edit.working_copy {
            return None;
        }
        let subtitle = self
            .diff_edit
            .change_id
            .as_deref()
            .unwrap_or_default()
            .chars()
            .take(12)
            .collect::<String>();
        Some((
            subtitle,
            self.diff_edit.message.clone(),
            self.diff_edit.session,
        ))
    }

    pub(crate) fn apply_diff_edit_description(&mut self, session: u64, text: String) {
        if self.diff_edit.active && self.diff_edit.session == session {
            self.diff_edit.message = text;
        }
    }

    pub fn start_diff_edit_apply(
        &mut self,
        destination: DiffEditDestination,
        cx: &mut Context<Self>,
    ) {
        if !self.diff_edit.select_all_pending.is_empty() {
            self.show_toast(SELECTION_STILL_LOADING_MESSAGE, cx);
            return;
        }
        if self.diff_edit_selection_summary().0 == 0 {
            self.show_toast(EMPTY_SELECTION_MESSAGE, cx);
            return;
        }
        if destination == DiffEditDestination::RemoveFromSource
            && !self.diff_edit_inverse_files_ready(cx)
        {
            self.show_toast(FILES_STILL_LOADING_MESSAGE, cx);
            return;
        }
        let message = self.diff_edit.message.clone();
        self.apply_diff_edit(destination, message, cx);
    }

    fn diff_edit_inverse_files_ready(&self, cx: &Context<Self>) -> bool {
        self.vm.read(cx).files.as_ref().is_some_and(|hunks| {
            hunks
                .iter()
                .filter(|hunk| hunk_supports_diff_edit(hunk))
                .all(|hunk| {
                    self.diff_edit.loaded_files.contains_key(&hunk.path)
                        || self.diff_edit.known_unsupported.contains(&hunk.path)
                })
        })
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
}

fn file_selection(
    hunk: &jayjay_core::DiffHunk,
    loaded: &super::state::DiffEditLoadedFile,
    lines: &BTreeSet<u32>,
) -> Option<DiffEditFileSelection> {
    (!lines.is_empty()).then(|| DiffEditFileSelection {
        path: hunk.path.clone(),
        old_path: hunk.old_path.clone(),
        old_content: (hunk.hunk_type != HunkType::Added).then(|| loaded.old_content.to_string()),
        new_content: (hunk.hunk_type != HunkType::Removed).then(|| loaded.new_content.to_string()),
        hunk_type: hunk.hunk_type,
        line_ranges: diff_edit_ranges(lines.iter().copied().collect()),
    })
}
