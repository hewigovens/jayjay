use crate::types::DiffSpanStyle;

use super::fixtures::span;

#[test]
fn split_spans_into_chunks_preserves_styles_when_spans_align_or_cross_boundaries() {
    // Aligned: span boundary == chunk boundary; each chunk owns one styled segment.
    let aligned = vec![
        span("abc", DiffSpanStyle::Unchanged),
        span("def", DiffSpanStyle::Added),
    ];
    let chunks = super::super::chunks::split_spans_into_chunks(&aligned, 3, 6);
    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0][0].text, "abc");
    assert_eq!(chunks[0][0].style, DiffSpanStyle::Unchanged);
    assert_eq!(chunks[1][0].text, "def");
    assert_eq!(chunks[1][0].style, DiffSpanStyle::Added);

    // Crossing: one long span split into multiple chunks, style preserved on each.
    let crossing = vec![span("abcdefgh", DiffSpanStyle::Added)];
    let chunks = super::super::chunks::split_spans_into_chunks(&crossing, 3, 8);
    assert_eq!(chunks.len(), 3);
    let texts: Vec<&str> = chunks.iter().map(|c| c[0].text.as_str()).collect();
    assert_eq!(texts, vec!["abc", "def", "gh"]);
    assert!(chunks.iter().all(|c| c[0].style == DiffSpanStyle::Added));
}
