use std::ops::Range;

use gpui::{Bounds, Pixels};
use jayjay_core::diff::side_by_side::SideBySideRow;
use jayjay_core::diff::{DiffLine, DiffSpan, DiffSpanStyle};

const DEFAULT_WRAP_COLS: usize = 120;
const MIN_WRAP_COLS: usize = 24;

#[derive(Clone)]
pub struct WrappedDiffLine {
    pub line_ix: usize,
    pub line_len: usize,
    pub col_start: usize,
    pub col_end: usize,
    pub line: DiffLine,
}

#[derive(Clone)]
pub struct WrappedSbsRow {
    pub row_ix: usize,
    pub old_line_len: usize,
    pub old_col_start: usize,
    pub old_col_end: usize,
    pub new_line_len: usize,
    pub new_col_start: usize,
    pub new_col_end: usize,
    pub row: SideBySideRow,
}

pub fn wrap_cols_from_bounds(bounds: Option<Bounds<Pixels>>, advance: Pixels) -> usize {
    let Some(bounds) = bounds else {
        return DEFAULT_WRAP_COLS;
    };
    wrap_cols_for_width(f32::from(bounds.size.width), f32::from(advance))
}

pub fn wrap_cols_for_width(width: f32, advance: f32) -> usize {
    if width <= 0. || advance <= 0. {
        return DEFAULT_WRAP_COLS;
    }
    ((width / advance).floor() as usize)
        .saturating_sub(1)
        .max(MIN_WRAP_COLS)
}

pub fn line_char_len(line: &DiffLine) -> usize {
    spans_char_len(&line.spans)
}

pub fn spans_char_len(spans: &[DiffSpan]) -> usize {
    spans.iter().map(|span| span.text.chars().count()).sum()
}

pub fn wrap_diff_lines(lines: &[DiffLine], cols: usize) -> Vec<WrappedDiffLine> {
    let cols = cols.max(1);
    let mut wrapped = Vec::new();
    for (line_ix, line) in lines.iter().enumerate() {
        let line_len = line_char_len(line);
        if line.style == DiffSpanStyle::Separator || line_len <= cols {
            wrapped.push(WrappedDiffLine {
                line_ix,
                line_len,
                col_start: 0,
                col_end: line_len,
                line: line.clone(),
            });
            continue;
        }

        for (visual_ix, start) in (0..line_len).step_by(cols).enumerate() {
            let end = (start + cols).min(line_len);
            wrapped.push(WrappedDiffLine {
                line_ix,
                line_len,
                col_start: start,
                col_end: end,
                line: DiffLine {
                    old_line_no: (visual_ix == 0).then_some(line.old_line_no).flatten(),
                    new_line_no: (visual_ix == 0).then_some(line.new_line_no).flatten(),
                    style: line.style,
                    spans: split_spans(&line.spans, start, end),
                    no_eof_newline: line.no_eof_newline && end == line_len,
                },
            });
        }
    }
    wrapped
}

pub fn wrap_sbs_rows(
    rows: &[SideBySideRow],
    old_cols: usize,
    new_cols: usize,
) -> Vec<WrappedSbsRow> {
    let old_cols = old_cols.max(1);
    let new_cols = new_cols.max(1);
    let mut wrapped = Vec::new();

    for (row_ix, row) in rows.iter().enumerate() {
        let old_len = spans_char_len(&row.old_spans);
        let new_len = spans_char_len(&row.new_spans);
        if row.old_style == DiffSpanStyle::Separator {
            wrapped.push(WrappedSbsRow {
                row_ix,
                old_line_len: old_len,
                old_col_start: 0,
                old_col_end: old_len,
                new_line_len: new_len,
                new_col_start: 0,
                new_col_end: new_len,
                row: row.clone(),
            });
            continue;
        }

        let old_chunks = side_chunks(&row.old_spans, old_cols);
        let new_chunks = side_chunks(&row.new_spans, new_cols);
        let visual_count = old_chunks.len().max(new_chunks.len()).max(1);
        for visual_ix in 0..visual_count {
            let old = old_chunks.get(visual_ix).cloned().unwrap_or_default();
            let new = new_chunks.get(visual_ix).cloned().unwrap_or_default();
            wrapped.push(WrappedSbsRow {
                row_ix,
                old_line_len: old_len,
                old_col_start: old.start,
                old_col_end: old.end,
                new_line_len: new_len,
                new_col_start: new.start,
                new_col_end: new.end,
                row: SideBySideRow {
                    old_line_no: if visual_ix == 0 {
                        row.old_line_no.clone()
                    } else {
                        String::new()
                    },
                    old_spans: old.spans,
                    old_style: row.old_style,
                    new_line_no: if visual_ix == 0 {
                        row.new_line_no.clone()
                    } else {
                        String::new()
                    },
                    new_spans: new.spans,
                    new_style: row.new_style,
                },
            });
        }
    }

    wrapped
}

pub fn visual_index_for_line(wrapped: &[WrappedDiffLine], line_ix: usize) -> usize {
    wrapped
        .iter()
        .position(|row| row.line_ix == line_ix)
        .unwrap_or(line_ix)
}

/// For each `DiffLine` index, the index of the `SideBySideRow` that consumes it.
/// Mirrors the pairing logic in `jj_diff::side_by_side::build_side_by_side_rows`:
/// consecutive Removed/Added lines pair up and produce `max(removed, added)` rows.
pub fn sbs_line_to_row(lines: &[DiffLine]) -> Vec<usize> {
    let mut map = vec![0usize; lines.len()];
    let mut i = 0;
    let mut row_ix = 0;
    while i < lines.len() {
        match lines[i].style {
            DiffSpanStyle::Context | DiffSpanStyle::Separator => {
                map[i] = row_ix;
                row_ix += 1;
                i += 1;
            }
            // Mirror the wildcard arm in `build_side_by_side_rows`: skip without producing a row.
            DiffSpanStyle::Unchanged => {
                i += 1;
            }
            DiffSpanStyle::Removed => {
                let rem_start = i;
                while i < lines.len() && lines[i].style == DiffSpanStyle::Removed {
                    i += 1;
                }
                let rem_end = i;
                let add_start = i;
                while i < lines.len() && lines[i].style == DiffSpanStyle::Added {
                    i += 1;
                }
                let add_end = i;
                let rem_count = rem_end - rem_start;
                let add_count = add_end - add_start;
                let pair_count = rem_count.max(add_count);
                for j in 0..rem_count {
                    map[rem_start + j] = row_ix + j;
                }
                for j in 0..add_count {
                    map[add_start + j] = row_ix + j;
                }
                row_ix += pair_count;
            }
            DiffSpanStyle::Added => {
                map[i] = row_ix;
                row_ix += 1;
                i += 1;
            }
        }
    }
    map
}

/// First wrapped visual position where `row_ix` appears.
pub fn visual_index_for_sbs_row(wrapped: &[WrappedSbsRow], row_ix: usize) -> usize {
    wrapped
        .iter()
        .position(|row| row.row_ix == row_ix)
        .unwrap_or(row_ix)
}

pub fn selection_cols_in_fragment(
    cols: Range<usize>,
    fragment_start: usize,
    fragment_end: usize,
) -> Option<Range<usize>> {
    // `.then_some(v)` is eager — `v` is evaluated even when the predicate is false.
    // Use `.then(|| v)` so the subtractions don't run on out-of-range fragments
    // (e.g. a selection on an earlier wrap segment vs. a later continuation fragment).
    if cols.start == cols.end {
        return (cols.start >= fragment_start && cols.start <= fragment_end)
            .then(|| (cols.start - fragment_start)..(cols.start - fragment_start));
    }

    let start = cols.start.max(fragment_start);
    let end = cols.end.min(fragment_end);
    (start < end).then(|| (start - fragment_start)..(end - fragment_start))
}

#[derive(Clone, Default)]
struct SpanChunk {
    start: usize,
    end: usize,
    spans: Vec<DiffSpan>,
}

fn side_chunks(spans: &[DiffSpan], cols: usize) -> Vec<SpanChunk> {
    let len = spans_char_len(spans);
    if len == 0 {
        return vec![SpanChunk::default()];
    }
    (0..len)
        .step_by(cols)
        .map(|start| {
            let end = (start + cols).min(len);
            SpanChunk {
                start,
                end,
                spans: split_spans(spans, start, end),
            }
        })
        .collect()
}

fn split_spans(spans: &[DiffSpan], start: usize, end: usize) -> Vec<DiffSpan> {
    let mut out = Vec::new();
    let mut cursor = 0usize;
    for span in spans {
        let span_len = span.text.chars().count();
        let span_start = cursor;
        let span_end = span_start + span_len;
        cursor = span_end;

        let overlap_start = start.max(span_start);
        let overlap_end = end.min(span_end);
        if overlap_start >= overlap_end {
            continue;
        }

        out.push(DiffSpan {
            text: span
                .text
                .chars()
                .skip(overlap_start - span_start)
                .take(overlap_end - overlap_start)
                .collect(),
            style: span.style,
            token: span.token,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use jayjay_core::diff::syntax::SyntaxToken;

    use super::*;

    #[test]
    fn wraps_unified_line_with_blank_continuation_numbers() {
        let line = diff_line("abcdefghij", Some(12), Some(14), DiffSpanStyle::Added);
        let wrapped = wrap_diff_lines(&[line], 4);

        assert_eq!(wrapped.len(), 3);
        assert_eq!(text(&wrapped[0].line.spans), "abcd");
        assert_eq!(text(&wrapped[1].line.spans), "efgh");
        assert_eq!(text(&wrapped[2].line.spans), "ij");
        assert_eq!(wrapped[0].line.new_line_no, Some(14));
        assert_eq!(wrapped[1].line.new_line_no, None);
        assert_eq!(wrapped[2].col_start, 8);
        assert_eq!(wrapped[2].col_end, 10);
    }

    #[test]
    fn split_preserves_span_styles() {
        let spans = vec![
            span("abc", DiffSpanStyle::Unchanged),
            span("def", DiffSpanStyle::Added),
        ];

        let split = split_spans(&spans, 2, 5);

        assert_eq!(split.len(), 2);
        assert_eq!(split[0].text, "c");
        assert_eq!(split[0].style, DiffSpanStyle::Unchanged);
        assert_eq!(split[1].text, "de");
        assert_eq!(split[1].style, DiffSpanStyle::Added);
    }

    #[test]
    fn wraps_side_by_side_to_tallest_side() {
        let row = SideBySideRow {
            old_line_no: "10".to_owned(),
            old_spans: vec![span("abcdefgh", DiffSpanStyle::Removed)],
            old_style: DiffSpanStyle::Removed,
            new_line_no: "10".to_owned(),
            new_spans: vec![span("wxyz", DiffSpanStyle::Added)],
            new_style: DiffSpanStyle::Added,
        };

        let wrapped = wrap_sbs_rows(&[row], 3, 3);

        assert_eq!(wrapped.len(), 3);
        assert_eq!(text(&wrapped[0].row.old_spans), "abc");
        assert_eq!(text(&wrapped[1].row.old_spans), "def");
        assert_eq!(text(&wrapped[2].row.old_spans), "gh");
        assert_eq!(text(&wrapped[0].row.new_spans), "wxy");
        assert_eq!(text(&wrapped[1].row.new_spans), "z");
        assert_eq!(text(&wrapped[2].row.new_spans), "");
        assert_eq!(wrapped[1].row.old_line_no, "");
    }

    #[test]
    fn selection_is_mapped_relative_to_visual_fragment() {
        assert_eq!(selection_cols_in_fragment(2..8, 4, 10), Some(0..4));
        assert_eq!(selection_cols_in_fragment(2..4, 4, 10), None);
        assert_eq!(selection_cols_in_fragment(6..6, 4, 10), Some(2..2));
    }

    #[test]
    fn selection_before_continuation_fragment_returns_none_without_overflow() {
        // Regression: selection ends before this fragment starts.
        // Previously `.then_some(v)` evaluated `(end - fragment_start)` eagerly
        // and panicked with subtraction overflow.
        assert_eq!(selection_cols_in_fragment(5..10, 80, 150), None);
        assert_eq!(selection_cols_in_fragment(5..5, 80, 150), None);
        assert_eq!(selection_cols_in_fragment(200..210, 80, 150), None);
    }

    #[test]
    fn visual_index_for_line_walks_wrapped_rows() {
        // Three lines: short, long (3-way wrap), short. Wrap width 3.
        let lines = vec![
            diff_line("foo", Some(1), Some(1), DiffSpanStyle::Context),
            diff_line("abcdefgh", Some(2), Some(2), DiffSpanStyle::Added),
            diff_line("bar", Some(3), Some(3), DiffSpanStyle::Context),
        ];
        let wrapped = wrap_diff_lines(&lines, 3);
        // line 0 → visual 0; line 1 wraps to visuals 1,2,3; line 2 → visual 4.
        assert_eq!(visual_index_for_line(&wrapped, 0), 0);
        assert_eq!(visual_index_for_line(&wrapped, 1), 1);
        assert_eq!(visual_index_for_line(&wrapped, 2), 4);
    }

    #[test]
    fn sbs_line_to_row_pairs_removed_and_added() {
        // Context, Removed×3, Added×2, Context  → rows: 0, 1,2,3 (paired), 4 (context)
        // Lines: 0=Ctx, 1..3=Removed, 4..5=Added, 6=Ctx.
        let lines = vec![
            diff_line("ctx", Some(1), Some(1), DiffSpanStyle::Context),
            diff_line("r1", Some(2), None, DiffSpanStyle::Removed),
            diff_line("r2", Some(3), None, DiffSpanStyle::Removed),
            diff_line("r3", Some(4), None, DiffSpanStyle::Removed),
            diff_line("a1", None, Some(2), DiffSpanStyle::Added),
            diff_line("a2", None, Some(3), DiffSpanStyle::Added),
            diff_line("ctx2", Some(5), Some(4), DiffSpanStyle::Context),
        ];
        let map = sbs_line_to_row(&lines);
        // Context row gets its own row index.
        assert_eq!(map[0], 0);
        // 3 removed pair with 2 added — max(3,2)=3 rows starting at row 1.
        assert_eq!(map[1], 1);
        assert_eq!(map[2], 2);
        assert_eq!(map[3], 3);
        assert_eq!(map[4], 1); // first added joins first removed row
        assert_eq!(map[5], 2);
        // Trailing context at row 4 (after the 3-pair block).
        assert_eq!(map[6], 4);
    }

    #[test]
    fn sbs_line_to_row_handles_added_only_and_separator() {
        let lines = vec![
            diff_line("sep", None, None, DiffSpanStyle::Separator),
            diff_line("a1", None, Some(1), DiffSpanStyle::Added),
            diff_line("a2", None, Some(2), DiffSpanStyle::Added),
        ];
        let map = sbs_line_to_row(&lines);
        // Separator gets its own row; bare Added (no preceding Removed) gets one row each.
        assert_eq!(map, vec![0, 1, 2]);
    }

    #[test]
    fn visual_index_for_sbs_row_returns_first_wrapped_position() {
        // Two SBS rows; the first wraps to 3 visual rows, the second to 1.
        let rows = vec![
            SideBySideRow {
                old_line_no: "1".into(),
                old_spans: vec![span("abcdefgh", DiffSpanStyle::Removed)],
                old_style: DiffSpanStyle::Removed,
                new_line_no: "1".into(),
                new_spans: vec![span("wxyz", DiffSpanStyle::Added)],
                new_style: DiffSpanStyle::Added,
            },
            SideBySideRow {
                old_line_no: "2".into(),
                old_spans: vec![span("ok", DiffSpanStyle::Context)],
                old_style: DiffSpanStyle::Context,
                new_line_no: "2".into(),
                new_spans: vec![span("ok", DiffSpanStyle::Context)],
                new_style: DiffSpanStyle::Context,
            },
        ];
        let wrapped = wrap_sbs_rows(&rows, 3, 3);
        // First row wraps to 3 visuals (0,1,2); second row is visual 3.
        assert_eq!(visual_index_for_sbs_row(&wrapped, 0), 0);
        assert_eq!(visual_index_for_sbs_row(&wrapped, 1), 3);
        // Missing row_ix falls back to the raw index — defensive default.
        assert_eq!(visual_index_for_sbs_row(&wrapped, 99), 99);
    }

    fn diff_line(
        text: &str,
        old_line_no: Option<u32>,
        new_line_no: Option<u32>,
        style: DiffSpanStyle,
    ) -> DiffLine {
        DiffLine {
            old_line_no,
            new_line_no,
            style,
            spans: vec![span(text, style)],
            no_eof_newline: false,
        }
    }

    fn span(text: &str, style: DiffSpanStyle) -> DiffSpan {
        DiffSpan {
            text: text.to_owned(),
            style,
            token: SyntaxToken::Plain,
        }
    }

    fn text(spans: &[DiffSpan]) -> String {
        spans.iter().map(|span| span.text.as_str()).collect()
    }
}
