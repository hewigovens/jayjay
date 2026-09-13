use gpui::Context;
use jayjay_core::DiffEditDestination;

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
    pub fn diff_edit_has_selection(&self) -> bool {
        self.diff_edit.session.has_selection()
    }

    pub(super) fn diff_edit_selection_text(&self) -> String {
        self.diff_edit.session.summary_text()
    }

    pub(super) fn diff_edit_should_deselect(&self) -> bool {
        self.diff_edit.session.should_deselect()
    }

    pub fn diff_edit_snapshot(&self) -> DiffEditSnapshot {
        let working_copy = self.diff_edit.working_copy;
        let summary = self.diff_edit.session.summary();
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
            selected_files: summary.files as usize,
            selected_lines: summary.lines as usize,
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
            self.diff_edit.epoch,
        ))
    }

    pub(crate) fn apply_diff_edit_description(&mut self, epoch: u64, text: String) {
        if self.diff_edit.active && self.diff_edit.epoch == epoch {
            self.diff_edit.message = text;
        }
    }

    pub fn start_diff_edit_apply(
        &mut self,
        destination: DiffEditDestination,
        cx: &mut Context<Self>,
    ) {
        if self.diff_edit.session.is_selecting_all() {
            self.show_toast(SELECTION_STILL_LOADING_MESSAGE, cx);
            return;
        }
        if !self.diff_edit_has_selection() {
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
                    self.diff_edit.session.is_loaded(&hunk.path)
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
        let paths: Vec<String> = vm
            .files
            .as_ref()?
            .iter()
            .filter(|hunk| hunk_supports_diff_edit(hunk))
            .map(|hunk| hunk.path.clone())
            .collect();
        let selections = self.diff_edit.session.selections(&paths, destination);
        if selections.is_empty() {
            return None;
        }
        Some(DiffEditApplyRequest {
            rev: change.selection_revision().to_owned(),
            destination,
            selections,
            message,
            ignore_whitespace: vm.ignore_whitespace,
            restore_path: vm.selected_hunk()?.path.clone(),
        })
    }
}
