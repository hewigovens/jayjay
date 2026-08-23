use gpui::{Context, Focusable, Window};

use super::super::RepoWindow;

impl RepoWindow {
    pub(super) fn dismiss_overlay(&mut self, cx: &mut Context<Self>) -> bool {
        let has_runtime_error = {
            let vm = self.vm.read(cx);
            vm.repo.is_some() && vm.error.is_some()
        };
        if has_runtime_error {
            self.vm.update(cx, |vm, cx| {
                vm.clear_error();
                cx.notify();
            });
        } else if self.stacked_pr.is_some() {
            self.close_stacked_pr(cx);
        } else if self.pending_rebase.is_some() {
            self.cancel_drag_rebase(cx);
        } else if self.confirmation.is_some() {
            self.cancel_confirmation(cx);
        } else if self.text_modal.is_some() {
            self.close_text_modal(cx);
        } else if self.dismiss_editor_overlay(cx) {
            return true;
        } else if self.context_menu.is_some() {
            self.close_context_menu(cx);
        } else if self.bookmark_picker.is_some() {
            self.close_bookmark_picker(cx);
        } else if self.repo_switcher.is_some() {
            self.close_repo_switcher(cx);
        } else if self.app_menu_open() {
            self.close_app_menu(cx);
        } else if self.find.query.is_some() {
            self.close_find(cx);
        } else if self.file_column.filter.is_some() {
            self.close_file_filter(cx);
        } else if self.revset_filter.is_some() {
            self.close_revset_filter(cx);
        } else if self.diff_edit_active() {
            self.exit_diff_edit(cx);
        } else {
            return false;
        }
        true
    }

    pub(super) fn is_text_input_focused(&self, window: &Window, cx: &gpui::App) -> bool {
        if self
            .summary_input
            .read(cx)
            .focus_handle(cx)
            .is_focused(window)
            || self
                .description_input
                .read(cx)
                .focus_handle(cx)
                .is_focused(window)
        {
            return true;
        }
        if self.revset_filter_focus.is_focused(window)
            || self.file_filter_focus.is_focused(window)
            || self.editor_input_focused(window, cx)
        {
            return true;
        }
        self.text_modal
            .as_ref()
            .is_some_and(|modal| modal.input.read(cx).focus_handle(cx).is_focused(window))
    }
}
