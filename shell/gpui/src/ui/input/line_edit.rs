use std::ops::Range;

mod key;

use super::{
    TextSelection, next_boundary, next_word_boundary, previous_boundary, previous_word_boundary,
};
pub use key::LineEditKeyResult;

#[derive(Debug, Default, Clone)]
pub struct LineEdit {
    text: String,
    selection: TextSelection,
}

impl LineEdit {
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        let end = text.len();
        Self {
            text,
            selection: TextSelection::at(end),
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
        self.move_to(self.text.len());
    }

    pub fn cursor_offset(&self) -> usize {
        self.selection.cursor_offset()
    }

    pub fn selection_range(&self) -> Range<usize> {
        self.selection.range_owned()
    }

    pub fn selection_reversed(&self) -> bool {
        self.selection.is_reversed()
    }

    fn selected_text(&self) -> Option<String> {
        (!self.selection.is_empty()).then(|| self.text[self.selection.range().clone()].to_owned())
    }

    fn select_all(&mut self) {
        self.selection.select_all(self.text.len());
    }

    fn move_left(&mut self) {
        if self.selection.is_empty() {
            self.move_to(previous_boundary(&self.text, self.cursor_offset()));
        } else {
            self.move_to(self.selection.range().start);
        }
    }

    fn move_right(&mut self) {
        if self.selection.is_empty() {
            self.move_to(next_boundary(&self.text, self.cursor_offset()));
        } else {
            self.move_to(self.selection.range().end);
        }
    }

    fn move_word_left(&mut self) {
        if self.selection.is_empty() {
            self.move_to(previous_word_boundary(&self.text, self.cursor_offset()));
        } else {
            self.move_to(self.selection.range().start);
        }
    }

    fn move_word_right(&mut self) {
        if self.selection.is_empty() {
            self.move_to(next_word_boundary(&self.text, self.cursor_offset()));
        } else {
            self.move_to(self.selection.range().end);
        }
    }

    fn backspace(&mut self) -> bool {
        if self.selection.is_empty() {
            let prev = previous_boundary(&self.text, self.cursor_offset());
            if self.cursor_offset() == prev {
                return false;
            }
            self.select_to(prev);
        }
        self.replace_selection("");
        true
    }

    fn delete(&mut self) -> bool {
        if self.selection.is_empty() {
            let next = next_boundary(&self.text, self.cursor_offset());
            if self.cursor_offset() == next {
                return false;
            }
            self.select_to(next);
        }
        self.replace_selection("");
        true
    }

    fn delete_previous_word(&mut self) -> bool {
        if self.selection.is_empty() {
            let cursor = self.cursor_offset();
            let prev = previous_word_boundary(&self.text, cursor);
            if cursor == prev {
                return false;
            }
            self.select_to(prev);
        }
        self.replace_selection("");
        true
    }

    fn delete_to_start(&mut self) -> bool {
        if self.selection.is_empty() {
            let cursor = self.cursor_offset();
            if cursor == 0 {
                return false;
            }
            self.select_to(0);
        }
        self.replace_selection("");
        true
    }

    fn move_to(&mut self, offset: usize) {
        self.selection.move_to(offset, self.text.len());
    }

    fn select_to(&mut self, offset: usize) {
        self.selection.select_to(offset, self.text.len());
    }

    fn replace_selection(&mut self, replacement: &str) {
        let range = self.selection.range().clone();
        self.text.replace_range(range.clone(), replacement);
        self.move_to(range.start + replacement.len());
    }
}
