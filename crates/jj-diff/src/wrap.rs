//! Visual wrapping for unified and side-by-side diffs.
//!
//! Shells render the diff in monospace; long lines either scroll horizontally or
//! wrap to multiple visual rows. These helpers compute the wrapped layout in
//! pure Rust so both the GPUI and AppKit shells share the same row/column math
//! (and therefore keep the gutter and panes vertically aligned).
//!
//! The functions are pure and do not depend on any UI framework — `f32` widths
//! cross the uniffi boundary into Swift.

use crate::side_by_side::SideBySideRow;
use crate::types::{DiffLine, DiffSpan, DiffSpanStyle};

pub const DEFAULT_WRAP_COLS: u32 = 120;
pub const MIN_WRAP_COLS: u32 = 24;

#[derive(Debug, Clone)]
pub struct WrappedDiffLine {
    pub line_ix: u32,
    pub line_len: u32,
    pub col_start: u32,
    pub col_end: u32,
    pub line: DiffLine,
}

#[derive(Debug, Clone)]
pub struct WrappedSbsRow {
    pub row_ix: u32,
    pub old_line_len: u32,
    pub old_col_start: u32,
    pub old_col_end: u32,
    pub new_line_len: u32,
    pub new_col_start: u32,
    pub new_col_end: u32,
    pub row: SideBySideRow,
}

/// Wrap columns for a pane of `width` pixels with monospace `advance`.
/// Returns `DEFAULT_WRAP_COLS` for non-positive inputs and clamps to `MIN_WRAP_COLS`.
pub fn wrap_cols_for_width(width: f32, advance: f32) -> u32 {
    if width <= 0. || advance <= 0. {
        return DEFAULT_WRAP_COLS;
    }
    ((width / advance).floor() as u32)
        .saturating_sub(1)
        .max(MIN_WRAP_COLS)
}

fn line_char_len(line: &DiffLine) -> usize {
    spans_char_len(&line.spans)
}

fn spans_char_len(spans: &[DiffSpan]) -> usize {
    spans.iter().map(|span| span.text.chars().count()).sum()
}

/// Wrap unified diff lines into per-visual-row records. Each `WrappedDiffLine`
/// is one visual row; continuation rows reuse `line_ix` but have `col_start > 0`.
/// Line numbers appear only on the first visual segment of each logical line.
pub fn wrap_diff_lines(lines: &[DiffLine], cols: u32) -> Vec<WrappedDiffLine> {
    let cols = (cols.max(1)) as usize;
    let mut wrapped = Vec::new();
    for (line_ix, line) in lines.iter().enumerate() {
        let line_len = line_char_len(line);
        if line.style == DiffSpanStyle::Separator || line_len <= cols {
            wrapped.push(WrappedDiffLine {
                line_ix: line_ix as u32,
                line_len: line_len as u32,
                col_start: 0,
                col_end: line_len as u32,
                line: line.clone(),
            });
            continue;
        }

        // Single-pass bucketing across all visual segments at once — O(line_len)
        // total instead of O(line_len^2 / cols) re-scanning per chunk.
        let chunks = split_spans_into_chunks(&line.spans, cols, line_len);
        for (visual_ix, chunk_spans) in chunks.into_iter().enumerate() {
            let start = visual_ix * cols;
            let end = (start + cols).min(line_len);
            wrapped.push(WrappedDiffLine {
                line_ix: line_ix as u32,
                line_len: line_len as u32,
                col_start: start as u32,
                col_end: end as u32,
                line: DiffLine {
                    old_line_no: (visual_ix == 0).then_some(line.old_line_no).flatten(),
                    new_line_no: (visual_ix == 0).then_some(line.new_line_no).flatten(),
                    style: line.style,
                    spans: chunk_spans,
                    no_eof_newline: line.no_eof_newline && end == line_len,
                },
            });
        }
    }
    wrapped
}

/// Wrap side-by-side rows, padding the shorter side with empty continuation
/// rows so both panes (and the gutters) advance in lock-step vertically.
pub fn wrap_sbs_rows(
    rows: &[SideBySideRow],
    old_cols: u32,
    new_cols: u32,
) -> Vec<WrappedSbsRow> {
    let old_cols = (old_cols.max(1)) as usize;
    let new_cols = (new_cols.max(1)) as usize;
    let mut wrapped = Vec::new();

    for (row_ix, row) in rows.iter().enumerate() {
        let old_len = spans_char_len(&row.old_spans);
        let new_len = spans_char_len(&row.new_spans);
        if row.old_style == DiffSpanStyle::Separator {
            wrapped.push(WrappedSbsRow {
                row_ix: row_ix as u32,
                old_line_len: old_len as u32,
                old_col_start: 0,
                old_col_end: old_len as u32,
                new_line_len: new_len as u32,
                new_col_start: 0,
                new_col_end: new_len as u32,
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
                row_ix: row_ix as u32,
                old_line_len: old_len as u32,
                old_col_start: old.start as u32,
                old_col_end: old.end as u32,
                new_line_len: new_len as u32,
                new_col_start: new.start as u32,
                new_col_end: new.end as u32,
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

/// First wrapped visual position for a unified `line_ix`; defaults to `line_ix`
/// when the wrapped slice doesn't contain that line (defensive fallback).
///
/// `wrap_diff_lines` produces rows in monotonically non-decreasing `line_ix`
/// order (lines walked left-to-right; each logical line may emit several
/// continuation rows but all share the same `line_ix`). That ordering lets us
/// binary-search via `partition_point` for O(log N) lookup.
pub fn visual_index_for_line(wrapped: &[WrappedDiffLine], line_ix: u32) -> u32 {
    let pos = wrapped.partition_point(|row| row.line_ix < line_ix);
    if pos < wrapped.len() && wrapped[pos].line_ix == line_ix {
        pos as u32
    } else {
        line_ix
    }
}

/// For each `DiffLine` index, the index of the `SideBySideRow` that consumes it.
/// Mirrors the pairing logic in [`crate::side_by_side::build_side_by_side_rows`]:
/// consecutive Removed/Added lines pair up and produce `max(removed, added)` rows.
pub fn sbs_line_to_row(lines: &[DiffLine]) -> Vec<u32> {
    let mut map = vec![0u32; lines.len()];
    let mut i = 0usize;
    let mut row_ix: u32 = 0;
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
                    map[rem_start + j] = row_ix + j as u32;
                }
                for j in 0..add_count {
                    map[add_start + j] = row_ix + j as u32;
                }
                row_ix += pair_count as u32;
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

/// First wrapped visual position where `row_ix` appears. Falls back to `row_ix`
/// when the wrapped slice doesn't contain it (defensive). Rows are emitted in
/// monotonically non-decreasing `row_ix` order, so `partition_point` gives O(log N).
pub fn visual_index_for_sbs_row(wrapped: &[WrappedSbsRow], row_ix: u32) -> u32 {
    let pos = wrapped.partition_point(|row| row.row_ix < row_ix);
    if pos < wrapped.len() && wrapped[pos].row_ix == row_ix {
        pos as u32
    } else {
        row_ix
    }
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
    let cols = cols.max(1);
    split_spans_into_chunks(spans, cols, len)
        .into_iter()
        .enumerate()
        .map(|(i, spans)| {
            let start = i * cols;
            let end = (start + cols).min(len);
            SpanChunk { start, end, spans }
        })
        .collect()
}

/// Walk `spans` once and bucket characters into `cols`-wide chunks. O(L) total,
/// replacing the previous O(L²/C) repeated `skip()/take()` per chunk. Returns
/// `ceil(len / cols)` buckets in order; each bucket carries its segments
/// alongside the per-span style/token.
fn split_spans_into_chunks(spans: &[DiffSpan], cols: usize, len: usize) -> Vec<Vec<DiffSpan>> {
    if len == 0 {
        return vec![Vec::new()];
    }
    let num_chunks = len.div_ceil(cols);
    let mut chunks: Vec<Vec<DiffSpan>> = (0..num_chunks).map(|_| Vec::new()).collect();

    let mut global_pos = 0usize;
    for span in spans {
        let span_len = span.text.chars().count();
        if span_len == 0 {
            continue;
        }
        let mut chars = span.text.chars();
        let mut remaining = span_len;
        while remaining > 0 {
            let chunk_ix = global_pos / cols;
            let chunk_end = ((chunk_ix + 1) * cols).min(len);
            let take = (chunk_end - global_pos).min(remaining);
            let text: String = (&mut chars).take(take).collect();
            chunks[chunk_ix].push(DiffSpan {
                text,
                style: span.style,
                token: span.token,
            });
            global_pos += take;
            remaining -= take;
        }
    }

    chunks
}

#[cfg(test)]
mod tests {
    use crate::syntax::SyntaxToken;

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
    fn split_into_chunks_preserves_span_styles_across_boundaries() {
        let spans = vec![
            span("abc", DiffSpanStyle::Unchanged),
            span("def", DiffSpanStyle::Added),
        ];
        let chunks = split_spans_into_chunks(&spans, 3, 6);
        // Chunk 0 covers cols 0..3 — "abc" (Unchanged).
        // Chunk 1 covers cols 3..6 — "def" (Added).
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].len(), 1);
        assert_eq!(chunks[0][0].text, "abc");
        assert_eq!(chunks[0][0].style, DiffSpanStyle::Unchanged);
        assert_eq!(chunks[1].len(), 1);
        assert_eq!(chunks[1][0].text, "def");
        assert_eq!(chunks[1][0].style, DiffSpanStyle::Added);
    }

    #[test]
    fn split_into_chunks_splits_a_span_across_chunks() {
        // A single span longer than the chunk width must split into multiple chunks
        // while preserving its style/token on each piece.
        let spans = vec![span("abcdefgh", DiffSpanStyle::Added)];
        let chunks = split_spans_into_chunks(&spans, 3, 8);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0][0].text, "abc");
        assert_eq!(chunks[1][0].text, "def");
        assert_eq!(chunks[2][0].text, "gh");
        for chunk in &chunks {
            assert_eq!(chunk[0].style, DiffSpanStyle::Added);
        }
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
    fn visual_index_for_line_walks_wrapped_rows() {
        let lines = vec![
            diff_line("foo", Some(1), Some(1), DiffSpanStyle::Context),
            diff_line("abcdefgh", Some(2), Some(2), DiffSpanStyle::Added),
            diff_line("bar", Some(3), Some(3), DiffSpanStyle::Context),
        ];
        let wrapped = wrap_diff_lines(&lines, 3);
        assert_eq!(visual_index_for_line(&wrapped, 0), 0);
        assert_eq!(visual_index_for_line(&wrapped, 1), 1);
        assert_eq!(visual_index_for_line(&wrapped, 2), 4);
    }

    #[test]
    fn sbs_line_to_row_pairs_removed_and_added() {
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
        assert_eq!(map[0], 0);
        assert_eq!(map[1], 1);
        assert_eq!(map[2], 2);
        assert_eq!(map[3], 3);
        assert_eq!(map[4], 1);
        assert_eq!(map[5], 2);
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
        assert_eq!(map, vec![0, 1, 2]);
    }

    #[test]
    fn visual_index_for_sbs_row_returns_first_wrapped_position() {
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
        assert_eq!(visual_index_for_sbs_row(&wrapped, 0), 0);
        assert_eq!(visual_index_for_sbs_row(&wrapped, 1), 3);
        assert_eq!(visual_index_for_sbs_row(&wrapped, 99), 99);
    }

    #[test]
    fn wrap_cols_for_width_clamps_and_defaults() {
        // Negative or zero inputs fall back to the default width.
        assert_eq!(wrap_cols_for_width(0., 8.), DEFAULT_WRAP_COLS);
        assert_eq!(wrap_cols_for_width(800., 0.), DEFAULT_WRAP_COLS);
        // Below the minimum: clamp up to MIN_WRAP_COLS.
        assert_eq!(wrap_cols_for_width(40., 8.), MIN_WRAP_COLS);
        // Normal: 800 / 8 = 100 cells, minus 1 for trailing gutter padding.
        assert_eq!(wrap_cols_for_width(800., 8.), 99);
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
