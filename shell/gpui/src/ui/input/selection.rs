use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone)]
pub struct TextSelection {
    range: Range<usize>,
    reversed: bool,
}

impl Default for TextSelection {
    fn default() -> Self {
        Self::at(0)
    }
}

impl TextSelection {
    pub fn at(offset: usize) -> Self {
        Self {
            range: offset..offset,
            reversed: false,
        }
    }

    pub fn from_range(range: Range<usize>, reversed: bool, text_len: usize) -> Self {
        let start = range.start.min(text_len);
        let end = range.end.min(text_len);
        let (range, reversed) = if end < start {
            (end..start, !reversed)
        } else {
            (start..end, reversed)
        };
        Self { range, reversed }
    }

    pub fn range(&self) -> &Range<usize> {
        &self.range
    }

    pub fn range_owned(&self) -> Range<usize> {
        self.range.clone()
    }

    pub fn is_empty(&self) -> bool {
        self.range.is_empty()
    }

    pub fn is_reversed(&self) -> bool {
        self.reversed
    }

    pub fn cursor_offset(&self) -> usize {
        if self.reversed {
            self.range.start
        } else {
            self.range.end
        }
    }

    pub fn move_to(&mut self, offset: usize, text_len: usize) {
        let offset = offset.min(text_len);
        self.range = offset..offset;
        self.reversed = false;
    }

    pub fn select_to(&mut self, offset: usize, text_len: usize) {
        let offset = offset.min(text_len);
        if self.reversed {
            self.range.start = offset;
        } else {
            self.range.end = offset;
        }
        if self.range.end < self.range.start {
            self.reversed = !self.reversed;
            self.range = self.range.end..self.range.start;
        }
    }

    pub fn select_all(&mut self, text_len: usize) {
        self.range = 0..text_len;
        self.reversed = false;
    }
}

pub fn previous_boundary(text: &str, offset: usize) -> usize {
    text.grapheme_indices(true)
        .rev()
        .find_map(|(idx, _)| (idx < offset).then_some(idx))
        .unwrap_or(0)
}

pub fn next_boundary(text: &str, offset: usize) -> usize {
    text.grapheme_indices(true)
        .find_map(|(idx, _)| (idx > offset).then_some(idx))
        .unwrap_or(text.len())
}

pub fn previous_word_boundary(text: &str, offset: usize) -> usize {
    let mut cursor = offset.min(text.len());
    while let Some((idx, ch)) = previous_char(text, cursor) {
        cursor = idx;
        if is_word_char(ch) {
            break;
        }
    }
    while let Some((idx, ch)) = previous_char(text, cursor) {
        if !is_word_char(ch) {
            break;
        }
        cursor = idx;
    }
    cursor
}

pub fn next_word_boundary(text: &str, offset: usize) -> usize {
    let mut cursor = offset.min(text.len());
    while let Some((_, ch)) = char_at(text, cursor) {
        if is_word_char(ch) {
            break;
        }
        cursor += ch.len_utf8();
    }
    while let Some((_, ch)) = char_at(text, cursor) {
        if !is_word_char(ch) {
            break;
        }
        cursor += ch.len_utf8();
    }
    cursor
}

pub fn line_ranges(text: &str) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut start = 0;
    for (ix, ch) in text.char_indices() {
        if ch == '\n' {
            ranges.push(start..ix);
            start = ix + ch.len_utf8();
        }
    }
    ranges.push(start..text.len());
    ranges
}

pub fn line_range_at(text: &str, offset: usize) -> Range<usize> {
    line_ranges(text)
        .into_iter()
        .find(|range| range.start <= offset && offset <= range.end)
        .unwrap_or(text.len()..text.len())
}

pub fn sanitize_single_line(text: &str) -> String {
    text.replace(['\n', '\r'], " ")
}

fn previous_char(text: &str, offset: usize) -> Option<(usize, char)> {
    text[..offset.min(text.len())].char_indices().next_back()
}

fn char_at(text: &str, offset: usize) -> Option<(usize, char)> {
    text[offset.min(text.len())..]
        .char_indices()
        .next()
        .map(|(idx, ch)| (idx + offset.min(text.len()), ch))
}

fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}
