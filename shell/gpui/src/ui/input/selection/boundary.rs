use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;

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
