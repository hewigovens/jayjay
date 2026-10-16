use std::ops::Range;

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
    pub tooltip: Option<String>,
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
    /// Set by move detection on a changed row whose code moved unchanged to another hunk.
    pub moved: Option<MovedRow>,
}

/// The other copy of a moved block starts at `counterpart_line` on the opposite side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MovedRow {
    pub counterpart_line: u32,
    pub starts_block: bool,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextExpansionError {
    UnknownRegion { region_id: u32 },
    InvalidLineCount,
    InvalidRegion { region_id: u32 },
    MissingSourceLine { line_no: u32 },
    // Constructed only by the FFI bridge when the expansion session lock is unusable.
    SessionUnavailable,
}

impl std::fmt::Display for ContextExpansionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownRegion { region_id } => {
                write!(f, "context region {region_id} is no longer available")
            }
            Self::InvalidLineCount => f.write_str("context expansion line count must be positive"),
            Self::InvalidRegion { region_id } => {
                write!(f, "context region {region_id} has invalid source bounds")
            }
            Self::MissingSourceLine { line_no } => {
                write!(f, "context source line {line_no} is unavailable")
            }
            Self::SessionUnavailable => f.write_str("context expansion session is unavailable"),
        }
    }
}

impl std::error::Error for ContextExpansionError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineSpan {
    pub start: u32,
    pub count: u32,
}

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

    /// Zero-based line containing byte `offset`.
    pub(super) fn line_at(&self, offset: usize) -> usize {
        self.ranges
            .partition_point(|&(start, _)| start <= offset)
            .saturating_sub(1)
    }

    /// Lines whose non-blank content, less a trailing `,` or `;` separator, lies inside `span`, when at least one does.
    pub(super) fn lines_within(&self, text: &str, span: Range<usize>) -> Option<LineSpan> {
        let first = self.ranges.partition_point(|&(_, end)| end <= span.start);
        let mut covered = (first..self.ranges.len())
            .take_while(|&line| self.ranges[line].0 < span.end)
            .filter(|&line| {
                let (start, end) = self.ranges[line];
                let content = text[start..end].trim_start();
                let item = content.trim_end_matches([',', ';', ' ', '\t']);
                let item_start = end - content.len();
                !item.is_empty() && item_start >= span.start && item_start + item.len() <= span.end
            });
        let first = covered.next()?;
        let last = covered.last().unwrap_or(first);
        Some(LineSpan {
            start: first as u32 + 1,
            count: (last - first) as u32 + 1,
        })
    }

    pub(super) fn get<'a>(&self, text: &'a str, line_no_1based: u32) -> Option<(usize, &'a str)> {
        let (start, end) = *self.ranges.get(line_no_1based.checked_sub(1)? as usize)?;
        Some((start, &text[start..end]))
    }
}
