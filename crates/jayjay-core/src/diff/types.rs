use crate::syntax::SyntaxToken;

pub const CONTEXT_LINES: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffSpanStyle {
    Context,
    Added,
    Removed,
    Unchanged,
    /// Collapsed region placeholder — `spans[0].text` contains "N hidden lines".
    Separator,
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
}

#[derive(Debug, Clone)]
pub struct FileDiff {
    pub path: String,
    pub language: String,
    pub lines: Vec<DiffLine>,
}

/// A collapsed diff with a mapping from display line indices to full diff line indices.
/// Both use 1-based numbering.
#[derive(Debug, Clone)]
pub struct CollapsedDiff {
    pub diff: FileDiff,
    /// Maps 1-based display line number → 1-based full diff line number.
    /// Separator lines have no entry.
    pub display_to_full: Vec<DisplayLineMapping>,
}

#[derive(Debug, Clone)]
pub struct DisplayLineMapping {
    /// 1-based line number in the collapsed (display) diff.
    pub display_line: u32,
    /// 1-based line number in the full (uncollapsed) diff.
    pub full_line: u32,
}

/// Pre-computed line info: byte offset and content for each line number.
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
            offset += line.len() + 1; // +1 for \n
        }
        // Remove trailing empty line from trailing newline
        if text.ends_with('\n') && entries.last().is_some_and(|(_, s)| s.is_empty()) {
            entries.pop();
        }
        Self { entries }
    }

    pub(super) fn get(&self, line_no_1based: u32) -> Option<&(usize, String)> {
        self.entries.get((line_no_1based - 1) as usize)
    }
}
