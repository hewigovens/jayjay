use gpui::{App, Context, point, px};

use crate::repo::window::RepoWindow;

impl RepoWindow {
    pub(crate) fn shows_diff_stack(&self) -> bool {
        self.diff.stack.plan.is_some()
    }

    /// Whether Show All Files would bring the stack back.
    pub(crate) fn shows_file_alone(&self, cx: &App) -> bool {
        self.diff.stack.solo && self.stackable_files(cx).is_some()
    }

    pub(crate) fn show_all_file_diffs(&mut self, cx: &mut Context<Self>) {
        self.diff.stack.solo = false;
        cx.notify();
    }

    pub(crate) fn show_stacked_file_alone(&mut self, hunk_ix: usize, cx: &mut Context<Self>) {
        self.diff.stack.solo = true;
        self.select_file(hunk_ix, cx);
        cx.notify();
    }

    pub(crate) fn retry_stacked_diffs(&mut self, cx: &mut Context<Self>) {
        self.vm.update(cx, |vm, cx| vm.retry_failed_diffs(cx));
    }

    /// A click inside a stacked file makes it the selected one, so what follows acts on it without leaving the stack.
    pub(crate) fn activate_stacked_file(&mut self, hunk_ix: usize, cx: &mut Context<Self>) {
        if self.vm.read(cx).selected_file_ix == Some(hunk_ix) {
            return;
        }
        self.collapse_file_multi_select(hunk_ix, cx);
        self.reset_diff_panel_for_new_file(cx);
        let vm = self.vm.clone();
        vm.update(cx, |vm, cx| vm.select_file(hunk_ix, cx));
    }

    pub(crate) fn scroll_diff_stack_to_row(&self, row: usize, cx: &App) -> bool {
        let Some(top) = self.stack_row_top(row, cx) else {
            return false;
        };
        let scroll = &self.diff.stack.scroll;
        let viewport = f32::from(scroll.bounds().size.height);
        scroll.set_offset(point(scroll.offset().x, px(-(top - viewport / 2.).max(0.))));
        true
    }
}
