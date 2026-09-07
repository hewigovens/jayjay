use crate::types::DiffSpanStyle;

use super::super::chunks::{side_chunks, spans_char_len, split_spans_into_chunks};
use super::fixtures::span;

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
fn split_never_breaks_a_grapheme_cluster() {
    // "é" as base 'e' + combining acute is one cluster; a tight budget must not
    // split between the base char and its combining mark.
    let combining = "e\u{0301}f"; // é + f, four bytes, three scalars, two clusters
    let spans = vec![span(combining, DiffSpanStyle::Context)];
    let len = spans_char_len(&spans);
    assert_eq!(len, 2, "combining mark adds no display width");
    let chunks = split_spans_into_chunks(&spans, 1, len);
    let texts: Vec<&str> = chunks.iter().map(|c| c[0].text.as_str()).collect();
    assert_eq!(texts, vec!["e\u{0301}", "f"]);
}

#[test]
fn wide_glyphs_take_two_cells_and_move_whole_to_the_next_chunk_at_the_edge() {
    let spans = vec![span("你好世", DiffSpanStyle::Context)];
    assert_eq!(spans_char_len(&spans), 6);

    let chunks = side_chunks(&spans, 3);
    let texts: Vec<String> = chunks
        .iter()
        .map(|c| c.spans.iter().map(|s| s.text.as_str()).collect())
        .collect();
    assert_eq!(texts, vec!["你", "好", "世"]);
    assert_eq!(
        chunks.iter().map(|c| (c.start, c.end)).collect::<Vec<_>>(),
        vec![(0, 2), (2, 4), (4, 6)]
    );
}
