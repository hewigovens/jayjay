use super::chunks::{line_char_len, split_spans_into_chunks};
use super::types::WrappedDiffLine;
use crate::types::{DiffLine, DiffSpanStyle};

/// Wrap unified lines into visual rows. Continuation rows share `line_ix` with
/// `col_start > 0` and carry no line numbers.
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

/// First wrapped visual position for `line_ix`; falls back to `line_ix` if absent.
/// O(log N) via `partition_point` — `wrap_diff_lines` emits non-decreasing `line_ix`.
pub fn visual_index_for_line(wrapped: &[WrappedDiffLine], line_ix: u32) -> u32 {
    let pos = wrapped.partition_point(|row| row.line_ix < line_ix);
    if pos < wrapped.len() && wrapped[pos].line_ix == line_ix {
        pos as u32
    } else {
        line_ix
    }
}
