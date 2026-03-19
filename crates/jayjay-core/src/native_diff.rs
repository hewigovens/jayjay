use jj_lib::diff::{self, DiffHunkKind};

use crate::syntax::{self, HighlightSpan, TokenKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffStyle {
    Context,
    Added,
    Removed,
    Unchanged,
    /// Collapsed region placeholder — `spans[0].text` contains "N hidden lines".
    Separator,
}

const CONTEXT_LINES: usize = 3;

#[derive(Debug, Clone)]
pub struct DiffSpan {
    pub text: String,
    pub style: DiffStyle,
    pub token: TokenKind,
}

#[derive(Debug, Clone)]
pub struct DiffLine {
    pub old_line_no: Option<u32>,
    pub new_line_no: Option<u32>,
    pub style: DiffStyle,
    pub spans: Vec<DiffSpan>,
}

#[derive(Debug, Clone)]
pub struct FileDiff {
    pub path: String,
    pub language: String,
    pub lines: Vec<DiffLine>,
}

/// Pre-computed line info: byte offset and content for each line number.
struct LineMap {
    /// (byte_start, line_content) indexed by 0-based line number
    entries: Vec<(usize, String)>,
}

impl LineMap {
    fn from_text(text: &str) -> Self {
        let mut entries = Vec::new();
        let mut offset = 0;
        for line in text.split('\n') {
            let clean = line.strip_suffix('\r').unwrap_or(line);
            entries.push((offset, clean.to_owned()));
            offset += line.len() + 1; // +1 for \n
        }
        // Remove trailing empty line from trailing newline
        if text.ends_with('\n') && entries.last().is_some_and(|(_, s)| s.is_empty()) {
            entries.pop();
        }
        Self { entries }
    }

    fn get(&self, line_no_1based: u32) -> Option<&(usize, String)> {
        self.entries.get((line_no_1based - 1) as usize)
    }
}

pub fn compute_file_diff(path: &str, old: &str, new: &str) -> FileDiff {
    let language = syntax::language_for_path(path);

    if old.is_empty() && new.is_empty() {
        return FileDiff {
            path: path.to_owned(),
            language: language.to_owned(),
            lines: vec![],
        };
    }

    // Pre-compute line maps and syntax highlights for both sides
    let old_line_map = LineMap::from_text(old);
    let new_line_map = LineMap::from_text(new);
    let old_highlights = syntax::highlight(old, language);
    let new_highlights = syntax::highlight(new, language);

    // Line-level diff to determine added/removed/context
    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();
    let line_ops = line_diff(&old_lines, &new_lines);

    let mut result_lines = Vec::new();
    let mut old_idx: u32 = 1;
    let mut new_idx: u32 = 1;

    for op in &line_ops {
        match op {
            LineOp::Equal => {
                if let Some((byte_start, text)) = new_line_map.get(new_idx) {
                    let spans = apply_highlights(text, *byte_start, &new_highlights, DiffStyle::Context);
                    result_lines.push(DiffLine {
                        old_line_no: Some(old_idx),
                        new_line_no: Some(new_idx),
                        style: DiffStyle::Context,
                        spans,
                    });
                }
                old_idx += 1;
                new_idx += 1;
            }
            LineOp::Remove => {
                if let Some((byte_start, text)) = old_line_map.get(old_idx) {
                    let spans = apply_highlights(text, *byte_start, &old_highlights, DiffStyle::Removed);
                    result_lines.push(DiffLine {
                        old_line_no: Some(old_idx),
                        new_line_no: None,
                        style: DiffStyle::Removed,
                        spans,
                    });
                }
                old_idx += 1;
            }
            LineOp::Add => {
                if let Some((byte_start, text)) = new_line_map.get(new_idx) {
                    let spans = apply_highlights(text, *byte_start, &new_highlights, DiffStyle::Added);
                    result_lines.push(DiffLine {
                        old_line_no: None,
                        new_line_no: Some(new_idx),
                        style: DiffStyle::Added,
                        spans,
                    });
                }
                new_idx += 1;
            }
        }
    }

    let collapsed = collapse_context(result_lines);

    FileDiff {
        path: path.to_owned(),
        language: language.to_owned(),
        lines: collapsed,
    }
}

/// Collapse long runs of context lines, keeping only CONTEXT_LINES around changes.
fn collapse_context(lines: Vec<DiffLine>) -> Vec<DiffLine> {
    if lines.is_empty() {
        return lines;
    }

    // Find indices of all changed (non-context) lines
    let changed_indices: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.style != DiffStyle::Context)
        .map(|(i, _)| i)
        .collect();

    if changed_indices.is_empty() {
        // All context — just show first/last few lines
        if lines.len() <= CONTEXT_LINES * 2 + 1 {
            return lines;
        }
        let mut result: Vec<DiffLine> = lines[..CONTEXT_LINES].to_vec();
        let hidden = lines.len() - CONTEXT_LINES * 2;
        result.push(separator_line(hidden));
        result.extend_from_slice(&lines[lines.len() - CONTEXT_LINES..]);
        return result;
    }

    // Mark which lines to keep (within CONTEXT_LINES of a change)
    let mut keep = vec![false; lines.len()];
    for &idx in &changed_indices {
        let start = idx.saturating_sub(CONTEXT_LINES);
        let end = (idx + CONTEXT_LINES + 1).min(lines.len());
        for i in start..end {
            keep[i] = true;
        }
    }

    let mut result = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if keep[i] {
            result.push(lines[i].clone());
            i += 1;
        } else {
            // Count consecutive hidden lines
            let start = i;
            while i < lines.len() && !keep[i] {
                i += 1;
            }
            let hidden = i - start;
            if hidden > 0 {
                result.push(separator_line(hidden));
            }
        }
    }

    result
}

fn separator_line(hidden_count: usize) -> DiffLine {
    DiffLine {
        old_line_no: None,
        new_line_no: None,
        style: DiffStyle::Separator,
        spans: vec![DiffSpan {
            text: format!("{hidden_count} hidden lines"),
            style: DiffStyle::Separator,
            token: TokenKind::Plain,
        }],
    }
}

/// Apply pre-computed syntax highlights to a line at a given byte offset.
fn apply_highlights(
    line: &str,
    byte_offset: usize,
    highlights: &[HighlightSpan],
    diff_style: DiffStyle,
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
            token: TokenKind::Plain,
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
                token: TokenKind::Plain,
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
            token: TokenKind::Plain,
        });
    }

    spans
}

// Simple line diff using jj-lib's diff on line-separated content.
enum LineOp {
    Equal,
    Remove,
    Add,
}

fn line_diff(old: &[&str], new: &[&str]) -> Vec<LineOp> {
    // Join with newlines and use jj-lib diff on whole lines
    let old_joined = old.join("\n");
    let new_joined = new.join("\n");
    let old_bytes = old_joined.as_bytes();
    let new_bytes = new_joined.as_bytes();
    let inputs = [old_bytes, new_bytes];
    let hunks: Vec<diff::DiffHunk<'_>> = diff::diff(&inputs);

    let mut ops = Vec::new();

    for hunk in &hunks {
        match hunk.kind {
            DiffHunkKind::Matching => {
                let text = std::str::from_utf8(hunk.contents[0]).unwrap_or("");
                // Count lines in matching section
                let line_count = if text.is_empty() {
                    0
                } else {
                    text.chars().filter(|&c| c == '\n').count() + 1
                };
                for _ in 0..line_count {
                    ops.push(LineOp::Equal);
                }
            }
            DiffHunkKind::Different => {
                let old_text = std::str::from_utf8(hunk.contents[0]).unwrap_or("");
                let new_text = std::str::from_utf8(hunk.contents[1]).unwrap_or("");

                let old_line_count = if old_text.is_empty() {
                    0
                } else {
                    old_text.chars().filter(|&c| c == '\n').count() + 1
                };
                let new_line_count = if new_text.is_empty() {
                    0
                } else {
                    new_text.chars().filter(|&c| c == '\n').count() + 1
                };

                for _ in 0..old_line_count {
                    ops.push(LineOp::Remove);
                }
                for _ in 0..new_line_count {
                    ops.push(LineOp::Add);
                }
            }
        }
    }

    ops
}
