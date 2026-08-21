use gpui::Context;
use jayjay_core::{MergeHunkSource, merge_result_use_source};

use crate::ui::merge_nav::next_unresolved_hunk_index;

use super::view::{ExternalToolState, ExternalToolWindow};

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
            result.update(cx, |result, cx| result.set_text(content.clone(), cx));
            session.selected_source = Some((path, content));
        } else {
            session.selected_source = Some((path, result.read(cx).text()));
        }
        cx.notify();
    }

    pub(super) fn toggle_merge_base(&mut self, cx: &mut Context<Self>) {
        self.show_merge_base = !self.show_merge_base;
        cx.notify();
    }

    pub(super) fn set_merge_result_raw(&mut self, raw: bool, cx: &mut Context<Self>) {
        self.show_merge_raw = raw;
        cx.notify();
    }

    pub(super) fn select_merge_hunk(&mut self, index: usize, cx: &mut Context<Self>) {
        self.selected_merge_hunk = index;
        cx.notify();
    }

    pub(super) fn use_selected_merge_hunk(
        &mut self,
        source: MergeHunkSource,
        cx: &mut Context<Self>,
    ) {
        self.use_merge_hunk(self.selected_merge_hunk, source, cx);
    }

    pub(super) fn use_merge_hunk(
        &mut self,
        index: usize,
        source: MergeHunkSource,
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
        if !jayjay_core::merge_hunk_is_unresolved(&text, hunk) {
            return;
        }
        match merge_result_use_source(&text, hunk, source) {
            Ok(content) => {
                result.update(cx, |result, cx| result.set_text(content, cx));
                session.selected_source = None;
                self.move_merge_hunk(1, cx);
            }
            Err(error) => {
                self.error_message = Some(error.to_string());
                cx.notify();
            }
        }
    }

    pub(super) fn move_merge_hunk(&mut self, delta: isize, cx: &mut Context<Self>) {
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
            self.selected_merge_hunk = index;
            cx.notify();
        }
    }
}
