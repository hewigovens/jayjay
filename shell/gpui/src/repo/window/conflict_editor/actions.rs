use gpui::{App, Context, Entity, Window};
use jayjay_core::{MergeEditorHunkExt, MergeHunkSource, MergePane};

use crate::ui::merge_nav::next_unresolved_hunk_index;
use crate::ui::merge_scroll::{MergeSync, MergeSynchronized};
use crate::ui::text_area::TextArea;

use super::RepoWindow;

impl MergeSynchronized for RepoWindow {
    fn merge_sync(&self) -> &MergeSync {
        &self.conflict_editor.sync
    }

    fn merge_sync_mut(&mut self) -> &mut MergeSync {
        &mut self.conflict_editor.sync
    }

    fn merge_source_areas(&self) -> Option<&[Entity<TextArea>; 3]> {
        let state = &self.conflict_editor;
        // A conflict with more than two sides renders no source panes.
        state.data.as_ref().filter(|data| data.side_count == 2)?;
        state.sources.as_ref()
    }

    fn merge_result_area(&self) -> Option<&Entity<TextArea>> {
        self.conflict_editor.result.as_ref()
    }

    fn merge_shows_base(&self) -> bool {
        self.conflict_editor.show_base
    }

    fn merge_session(&self) -> Option<u64> {
        let state = &self.conflict_editor;
        state.active.then_some(state.session)
    }
}

impl RepoWindow {
    pub fn set_conflict_editor_result(&mut self, content: String, cx: &mut Context<Self>) {
        if let Some(result) = self.conflict_editor.result.as_ref() {
            result.update(cx, |result, cx| result.set_text_keeping_scroll(content, cx));
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
            result.update(cx, |result, cx| {
                result.set_text_keeping_scroll(content.clone(), cx)
            });
            self.conflict_editor.selected_source = Some((source, content));
            cx.notify();
        }
    }

    pub(crate) fn toggle_conflict_base(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.conflict_editor.show_base = !self.conflict_editor.show_base;
        self.merge_base_toggled(window, cx);
        cx.notify();
    }

    pub(crate) fn set_conflict_result_raw(
        &mut self,
        show_raw: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.conflict_editor.show_raw = show_raw;
        self.conflict_editor.focus_pending = show_raw;
        let selected = self.conflict_editor.selected_hunk as u32;
        self.merge_raw_changed(show_raw, selected, window, cx);
        cx.notify();
    }

    pub(crate) fn select_conflict_hunk(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.conflict_editor.selected_hunk = index;
        self.merge_hunk_selected(index as u32, window, cx);
        cx.notify();
    }

    pub(crate) fn use_selected_conflict_hunk(
        &mut self,
        source: MergeHunkSource,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.use_conflict_hunk(self.conflict_editor.selected_hunk, source, window, cx);
    }

    pub(crate) fn use_conflict_hunk(
        &mut self,
        index: usize,
        source: MergeHunkSource,
        window: &mut Window,
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
        if !hunk.is_unresolved(&text) {
            return;
        }
        match hunk.use_source(&text, source) {
            Ok(content) => {
                result.update(cx, |result, cx| result.set_text_keeping_scroll(content, cx));
                self.conflict_editor.selected_source = None;
                self.move_conflict_hunk(1, window, cx);
            }
            Err(error) => self.show_toast(crate::app::error_text(error), cx),
        }
    }

    pub(crate) fn move_conflict_hunk(
        &mut self,
        delta: isize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
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
            self.select_conflict_hunk(index, window, cx);
        }
    }

    pub fn conflict_editor_pane_center(&self, pane: MergePane, cx: &App) -> Option<f64> {
        self.merge_pane_center_line(pane, cx)
    }

    pub fn selected_conflict_hunk(&self) -> usize {
        self.conflict_editor.selected_hunk
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
