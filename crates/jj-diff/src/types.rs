use crate::syntax::SyntaxToken;

pub const CONTEXT_LINES: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffSpanStyle {
    Context,
    Added,
    Removed,
    Unchanged,
    /// Collapsed region placeholder — `spans[0].text` contains "N unmodified lines".
    Separator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictLineKind {
    None,
    Start,
    End,
    Section,
    Content,
    Removed,
    Added,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiffSide {
    Old,
    New,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeGroup {
    pub index: u32,
    /// 1-based display line range in the rendered unified diff.
    pub start_line: u32,
    pub end_line: u32,
    pub anchor_side: DiffSide,
    pub anchor_line: u32,
    pub anchor_excerpt: String,
    pub anchor_context: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DiffSpan {
    pub text: String,
    pub style: DiffSpanStyle,
    pub token: SyntaxToken,
}

#[derive(Debug, Clone)]
pub struct DiffLine {
    pub old_line_no: Option<u32>,
    pub new_line_no: Option<u32>,
    pub style: DiffSpanStyle,
    pub spans: Vec<DiffSpan>,
    pub conflict_kind: ConflictLineKind,
    /// True if this line is the last line on its side and the file has no trailing newline.
    pub no_eof_newline: bool,
    /// Present only for collapsed-context separator lines.
    pub context_region: Option<ContextRegion>,
}

impl DiffLine {
    pub fn text(&self) -> String {
        self.spans.iter().map(|span| span.text.as_str()).collect()
    }

    pub fn is_changed(&self) -> bool {
        matches!(self.style, DiffSpanStyle::Added | DiffSpanStyle::Removed)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContextRegion {
    /// Stable identity derived from the region's first line in the full diff.
    pub id: u32,
    pub old_start_line: u32,
    pub new_start_line: u32,
    pub line_count: u32,
    /// The collapse-time size; reveals shrink `line_count` but never this, so control layouts stay stable.
    pub initial_line_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextExpansion {
    ShowMore { line_count: u32 },
    ShowAll,
}

#[derive(Debug, Clone)]
pub struct ContextExpansionResult {
    pub diff: FileDiff,
    /// Zero-based raw diff-line index where the newly visible lines were inserted.
    pub inserted: LineSpan,
}

pub use jayjay_primitives::{ContextExpansionError, LineSpan};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConflictBlockSection {
    pub label: String,
    pub marker_line: u32,
    pub content_start: u32,
    pub line_end: u32,
    pub kind: ConflictLineKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConflictBlock {
    pub title: String,
    pub line_start: u32,
    pub line_end: u32,
    pub sections: Vec<ConflictBlockSection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffDisplayItem {
    Lines { line_start: u32, line_end: u32 },
    ConflictBlock { block: ConflictBlock },
}

#[derive(Debug, Clone)]
pub struct FileDiff {
    pub path: String,
    pub language: String,
    pub lines: Vec<DiffLine>,
    pub whitespace_only_hidden: bool,
}

#[derive(Debug, Clone)]
pub struct CollapsedDiff {
    pub diff: FileDiff,
    /// Maps 1-based display line number → 1-based full diff line number; separator lines have no entry.
    pub display_to_full: Vec<DisplayLineMapping>,
}

#[derive(Debug, Clone)]
pub struct DisplayLineMapping {
    pub display_line: u32,
    pub full_line: u32,
}

#[derive(Debug)]
pub(super) struct LineIndex {
    /// Trimmed byte ranges indexed by zero-based line number.
    ranges: Vec<(usize, usize)>,
}

impl LineIndex {
    pub(super) fn from_text(text: &str) -> Self {
        let mut ranges = Vec::new();
        let mut offset = 0;
        for line in text.split('\n') {
            let clean = line.strip_suffix('\r').unwrap_or(line).trim_end();
            ranges.push((offset, offset + clean.len()));
            offset += line.len() + 1;
        }
        // split('\n') leaves a phantom empty final entry when text ends with a newline; drop it so the line count matches `.lines()`.
        if text.ends_with('\n') && ranges.last().is_some_and(|(start, end)| start == end) {
            ranges.pop();
        }
        Self { ranges }
    }

    pub(super) fn get<'a>(&self, text: &'a str, line_no_1based: u32) -> Option<(usize, &'a str)> {
        let (start, end) = *self.ranges.get(line_no_1based.checked_sub(1)? as usize)?;
        Some((start, &text[start..end]))
    }
}
