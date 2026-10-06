use gpui::{App, Context, FocusHandle, Window};

use super::super::RepoWindow;

impl RepoWindow {
    pub(super) fn dismiss_overlay(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let has_runtime_error = {
            let vm = self.vm.read(cx);
            vm.repo.is_some() && vm.error.is_some()
        };
        if has_runtime_error {
            self.vm.update(cx, |vm, cx| {
                vm.clear_error();
                cx.notify();
            });
        } else if self.rating_prompt {
            self.close_rating_prompt(cx);
        } else if self.stacked_pr.is_some() {
            self.close_stacked_pr(cx);
        } else if self.pending_rebase.is_some() {
            self.cancel_drag_rebase(cx);
        } else if self.confirmation.is_some() {
            self.cancel_confirmation(cx);
        } else if self.pr_import.is_some() {
            self.cancel_pr_import(cx);
        } else if self.text_modal.is_some() {
            self.close_text_modal(cx);
        } else if self.dismiss_editor_overlay(cx) {
            return true;
        } else if self.context_menu.is_some() {
            self.close_context_menu(cx);
        } else if self.revset_popup.is_some() {
            self.close_revset_popup(cx);
        } else if self.bookmark_picker.is_some() {
            self.close_bookmark_picker(cx);
        } else if self.repo_switcher.is_some() {
            self.close_repo_switcher(cx);
        } else if self.app_menu_open() {
            self.close_app_menu(cx);
        } else if self.find.query.is_some() {
            self.close_find(cx);
        } else if self.file_column.filter.is_some() {
            self.dismiss_file_filter(window, cx);
        } else if self.revset_completions.take().is_some() {
            cx.notify();
        } else if self.revset_editor.is_some() {
            self.close_revset_editor(cx);
        } else if self.diff_edit_active() {
            self.exit_diff_edit(cx);
        } else if let Some(selected) = {
            let vm = self.vm.read(cx);
            vm.multi_selection_primary_index()
        } {
            self.select_change(selected, cx);
        } else {
            return false;
        }
        true
    }

    /// A context menu or the rating prompt takes keys before any text field, so platform text must not land in one behind it.
    pub(in crate::repo::window) fn modal_overlay_open(&self) -> bool {
        self.context_menu.is_some() || self.rating_prompt
    }

    /// Find takes text only while nothing else does: no overlay that handles keys first and no focused text field.
    pub(in crate::repo::window) fn find_ime_focus(
        &self,
        window: &Window,
        cx: &App,
    ) -> Option<FocusHandle> {
        let covered = self.modal_overlay_open()
            || self.revset_popup.is_some()
            || self.stacked_pr.is_some()
            || self.bookmark_picker.is_some()
            || self.repo_switcher.is_some()
            || self.is_text_input_focused(window, cx);
        if covered { None } else { window.focused(cx) }
    }

    pub(in crate::repo::window) fn is_text_input_focused(&self, window: &Window, cx: &App) -> bool {
        self.focused_text_input(window, cx).is_some()
            || self.file_filter_focus.is_focused(window)
            || self.editor_input_focused(window, cx)
            || self
                .text_modal
                .as_ref()
                .is_some_and(|modal| modal.prompt.is_focused(window, cx))
            || self
                .pr_import
                .as_ref()
                .is_some_and(|state| state.is_focused(window, cx))
    }
}
