use crate::side_by_side::{RowSide, SideBySideRow};
use crate::syntax::SyntaxToken;
use crate::types::{DiffLine, DiffSpan, DiffSpanStyle};

use super::chunks::split_spans_into_chunks;
use super::{
    DEFAULT_WRAP_COLS, MIN_WRAP_COLS, sbs_line_to_row, visual_index_for_line,
    visual_index_for_sbs_row, wrap_cols_for_width, wrap_diff_lines, wrap_sbs_rows,
};

#[test]
fn wrap_cols_for_width_clamps_and_defaults() {
    assert_eq!(wrap_cols_for_width(0., 8.), DEFAULT_WRAP_COLS);
    assert_eq!(wrap_cols_for_width(800., 0.), DEFAULT_WRAP_COLS);
    assert_eq!(wrap_cols_for_width(40., 8.), MIN_WRAP_COLS);
    // 800 / 8 = 100 cells, minus 1 for trailing gutter padding.
    assert_eq!(wrap_cols_for_width(800., 8.), 99);
}

#[test]
fn split_spans_into_chunks_preserves_styles_when_spans_align_or_cross_boundaries() {
    // Aligned: span boundary == chunk boundary; each chunk owns one styled segment.
    let aligned = vec![
        span("abc", DiffSpanStyle::Unchanged),
        span("def", DiffSpanStyle::Added),
    ];
    let chunks = split_spans_into_chunks(&aligned, 3, 6);
    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0][0].text, "abc");
    assert_eq!(chunks[0][0].style, DiffSpanStyle::Unchanged);
    assert_eq!(chunks[1][0].text, "def");
    assert_eq!(chunks[1][0].style, DiffSpanStyle::Added);

    // Crossing: one long span split into multiple chunks, style preserved on each.
    let crossing = vec![span("abcdefgh", DiffSpanStyle::Added)];
    let chunks = split_spans_into_chunks(&crossing, 3, 8);
    assert_eq!(chunks.len(), 3);
    let texts: Vec<&str> = chunks.iter().map(|c| c[0].text.as_str()).collect();
    assert_eq!(texts, vec!["abc", "def", "gh"]);
    assert!(chunks.iter().all(|c| c[0].style == DiffSpanStyle::Added));
}

#[test]
fn wrap_diff_lines_emits_continuation_rows_without_line_numbers() {
    let line = diff_line("abcdefghij", Some(12), Some(14), DiffSpanStyle::Added);
    let wrapped = wrap_diff_lines(&[line], 4);

    assert_eq!(wrapped.len(), 3);
    let texts: Vec<String> = wrapped.iter().map(|w| text(&w.line.spans)).collect();
    assert_eq!(texts, vec!["abcd", "efgh", "ij"]);
    assert_eq!(wrapped[0].line.new_line_no, Some(14));
    assert_eq!(wrapped[1].line.new_line_no, None);
    assert_eq!((wrapped[2].col_start, wrapped[2].col_end), (8, 10));
}

#[test]
fn wrap_sbs_rows_pads_to_tallest_side_and_blanks_continuation_line_no() {
    let row = SideBySideRow {
        old: row_side("10", "abcdefgh", DiffSpanStyle::Removed),
        new: row_side("10", "wxyz", DiffSpanStyle::Added),
    };
    let wrapped = wrap_sbs_rows(&[row], 3, 3);

    assert_eq!(wrapped.len(), 3);
    let old_texts: Vec<String> = wrapped.iter().map(|w| text(&w.row.old.spans)).collect();
    let new_texts: Vec<String> = wrapped.iter().map(|w| text(&w.row.new.spans)).collect();
    assert_eq!(old_texts, vec!["abc", "def", "gh"]);
    assert_eq!(new_texts, vec!["wxy", "z", ""]);
    assert_eq!(wrapped[0].row.old.line_no, "10");
    assert_eq!(wrapped[1].row.old.line_no, "");
}

#[test]
fn visual_index_finds_first_wrapped_position_for_unified_and_sbs() {
    // Unified: a long Added line in the middle inflates the visual count.
    let lines = vec![
        diff_line("foo", Some(1), Some(1), DiffSpanStyle::Context),
        diff_line("abcdefgh", Some(2), Some(2), DiffSpanStyle::Added),
        diff_line("bar", Some(3), Some(3), DiffSpanStyle::Context),
    ];
    let unified = wrap_diff_lines(&lines, 3);
    assert_eq!(visual_index_for_line(&unified, 0), 0);
    assert_eq!(visual_index_for_line(&unified, 1), 1);
    assert_eq!(visual_index_for_line(&unified, 2), 4);

    // SBS: same shape across the row pairing.
    let rows = vec![
        SideBySideRow {
            old: row_side("1", "abcdefgh", DiffSpanStyle::Removed),
            new: row_side("1", "wxyz", DiffSpanStyle::Added),
        },
        SideBySideRow {
            old: row_side("2", "ok", DiffSpanStyle::Context),
            new: row_side("2", "ok", DiffSpanStyle::Context),
        },
    ];
    let sbs = wrap_sbs_rows(&rows, 3, 3);
    assert_eq!(visual_index_for_sbs_row(&sbs, 0), 0);
    assert_eq!(visual_index_for_sbs_row(&sbs, 1), 3);
    // Out-of-range falls back to the requested ix.
    assert_eq!(visual_index_for_sbs_row(&sbs, 99), 99);
}

#[test]
fn sbs_line_to_row_maps_all_styles() {
    // Context bracketing a 3-removed / 2-added block, plus separator and trailing context.
    let lines = vec![
        diff_line("ctx", Some(1), Some(1), DiffSpanStyle::Context),
        diff_line("r1", Some(2), None, DiffSpanStyle::Removed),
        diff_line("r2", Some(3), None, DiffSpanStyle::Removed),
        diff_line("r3", Some(4), None, DiffSpanStyle::Removed),
        diff_line("a1", None, Some(2), DiffSpanStyle::Added),
        diff_line("a2", None, Some(3), DiffSpanStyle::Added),
        diff_line("ctx2", Some(5), Some(4), DiffSpanStyle::Context),
        diff_line("sep", None, None, DiffSpanStyle::Separator),
        diff_line("a3", None, Some(5), DiffSpanStyle::Added),
    ];
    let map = sbs_line_to_row(&lines);
    // ctx → row 0; r1/r2/r3 → rows 1/2/3; a1/a2 pair into rows 1/2; ctx2 → row 4;
    // separator → row 5; trailing a3 → row 6.
    assert_eq!(map, vec![0, 1, 2, 3, 1, 2, 4, 5, 6]);
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

fn row_side(line_no: &str, text: &str, style: DiffSpanStyle) -> RowSide {
    RowSide {
        line_no: line_no.to_owned(),
        spans: vec![span(text, style)],
        style,
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
