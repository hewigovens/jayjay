use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::types::{DiffLine, DiffSpan};

/// One bucket of a wrapped line: its display-cell range and the spans in it.
#[derive(Default)]
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
    if is_single_cell_ascii(text) {
        return text.len();
    }
    text.graphemes(true).map(grapheme_cells).sum()
}

/// ASCII graphemes are single bytes of one cell each, except CRLF, which is one cluster.
fn is_single_cell_ascii(text: &str) -> bool {
    text.is_ascii() && !text.contains("\r\n")
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
pub(super) fn side_chunks(spans: &[DiffSpan], cols: usize) -> Vec<SpanChunk> {
    let mut writer = ChunkWriter::new(cols.max(1));
    for span in spans {
        if is_single_cell_ascii(&span.text) {
            let mut rest = span.text.as_str();
            while !rest.is_empty() {
                let room = writer.room();
                if room == 0 {
                    writer.break_chunk(span);
                    continue;
                }
                let (head, tail) = rest.split_at(room.min(rest.len()));
                writer.push(head, head.len());
                rest = tail;
            }
        } else {
            for g in span.text.graphemes(true) {
                let w = grapheme_cells(g);
                if !writer.fits(w) {
                    writer.break_chunk(span);
                }
                writer.push(g, w);
            }
        }
        writer.flush(span);
    }
    writer.finish()
}

/// Fills `current` until it reaches `cols`; `buf` holds the current span's text not yet flushed.
struct ChunkWriter {
    cols: usize,
    chunks: Vec<SpanChunk>,
    current: SpanChunk,
    buf: String,
}

impl ChunkWriter {
    fn new(cols: usize) -> Self {
        Self {
            cols,
            chunks: Vec::new(),
            current: SpanChunk::default(),
            buf: String::new(),
        }
    }

    fn used(&self) -> usize {
        self.current.end - self.current.start
    }

    fn room(&self) -> usize {
        self.cols.saturating_sub(self.used())
    }

    /// An empty chunk takes any glyph, so one wider than `cols` still gets its own row.
    fn fits(&self, cells: usize) -> bool {
        self.used() == 0 || cells <= self.room()
    }

    fn push(&mut self, text: &str, cells: usize) {
        self.buf.push_str(text);
        self.current.end += cells;
    }

    fn break_chunk(&mut self, source: &DiffSpan) {
        self.flush(source);
        let start = self.current.end;
        let next = SpanChunk {
            start,
            end: start,
            spans: Vec::new(),
        };
        self.chunks.push(std::mem::replace(&mut self.current, next));
    }

    /// Append `buf` (if non-empty) as a span carrying `source`'s style to the current chunk.
    fn flush(&mut self, source: &DiffSpan) {
        if self.buf.is_empty() {
            return;
        }
        self.current.spans.push(DiffSpan {
            text: std::mem::take(&mut self.buf),
            style: source.style,
            token: source.token,
        });
    }

    fn finish(mut self) -> Vec<SpanChunk> {
        self.chunks.push(self.current);
        self.chunks
    }
}
