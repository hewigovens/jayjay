use gpui::{ClipboardItem, Context, EntityInputHandler, SharedString, Window};

use super::super::TextArea;
use super::super::action::{
    Backspace, Copy, Cut, Delete, DeletePreviousWord, DeleteToLineEnd, DeleteToLineStart, Newline,
    Paste, Redo, Undo,
};
use crate::ui::input::{HistoryStep, TextSnapshot};

impl TextArea {
    pub(crate) fn newline(&mut self, _: &Newline, window: &mut Window, cx: &mut Context<Self>) {
        if !self.is_editable() {
            return;
        }
        if self.multiline {
            self.replace_text_in_range(None, "\n", window, cx);
        } else {
            // A single-line field has no newline to insert, so let its owner handle Enter.
            cx.propagate();
        }
    }

    pub(crate) fn backspace(&mut self, _: &Backspace, window: &mut Window, cx: &mut Context<Self>) {
        if !self.is_editable() {
            return;
        }
        if self.selection.is_empty() && !self.select_previous_boundary(cx) {
            window.play_system_bell();
            return;
        }
        self.replace_text_in_range(None, "", window, cx);
    }

    pub(crate) fn delete(&mut self, _: &Delete, window: &mut Window, cx: &mut Context<Self>) {
        if !self.is_editable() {
            return;
        }
        if self.selection.is_empty() && !self.select_next_boundary(cx) {
            window.play_system_bell();
            return;
        }
        self.replace_text_in_range(None, "", window, cx);
    }

    pub(crate) fn delete_previous_word(
        &mut self,
        _: &DeletePreviousWord,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.is_editable() {
            return;
        }
        if self.selection.is_empty() && !self.select_previous_word(cx) {
            window.play_system_bell();
            return;
        }
        self.replace_text_in_range(None, "", window, cx);
    }

    pub(crate) fn delete_to_line_start(
        &mut self,
        _: &DeleteToLineStart,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.is_editable() {
            return;
        }
        if self.selection.is_empty() && !self.select_to_line_start(cx) {
            window.play_system_bell();
            return;
        }
        self.replace_text_in_range(None, "", window, cx);
    }

    pub(crate) fn delete_to_line_end(
        &mut self,
        _: &DeleteToLineEnd,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.is_editable() {
            return;
        }
        if self.selection.is_empty() && !self.select_to_line_end(cx) {
            window.play_system_bell();
            return;
        }
        self.replace_text_in_range(None, "", window, cx);
    }

    pub(crate) fn paste(&mut self, _: &Paste, window: &mut Window, cx: &mut Context<Self>) {
        if !self.is_editable() {
            return;
        }
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            self.replace_text_in_range(None, &text, window, cx);
        }
    }

    pub(crate) fn copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        if !self.selection.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[self.selection.range().clone()].to_string(),
            ));
        }
    }

    pub(crate) fn cut(&mut self, _: &Cut, window: &mut Window, cx: &mut Context<Self>) {
        if !self.is_editable() {
            return;
        }
        if !self.selection.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[self.selection.range().clone()].to_string(),
            ));
            self.replace_text_in_range(None, "", window, cx);
        }
    }

    pub(crate) fn undo(&mut self, _: &Undo, _: &mut Window, cx: &mut Context<Self>) {
        self.step_history(HistoryStep::Undo, cx);
    }

    pub(crate) fn redo(&mut self, _: &Redo, _: &mut Window, cx: &mut Context<Self>) {
        self.step_history(HistoryStep::Redo, cx);
    }

    fn step_history(&mut self, step: HistoryStep, cx: &mut Context<Self>) {
        if !self.is_editable() {
            // Read-only text has nothing of its own to undo, so the keys step jj operations.
            cx.propagate();
            return;
        }
        let Some(snapshot) = self.history.step(step, self.snapshot()) else {
            return;
        };
        self.content = snapshot.text;
        self.selection = snapshot.selection;
        self.marked_range = None;
        self.content_changed(cx);
        self.show_caret(cx);
    }

    pub(crate) fn snapshot(&self) -> TextSnapshot<SharedString> {
        TextSnapshot {
            text: self.content.clone(),
            selection: self.selection.clone(),
        }
    }

    fn select_previous_boundary(&mut self, cx: &mut Context<Self>) -> bool {
        let prev = self.previous_boundary(self.cursor_offset());
        if self.cursor_offset() == prev {
            return false;
        }
        self.select_to(prev, cx);
        true
    }

    fn select_next_boundary(&mut self, cx: &mut Context<Self>) -> bool {
        let next = self.next_boundary(self.cursor_offset());
        if self.cursor_offset() == next {
            return false;
        }
        self.select_to(next, cx);
        true
    }

    fn select_previous_word(&mut self, cx: &mut Context<Self>) -> bool {
        let cursor = self.cursor_offset();
        let prev = self.previous_word_boundary(cursor);
        if cursor == prev {
            return false;
        }
        self.select_to(prev, cx);
        true
    }

    fn select_to_line_start(&mut self, cx: &mut Context<Self>) -> bool {
        let cursor = self.cursor_offset();
        let start = self.line_range_at(cursor).start;
        if cursor == start {
            return false;
        }
        self.select_to(start, cx);
        true
    }

    fn select_to_line_end(&mut self, cx: &mut Context<Self>) -> bool {
        let cursor = self.cursor_offset();
        let end = self.line_range_at(cursor).end;
        let target = if cursor < end {
            end
        } else {
            self.next_boundary(cursor)
        };
        if cursor == target {
            return false;
        }
        self.select_to(target, cx);
        true
    }
}
