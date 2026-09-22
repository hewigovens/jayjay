use gpui::{App, Context, Entity, Window};
use jayjay_core::{MergeEditorHunkExt, MergeHunkSource, MergePane};

use crate::ui::merge_nav::next_unresolved_hunk_index;
use crate::ui::merge_scroll::{MergeSync, MergeSynchronized};
use crate::ui::text_area::TextArea;

use super::view::{ExternalToolState, ExternalToolWindow};

impl MergeSynchronized for ExternalToolWindow {
    fn merge_sync(&self) -> &MergeSync {
        &self.merge
    }

    fn merge_sync_mut(&mut self) -> &mut MergeSync {
        &mut self.merge
    }

    fn merge_source_areas(&self) -> Option<&[Entity<TextArea>; 3]> {
        match &self.state {
            ExternalToolState::Merge { sources, .. } => Some(sources),
            _ => None,
        }
    }

    fn merge_result_area(&self) -> Option<&Entity<TextArea>> {
        match &self.state {
            ExternalToolState::Merge { result, .. } => Some(result),
            _ => None,
        }
    }

    fn merge_shows_base(&self) -> bool {
        self.show_merge_base
    }

    fn merge_session(&self) -> Option<u64> {
        // The tool window hosts a single merge for the life of the process.
        matches!(self.state, ExternalToolState::Merge { .. }).then_some(0)
    }
}

impl ExternalToolWindow {
    pub(super) fn save(&mut self, cx: &mut Context<Self>) {
        let saved = match &self.state {
            ExternalToolState::Diff(session) if session.editable => session.save_request().run(),
            ExternalToolState::Merge {
                session, result, ..
            } if session.can_save(&result.read(cx).text()) => {
                session.save_request(result.read(cx).text()).run()
            }
            _ => return,
        };
        match saved {
            Ok(()) => (self.exit)(0),
            Err(error) => {
                self.error_message = Some(error.to_string());
                cx.notify();
            }
        }
    }

    pub(super) fn use_merge_source(&mut self, source: MergeHunkSource, cx: &mut Context<Self>) {
        let ExternalToolState::Merge {
            session, result, ..
        } = &mut self.state
        else {
            return;
        };
        let (path, content) = session.source(source);
        let path = path.clone();
        let content = content.to_owned();
        if session.is_text_merge() {
            result.update(cx, |result, cx| {
                result.set_text_keeping_scroll(content.clone(), cx)
            });
            session.selected_source = Some((path, content));
        } else {
            session.selected_source = Some((path, result.read(cx).text()));
        }
        cx.notify();
    }

    pub(super) fn toggle_merge_base(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.show_merge_base = !self.show_merge_base;
        self.merge_base_toggled(window, cx);
        cx.notify();
    }

    pub(super) fn set_merge_result_raw(
        &mut self,
        raw: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.show_merge_raw = raw;
        self.merge_raw_changed(raw, self.selected_merge_hunk as u32, window, cx);
        cx.notify();
    }

    pub(super) fn select_merge_hunk(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.selected_merge_hunk = index;
        self.merge_hunk_selected(index as u32, window, cx);
        cx.notify();
    }

    pub(super) fn use_selected_merge_hunk(
        &mut self,
        source: MergeHunkSource,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.use_merge_hunk(self.selected_merge_hunk, source, window, cx);
    }

    pub(super) fn use_merge_hunk(
        &mut self,
        index: usize,
        source: MergeHunkSource,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let ExternalToolState::Merge {
            session, result, ..
        } = &mut self.state
        else {
            return;
        };
        let Some(hunk) = session.hunks.get(index) else {
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
                session.selected_source = None;
                self.move_merge_hunk(1, window, cx);
            }
            Err(error) => {
                self.error_message = Some(error.to_string());
                cx.notify();
            }
        }
    }

    pub(super) fn move_merge_hunk(
        &mut self,
        delta: isize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let ExternalToolState::Merge {
            session, result, ..
        } = &self.state
        else {
            return;
        };
        let result = result.read(cx).text();
        if let Some(index) =
            next_unresolved_hunk_index(&session.hunks, &result, self.selected_merge_hunk, delta)
        {
            self.select_merge_hunk(index, window, cx);
        }
    }

    pub fn merge_pane_center(&self, pane: MergePane, cx: &App) -> Option<f64> {
        self.merge_pane_center_line(pane, cx)
    }

    pub fn selected_merge_hunk(&self) -> usize {
        self.selected_merge_hunk
    }
}
