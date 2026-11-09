use gpui::{App, Context, FocusHandle, Window};

use super::super::{RepoModal, RepoWindow};

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
        } else if self.modal.is_some() {
            self.dismiss_modal(cx);
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
    pub(crate) fn modal_overlay_open(&self) -> bool {
        self.context_menu.is_some() || self.rating_prompt_open()
    }

    /// Find takes text only while nothing else does: no overlay that handles keys first and no focused text field.
    pub(crate) fn find_ime_focus(&self, window: &Window, cx: &App) -> Option<FocusHandle> {
        let covered = self.modal_overlay_open()
            || self.revset_popup.is_some()
            || matches!(
                self.modal,
                Some(RepoModal::StackedPr(_) | RepoModal::Converge(_))
            )
            || self.bookmark_picker.is_some()
            || self.repo_switcher.is_some()
            || self.is_text_input_focused(window, cx);
        if covered { None } else { window.focused(cx) }
    }

    pub(crate) fn is_text_input_focused(&self, window: &Window, cx: &App) -> bool {
        self.focused_text_input(window, cx).is_some()
            || self.file_filter_focus.is_focused(window)
            || self.editor_input_focused(window, cx)
            || self
                .text_modal()
                .is_some_and(|modal| modal.prompt.is_focused(window, cx))
            || self
                .pr_import()
                .is_some_and(|state| state.is_focused(window, cx))
    }
}
