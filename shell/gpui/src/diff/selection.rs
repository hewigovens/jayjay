use std::ops::{Range, RangeInclusive};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SbsSide {
    Unified,
    Old,
    New,
}

// `side` gates highlight rendering; old/new clicks are mutually exclusive.
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

    // None when line is outside selection; line_len clamps drags past EOL.
    pub fn col_range_for(&self, line_ix: usize, line_len: usize) -> Option<Range<usize>> {
        if !self.line_range().contains(&line_ix) {
            return None;
        }
        let (lo_line, lo_col, hi_line, hi_col) =
            if (self.anchor_line, self.anchor_col) <= (self.focus_line, self.focus_col) {
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
        if start >= end {
            Some(start..start)
        } else {
            Some(start..end)
        }
    }
}

// Word chars are alphanumeric + `_`; a non-word click returns an empty range.
// `col` and the result are display cells (wide CJK/emoji glyphs span two), matching
// the wrap geometry and pixel-to-cell mouse mapping.
pub fn word_at(text: &str, col: usize) -> Range<usize> {
    let cells = grapheme_cells(text);
    if cells.is_empty() {
        return 0..0;
    }
    // Clamp the click to a grapheme and find its cell span.
    let ix = cells
        .iter()
        .position(|c| col < c.cell_end)
        .unwrap_or(cells.len() - 1);
    let is_word = |g: &str| g.chars().all(|c| c.is_alphanumeric() || c == '_') && !g.is_empty();
    if !is_word(cells[ix].text) {
        return cells[ix].cell_start..cells[ix].cell_start;
    }
    let mut start = ix;
    while start > 0 && is_word(cells[start - 1].text) {
        start -= 1;
    }
    let mut end = ix + 1;
    while end < cells.len() && is_word(cells[end].text) {
        end += 1;
    }
    cells[start].cell_start..cells[end - 1].cell_end
}

struct GraphemeCell<'a> {
    text: &'a str,
    cell_start: usize,
    cell_end: usize,
}

fn grapheme_cells(text: &str) -> Vec<GraphemeCell<'_>> {
    use unicode_segmentation::UnicodeSegmentation;
    use unicode_width::UnicodeWidthStr;
    let mut cell = 0usize;
    text.graphemes(true)
        .map(|g| {
            let w = UnicodeWidthStr::width(g).max(1);
            let start = cell;
            cell += w;
            GraphemeCell {
                text: g,
                cell_start: start,
                cell_end: cell,
            }
        })
        .collect()
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

    #[test]
    fn word_at_uses_display_cells_for_wide_glyphs() {
        // "你好 abc": CJK at cells 0..4, space at 4, "abc" at 5..8. A click at cell 6
        // must land on "abc", not drift by the accumulated wide-glyph width.
        let text = "你好 abc";
        assert_eq!(word_at(text, 6), 5..8);
        // Clicking either cell of a wide glyph selects the whole CJK run.
        assert_eq!(word_at(text, 1), 0..4);
    }
}
