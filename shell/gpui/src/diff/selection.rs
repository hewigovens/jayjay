use std::ops::{Range, RangeInclusive};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SbsSide {
    Unified,
    Old,
    New,
}

// Per-side selection: a click on the new side clears an old-side selection
// and vice-versa — `side` gates which panel renders the highlight.
#[derive(Debug, Clone, Copy)]
pub struct DiffSelection {
    pub anchor_line: usize,
    pub anchor_col: usize,
    pub focus_line: usize,
    pub focus_col: usize,
    pub side: SbsSide,
    pub dragging: bool,
}

impl DiffSelection {
    pub fn start(line: usize, col: usize, side: SbsSide) -> Self {
        Self {
            anchor_line: line,
            anchor_col: col,
            focus_line: line,
            focus_col: col,
            side,
            dragging: true,
        }
    }

    pub fn extend(&mut self, line: usize, col: usize) {
        self.focus_line = line;
        self.focus_col = col;
    }

    pub fn extend_to_word(&mut self, line: usize, word: Range<usize>) {
        self.anchor_line = line;
        self.focus_line = line;
        self.anchor_col = word.start;
        self.focus_col = word.end;
        self.dragging = false;
    }

    pub fn line_range(&self) -> RangeInclusive<usize> {
        let lo = self.anchor_line.min(self.focus_line);
        let hi = self.anchor_line.max(self.focus_line);
        lo..=hi
    }

    pub fn covers(&self, line_ix: usize, side: SbsSide) -> bool {
        self.side == side && self.line_range().contains(&line_ix)
    }

    // Returns None when the line is outside the selection; passing line_len
    // here lets us clamp end-of-line so selection drags past EOL behave.
    pub fn col_range_for(&self, line_ix: usize, line_len: usize) -> Option<Range<usize>> {
        if !self.line_range().contains(&line_ix) {
            return None;
        }
        let (lo_line, lo_col, hi_line, hi_col) = if (self.anchor_line, self.anchor_col)
            <= (self.focus_line, self.focus_col)
        {
            (
                self.anchor_line,
                self.anchor_col,
                self.focus_line,
                self.focus_col,
            )
        } else {
            (
                self.focus_line,
                self.focus_col,
                self.anchor_line,
                self.anchor_col,
            )
        };
        let start = if line_ix == lo_line { lo_col } else { 0 };
        let end = if line_ix == hi_line { hi_col } else { line_len };
        let start = start.min(line_len);
        let end = end.min(line_len);
        if start >= end { Some(start..start) } else { Some(start..end) }
    }
}

// Word chars are alphanumeric + `_`; non-word click returns an empty range.
pub fn word_at(text: &str, col: usize) -> Range<usize> {
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return 0..0;
    }
    let col = col.min(chars.len().saturating_sub(1));
    let is_word = |c: char| c.is_alphanumeric() || c == '_';
    if !is_word(chars[col]) {
        return col..col;
    }
    let mut start = col;
    while start > 0 && is_word(chars[start - 1]) {
        start -= 1;
    }
    let mut end = col + 1;
    while end < chars.len() && is_word(chars[end]) {
        end += 1;
    }
    start..end
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_range_orders_endpoints_low_to_high() {
        let mut sel = DiffSelection::start(5, 0, SbsSide::Unified);
        sel.extend(2, 0);
        assert_eq!(*sel.line_range().start(), 2);
        assert_eq!(*sel.line_range().end(), 5);
    }

    #[test]
    fn covers_respects_side() {
        let sel = DiffSelection::start(2, 0, SbsSide::Old);
        assert!(sel.covers(2, SbsSide::Old));
        assert!(!sel.covers(2, SbsSide::New));
        assert!(!sel.covers(2, SbsSide::Unified));
    }

    #[test]
    fn col_range_full_for_middle_lines_partial_for_edges() {
        let mut sel = DiffSelection::start(2, 5, SbsSide::Unified);
        sel.extend(4, 7);
        assert_eq!(sel.col_range_for(2, 20), Some(5..20));
        assert_eq!(sel.col_range_for(3, 20), Some(0..20));
        assert_eq!(sel.col_range_for(4, 20), Some(0..7));
        assert_eq!(sel.col_range_for(5, 20), None);
    }

    #[test]
    fn col_range_clamps_to_line_length() {
        let mut sel = DiffSelection::start(2, 100, SbsSide::Unified);
        sel.extend(2, 200);
        assert_eq!(sel.col_range_for(2, 10), Some(10..10));
    }

    #[test]
    fn word_at_finds_alphanumeric_run() {
        let r = word_at("foo bar baz", 5);
        assert_eq!(r, 4..7);
    }

    #[test]
    fn word_at_includes_underscore() {
        let r = word_at("foo_bar baz", 4);
        assert_eq!(r, 0..7);
    }

    #[test]
    fn word_at_returns_empty_on_whitespace() {
        let r = word_at("foo bar", 3);
        assert_eq!(r, 3..3);
    }
}
