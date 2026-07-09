use std::collections::HashMap;

use crate::syntax::{self, HighlightSpan, SyntaxToken};

use super::highlights::apply_highlights;
use super::types::{DiffLine, DiffSide, DiffSpan, DiffSpanStyle, LineMap};
use super::word_diff::word_diff_paired_line;

pub(super) struct HighlightInputs<'a> {
    pub old: &'a str,
    pub new: &'a str,
    pub old_line_map: &'a LineMap,
    pub new_line_map: &'a LineMap,
    pub language: &'a str,
    pub skip_highlight: bool,
    pub collapse: bool,
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

pub(super) fn apply_rendered_highlights(lines: &mut [DiffLine], inputs: HighlightInputs<'_>) {
    let old_highlights = SideHighlights::new(
        inputs.old,
        inputs.old_line_map,
        lines,
        DiffSide::Old,
        inputs.language,
        inputs.skip_highlight,
        inputs.collapse,
    );
    let new_highlights = SideHighlights::new(
        inputs.new,
        inputs.new_line_map,
        lines,
        DiffSide::New,
        inputs.language,
        inputs.skip_highlight,
        inputs.collapse,
    );

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

struct SideHighlights {
    spans: Vec<HighlightSpan>,
    offsets: HighlightOffsets,
}

impl SideHighlights {
    fn new(
        source: &str,
        line_map: &LineMap,
        lines: &[DiffLine],
        side: DiffSide,
        language: &str,
        skip_highlight: bool,
        collapse: bool,
    ) -> Self {
        if skip_highlight {
            return Self {
                spans: vec![],
                offsets: HighlightOffsets::Empty,
            };
        }
        if collapse {
            let source = VisibleHighlightSource::new(lines, side, line_map);
            return Self {
                spans: syntax::highlight(&source.text, language),
                offsets: HighlightOffsets::Mapped(source.offsets),
            };
        }
        Self {
            spans: syntax::highlight(source, language),
            offsets: HighlightOffsets::Mapped(original_offsets(lines, side, line_map)),
        }
    }

    fn spans(&self) -> &[HighlightSpan] {
        &self.spans
    }

    fn offset(&self, line_no: u32) -> usize {
        match &self.offsets {
            HighlightOffsets::Empty => 0,
            HighlightOffsets::Mapped(offsets) => offsets.get(&line_no).copied().unwrap_or(0),
        }
    }
}

enum HighlightOffsets {
    Empty,
    Mapped(HashMap<u32, usize>),
}

fn original_offsets(lines: &[DiffLine], side: DiffSide, line_map: &LineMap) -> HashMap<u32, usize> {
    let mut offsets = HashMap::new();
    for line in lines {
        let line_no = match side {
            DiffSide::Old => line.old_line_no,
            DiffSide::New => line.new_line_no,
        };
        let Some(line_no) = line_no else { continue };
        if offsets.contains_key(&line_no) {
            continue;
        }
        if let Some((offset, _text)) = line_map.get(line_no) {
            offsets.insert(line_no, *offset);
        }
    }
    offsets
}

struct VisibleHighlightSource {
    text: String,
    offsets: HashMap<u32, usize>,
}

impl VisibleHighlightSource {
    fn new(lines: &[DiffLine], side: DiffSide, line_map: &LineMap) -> Self {
        let mut text = String::new();
        let mut offsets = HashMap::new();
        for line in lines {
            let line_no = match side {
                DiffSide::Old => line.old_line_no,
                DiffSide::New => line.new_line_no,
            };
            let Some(line_no) = line_no else { continue };
            if offsets.contains_key(&line_no) {
                continue;
            }
            let Some((_original_offset, line_text)) = line_map.get(line_no) else {
                continue;
            };
            offsets.insert(line_no, text.len());
            text.push_str(line_text);
            text.push('\n');
        }
        Self { text, offsets }
    }
}
