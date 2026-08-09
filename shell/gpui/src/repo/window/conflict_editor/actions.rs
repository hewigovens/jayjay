use gpui::Context;
use jayjay_core::{MergeHunkSource, merge_result_use_source};

use crate::ui::merge_nav::next_unresolved_hunk_index;

use super::RepoWindow;

impl RepoWindow {
    pub fn set_conflict_editor_result(&mut self, content: String, cx: &mut Context<Self>) {
        if let Some(result) = self.conflict_editor.result.as_ref() {
            result.update(cx, |result, cx| result.set_text(content, cx));
            self.conflict_editor.selected_source = None;
            cx.notify();
        }
    }

    pub(crate) fn use_conflict_source(&mut self, source: MergeHunkSource, cx: &mut Context<Self>) {
        let Some(data) = self.conflict_editor.data.as_ref() else {
            return;
        };
        if !data.is_text {
            return;
        }
        let content = match source {
            MergeHunkSource::Left => data.left.clone(),
            MergeHunkSource::Base => data.base.clone(),
            MergeHunkSource::Right => data.right.clone(),
        };
        if let Some(result) = self.conflict_editor.result.as_ref() {
            result.update(cx, |result, cx| result.set_text(content.clone(), cx));
            self.conflict_editor.selected_source = Some((source, content));
            cx.notify();
        }
    }

    pub(crate) fn toggle_conflict_base(&mut self, cx: &mut Context<Self>) {
        self.conflict_editor.show_base = !self.conflict_editor.show_base;
        cx.notify();
    }

    pub(crate) fn set_conflict_result_raw(&mut self, show_raw: bool, cx: &mut Context<Self>) {
        self.conflict_editor.show_raw = show_raw;
        self.conflict_editor.focus_pending = show_raw;
        cx.notify();
    }

    pub(crate) fn select_conflict_hunk(&mut self, index: usize, cx: &mut Context<Self>) {
        self.conflict_editor.selected_hunk = index;
        cx.notify();
    }

    pub(crate) fn use_selected_conflict_hunk(
        &mut self,
        source: MergeHunkSource,
        cx: &mut Context<Self>,
    ) {
        self.use_conflict_hunk(self.conflict_editor.selected_hunk, source, cx);
    }

    pub(crate) fn use_conflict_hunk(
        &mut self,
        index: usize,
        source: MergeHunkSource,
        cx: &mut Context<Self>,
    ) {
        let (Some(data), Some(result)) = (
            self.conflict_editor.data.as_ref(),
            self.conflict_editor.result.as_ref(),
        ) else {
            return;
        };
        let Some(hunk) = data.hunks.get(index) else {
            return;
        };
        let text = result.read(cx).text();
        // Keyboard actions can arrive with a resolved card selected; acting on it would hit some other occurrence.
        if !jayjay_core::merge_hunk_is_unresolved(&text, hunk) {
            return;
        }
        match merge_result_use_source(&text, hunk, source) {
            Ok(content) => {
                result.update(cx, |result, cx| result.set_text(content, cx));
                self.conflict_editor.selected_source = None;
                self.move_conflict_hunk(1, cx);
            }
            Err(error) => self.show_toast(error.to_string(), cx),
        }
    }

    pub(crate) fn move_conflict_hunk(&mut self, delta: isize, cx: &mut Context<Self>) {
        let (Some(data), Some(result)) = (
            self.conflict_editor.data.as_ref(),
            self.conflict_editor.result.as_ref(),
        ) else {
            return;
        };
        let result = result.read(cx).text();
        if let Some(index) = next_unresolved_hunk_index(
            &data.hunks,
            &result,
            self.conflict_editor.selected_hunk,
            delta,
        ) {
            self.conflict_editor.selected_hunk = index;
            cx.notify();
        }
    }

    pub(crate) fn save_conflict_editor(&mut self, cx: &mut Context<Self>) {
        let (Some(data), Some(result)) = (
            self.conflict_editor.data.clone(),
            self.conflict_editor.result.as_ref(),
        ) else {
            return;
        };
        if !data.is_text || self.conflict_editor.saving {
            return;
        }
        let content = result.read(cx).text();
        let rev = self.conflict_editor.rev.clone();
        let session = self.conflict_editor.session;
        self.conflict_editor.saving = true;
        let task = self.vm.update(cx, |vm, cx| {
            vm.apply_conflict_editor(rev, data, content, cx)
        });
        cx.spawn(async move |this, cx| {
            let saved = task.await.is_ok();
            let _ = this.update(cx, move |view, cx| {
                if view.conflict_editor.session != session {
                    return;
                }
                if saved {
                    view.exit_conflict_editor(cx);
                } else {
                    view.conflict_editor.saving = false;
                    cx.notify();
                }
            });
        })
        .detach();
        cx.notify();
    }
}
