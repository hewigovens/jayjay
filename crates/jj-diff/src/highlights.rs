use crate::syntax::{HighlightSpan, SyntaxToken};

use super::types::{DiffSpan, DiffSpanStyle};

/// Apply pre-computed syntax highlights to a line at a given byte offset.
pub(super) fn apply_highlights(
    line: &str,
    byte_offset: usize,
    highlights: &[HighlightSpan],
    diff_style: DiffSpanStyle,
) -> Vec<DiffSpan> {
    if line.is_empty() {
        return vec![];
    }

    let line_start = byte_offset;
    let line_end = byte_offset + line.len();

    let relevant: Vec<&HighlightSpan> = highlights
        .iter()
        .filter(|s| s.start < line_end && s.end > line_start)
        .collect();

    if relevant.is_empty() {
        return vec![DiffSpan {
            text: line.to_owned(),
            style: diff_style,
            token: SyntaxToken::Plain,
        }];
    }

    let mut spans = Vec::new();
    let mut pos = 0usize;

    for hs in &relevant {
        let span_start = hs.start.saturating_sub(line_start).min(line.len());
        let span_end = (hs.end.saturating_sub(line_start)).min(line.len());

        if span_start > pos {
            spans.push(DiffSpan {
                text: line[pos..span_start].to_owned(),
                style: diff_style,
                token: SyntaxToken::Plain,
            });
        }

        if span_start < span_end {
            spans.push(DiffSpan {
                text: line[span_start..span_end].to_owned(),
                style: diff_style,
                token: hs.token,
            });
            pos = span_end;
        }
    }

    if pos < line.len() {
        spans.push(DiffSpan {
            text: line[pos..].to_owned(),
            style: diff_style,
            token: SyntaxToken::Plain,
        });
    }

    spans
}
