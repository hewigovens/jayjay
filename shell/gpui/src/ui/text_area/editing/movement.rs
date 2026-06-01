use gpui::{Context, Window};

use super::super::TextArea;
use super::super::action::{
    DocumentEnd, DocumentStart, End, Home, Left, Right, SelectAll, SelectDocumentEnd,
    SelectDocumentStart, SelectEnd, SelectHome, SelectLeft, SelectRight, SelectWordLeft,
    SelectWordRight, WordLeft, WordRight,
};

impl TextArea {
    pub(in crate::ui::text_area) fn left(
        &mut self,
        _: &Left,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.selection.is_empty() {
            self.move_to(self.previous_boundary(self.cursor_offset()), cx);
        } else {
            self.move_to(self.selection.range().start, cx);
        }
    }

    pub(in crate::ui::text_area) fn right(
        &mut self,
        _: &Right,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.selection.is_empty() {
            self.move_to(self.next_boundary(self.cursor_offset()), cx);
        } else {
            self.move_to(self.selection.range().end, cx);
        }
    }

    pub(in crate::ui::text_area) fn word_left(
        &mut self,
        _: &WordLeft,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.selection.is_empty() {
            self.move_to(self.previous_word_boundary(self.cursor_offset()), cx);
        } else {
            self.move_to(self.selection.range().start, cx);
        }
    }

    pub(in crate::ui::text_area) fn word_right(
        &mut self,
        _: &WordRight,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.selection.is_empty() {
            self.move_to(self.next_word_boundary(self.cursor_offset()), cx);
        } else {
            self.move_to(self.selection.range().end, cx);
        }
    }

    pub(in crate::ui::text_area) fn select_left(
        &mut self,
        _: &SelectLeft,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.select_to(self.previous_boundary(self.cursor_offset()), cx);
    }

    pub(in crate::ui::text_area) fn select_right(
        &mut self,
        _: &SelectRight,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.select_to(self.next_boundary(self.cursor_offset()), cx);
    }

    pub(in crate::ui::text_area) fn select_word_left(
        &mut self,
        _: &SelectWordLeft,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.select_to(self.previous_word_boundary(self.cursor_offset()), cx);
    }

    pub(in crate::ui::text_area) fn select_word_right(
        &mut self,
        _: &SelectWordRight,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.select_to(self.next_word_boundary(self.cursor_offset()), cx);
    }

    pub(in crate::ui::text_area) fn select_all(
        &mut self,
        _: &SelectAll,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.move_to(0, cx);
        self.select_to(self.content.len(), cx);
    }

    pub(in crate::ui::text_area) fn document_start(
        &mut self,
        _: &DocumentStart,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.move_to(0, cx);
    }

    pub(in crate::ui::text_area) fn document_end(
        &mut self,
        _: &DocumentEnd,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.move_to(self.content.len(), cx);
    }

    pub(in crate::ui::text_area) fn select_document_start(
        &mut self,
        _: &SelectDocumentStart,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.select_to(0, cx);
    }

    pub(in crate::ui::text_area) fn select_document_end(
        &mut self,
        _: &SelectDocumentEnd,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.select_to(self.content.len(), cx);
    }

    pub(in crate::ui::text_area) fn home(
        &mut self,
        _: &Home,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let line = self.line_range_at(self.cursor_offset());
        self.move_to(line.start, cx);
    }

    pub(in crate::ui::text_area) fn select_home(
        &mut self,
        _: &SelectHome,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let line = self.line_range_at(self.cursor_offset());
        self.select_to(line.start, cx);
    }

    pub(in crate::ui::text_area) fn end(
        &mut self,
        _: &End,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let line = self.line_range_at(self.cursor_offset());
        self.move_to(line.end, cx);
    }

    pub(in crate::ui::text_area) fn select_end(
        &mut self,
        _: &SelectEnd,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let line = self.line_range_at(self.cursor_offset());
        self.select_to(line.end, cx);
    }
}
