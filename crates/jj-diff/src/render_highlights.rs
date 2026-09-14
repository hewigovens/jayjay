use std::sync::Arc;

mod cache;

use cache::HighlightCache;

// Shared across rendering and expansion workers, with bounded retention when a file is never expanded.
static HIGHLIGHTS: HighlightCache = HighlightCache::new(64, 64 * 1024 * 1024);

use crate::syntax::{self, HighlightSpan, SyntaxToken};

use super::highlights::apply_highlights;
use super::types::{DiffLine, DiffSide, DiffSpan, DiffSpanStyle, LineIndex};
use super::word_diff::word_diff_paired_line;

pub(super) struct HighlightInputs<'a> {
    pub old: &'a str,
    pub new: &'a str,
    pub old_line_index: &'a LineIndex,
    pub new_line_index: &'a LineIndex,
    pub language: &'a str,
    pub skip_highlight: bool,
}

pub(super) fn plain_spans(text: &str, style: DiffSpanStyle) -> Vec<DiffSpan> {
    if text.is_empty() {
        return vec![];
    }
    vec![DiffSpan {
        text: text.to_owned(),
        style,
        token: SyntaxToken::Plain,
    }]
}

/// Both sides are highlighted from their whole source: a fragment of visible lines loses the lexer state of the hidden ones, so a string or comment opened above a collapsed region mis-tokenizes everything after it.
pub(super) fn apply_rendered_highlights(lines: &mut [DiffLine], inputs: HighlightInputs<'_>) {
    if inputs.skip_highlight {
        return;
    }
    let old_highlights = SideHighlights::full(inputs.old, inputs.old_line_index, inputs.language);
    let new_highlights = SideHighlights::full(inputs.new, inputs.new_line_index, inputs.language);
    apply_side_highlights(lines, &old_highlights, &new_highlights);
}

pub(crate) fn apply_side_highlights(
    lines: &mut [DiffLine],
    old_highlights: &SideHighlights,
    new_highlights: &SideHighlights,
) {
    let mut index = 0usize;
    while index < lines.len() {
        if lines[index].style == DiffSpanStyle::Removed
            && index + 1 < lines.len()
            && lines[index + 1].style == DiffSpanStyle::Added
            && let (Some(old_ln), Some(new_ln)) =
                (lines[index].old_line_no, lines[index + 1].new_line_no)
        {
            let old_text = lines[index].text();
            let new_text = lines[index + 1].text();
            let (removed_spans, added_spans) = word_diff_paired_line(
                &old_text,
                old_highlights.offset(old_ln),
                old_highlights.spans(),
                &new_text,
                new_highlights.offset(new_ln),
                new_highlights.spans(),
            );
            lines[index].spans = removed_spans;
            lines[index + 1].spans = added_spans;
            index += 2;
            continue;
        }

        if let Some((side, line_no, style)) = highlight_side_for_line(&lines[index]) {
            let text = lines[index].text();
            let highlights = match side {
                DiffSide::Old => &old_highlights,
                DiffSide::New => &new_highlights,
            };
            lines[index].spans =
                apply_highlights(&text, highlights.offset(line_no), highlights.spans(), style);
        }
        index += 1;
    }
}

fn highlight_side_for_line(line: &DiffLine) -> Option<(DiffSide, u32, DiffSpanStyle)> {
    match line.style {
        DiffSpanStyle::Context => line
            .new_line_no
            .map(|line_no| (DiffSide::New, line_no, DiffSpanStyle::Context))
            .or_else(|| {
                line.old_line_no
                    .map(|line_no| (DiffSide::Old, line_no, DiffSpanStyle::Context))
            }),
        DiffSpanStyle::Removed => line
            .old_line_no
            .map(|line_no| (DiffSide::Old, line_no, DiffSpanStyle::Unchanged)),
        DiffSpanStyle::Added => line
            .new_line_no
            .map(|line_no| (DiffSide::New, line_no, DiffSpanStyle::Unchanged)),
        DiffSpanStyle::Unchanged | DiffSpanStyle::Separator => None,
    }
}

#[derive(Default)]
pub(crate) struct SideHighlights {
    spans: Vec<HighlightSpan>,
    offsets: Vec<usize>,
}

impl SideHighlights {
    /// Offsets cover every source line, so lines revealed after construction still resolve.
    pub(crate) fn full(source: &str, line_index: &LineIndex, language: &str) -> Arc<Self> {
        if language == "plaintext" || source.is_empty() {
            return Arc::new(Self::default());
        }
        HIGHLIGHTS.get_or_insert_with(source, language, || {
            Self::uncached(source, line_index, language)
        })
    }

    fn uncached(source: &str, line_index: &LineIndex, language: &str) -> Self {
        let spans = syntax::highlight(source, language);
        // Without a grammar there are no spans to index, so avoid retaining a per-line table.
        let offsets = if spans.is_empty() {
            Vec::new()
        } else {
            (1..)
                .map_while(|line_no| line_index.get(source, line_no).map(|(offset, _)| offset))
                .collect()
        };
        Self { spans, offsets }
    }

    fn retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.spans.capacity() * std::mem::size_of::<HighlightSpan>()
            + self.offsets.capacity() * std::mem::size_of::<usize>()
    }

    fn spans(&self) -> &[HighlightSpan] {
        &self.spans
    }

    fn offset(&self, line_no: u32) -> usize {
        line_no
            .checked_sub(1)
            .and_then(|index| self.offsets.get(index as usize))
            .copied()
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn files_without_a_grammar_build_no_offset_table() {
        let source = "fn main() {}\n";
        let line_index = LineIndex::from_text(source);
        for (language, mapped) in [("plaintext", false), ("rust", true)] {
            let highlights = SideHighlights::full(source, &line_index, language);
            assert_eq!(!highlights.offsets.is_empty(), mapped, "{language}");
        }
    }
}
