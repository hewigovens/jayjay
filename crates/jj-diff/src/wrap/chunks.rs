use crate::types::{DiffLine, DiffSpan};

/// One bucket of a wrapped line: which char range it covers and the spans that fall in it.
#[derive(Clone, Default)]
pub(super) struct SpanChunk {
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) spans: Vec<DiffSpan>,
}

pub(super) fn line_char_len(line: &DiffLine) -> usize {
    spans_char_len(&line.spans)
}

pub(super) fn spans_char_len(spans: &[DiffSpan]) -> usize {
    spans.iter().map(|span| span.text.chars().count()).sum()
}

/// Bucket `spans` into `cols`-wide chunks in one pass — O(L) total.
pub(super) fn split_spans_into_chunks(
    spans: &[DiffSpan],
    cols: usize,
    len: usize,
) -> Vec<Vec<DiffSpan>> {
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

/// Like `split_spans_into_chunks` but packages each chunk with its `[start, end)` range.
pub(super) fn side_chunks(spans: &[DiffSpan], cols: usize) -> Vec<SpanChunk> {
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
