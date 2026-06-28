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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}

impl DiffLine {
    pub fn text(&self) -> String {
        self.spans.iter().map(|span| span.text.as_str()).collect()
    }

    pub fn is_changed(&self) -> bool {
        matches!(self.style, DiffSpanStyle::Added | DiffSpanStyle::Removed)
    }
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

pub(super) struct LineMap {
    /// (byte_start, line_content) indexed by 0-based line number
    entries: Vec<(usize, String)>,
}

impl LineMap {
    pub(super) fn from_text(text: &str) -> Self {
        let mut entries = Vec::new();
        let mut offset = 0;
        for line in text.split('\n') {
            let clean = line.strip_suffix('\r').unwrap_or(line).trim_end();
            entries.push((offset, clean.to_owned()));
            offset += line.len() + 1;
        }
        // split('\n') leaves a phantom empty final entry when text ends with a newline; drop it so the line count matches `.lines()`.
        if text.ends_with('\n') && entries.last().is_some_and(|(_, s)| s.is_empty()) {
            entries.pop();
        }
        Self { entries }
    }

    pub(super) fn get(&self, line_no_1based: u32) -> Option<&(usize, String)> {
        self.entries.get((line_no_1based - 1) as usize)
    }
}
