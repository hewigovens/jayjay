use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::types::{DiffLine, DiffSpan};

/// One bucket of a wrapped line: its display-cell range and the spans in it.
#[derive(Clone, Default)]
pub(super) struct SpanChunk {
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) spans: Vec<DiffSpan>,
}

/// Display-cell width of a grapheme cluster; width-1 floor keeps control/zero-width
/// clusters addressable by the cell-based selection geometry.
pub(super) fn grapheme_cells(g: &str) -> usize {
    UnicodeWidthStr::width(g).max(1)
}

/// Display-cell width of a string, counting CJK/emoji as two cells.
fn text_cells(text: &str) -> usize {
    text.graphemes(true).map(grapheme_cells).sum()
}

pub(super) fn line_char_len(line: &DiffLine) -> usize {
    spans_char_len(&line.spans)
}

pub(super) fn spans_char_len(spans: &[DiffSpan]) -> usize {
    spans.iter().map(|span| text_cells(&span.text)).sum()
}

/// Bucket `spans` into `cols`-wide (display-cell) chunks in one pass.
/// Never splits inside a grapheme cluster; a wide glyph that would straddle a
/// chunk edge moves wholly into the next chunk.
pub(super) fn split_spans_into_chunks(
    spans: &[DiffSpan],
    cols: usize,
    len: usize,
) -> Vec<Vec<DiffSpan>> {
    if len == 0 {
        return vec![Vec::new()];
    }
    let cols = cols.max(1);
    let mut chunks: Vec<Vec<DiffSpan>> = vec![Vec::new()];
    // Cell position within the current (last) chunk.
    let mut chunk_cells = 0usize;

    for span in spans {
        if span.text.is_empty() {
            continue;
        }
        let mut buf = String::new();
        for g in span.text.graphemes(true) {
            let w = grapheme_cells(g);
            // `chunk_cells > 0` lets a glyph wider than a whole chunk still occupy its own row.
            if chunk_cells + w > cols && chunk_cells > 0 {
                flush(&mut chunks, &mut buf, span);
                chunks.push(Vec::new());
                chunk_cells = 0;
            }
            buf.push_str(g);
            chunk_cells += w;
        }
        flush(&mut chunks, &mut buf, span);
    }

    chunks
}

/// Append `buf` (if non-empty) as a span carrying `source`'s style to the last chunk.
fn flush(chunks: &mut [Vec<DiffSpan>], buf: &mut String, source: &DiffSpan) {
    if buf.is_empty() {
        return;
    }
    if let Some(chunk) = chunks.last_mut() {
        chunk.push(DiffSpan {
            text: std::mem::take(buf),
            style: source.style,
            token: source.token,
        });
    }
}

/// Like `split_spans_into_chunks` but packages each chunk with its `[start, end)` cell range.
pub(super) fn side_chunks(spans: &[DiffSpan], cols: usize) -> Vec<SpanChunk> {
    let len = spans_char_len(spans);
    if len == 0 {
        return vec![SpanChunk::default()];
    }
    let mut start = 0usize;
    split_spans_into_chunks(spans, cols, len)
        .into_iter()
        .map(|spans| {
            let width = spans.iter().map(|s| text_cells(&s.text)).sum::<usize>();
            let chunk = SpanChunk {
                start,
                end: start + width,
                spans,
            };
            start += width;
            chunk
        })
        .collect()
}
