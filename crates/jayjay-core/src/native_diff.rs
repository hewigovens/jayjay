use jj_lib::diff::{ContentDiff, DiffHunkKind};

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

    let mut op_pos = 0;
    while op_pos < line_ops.len() {
        match line_ops[op_pos] {
            LineOp::Equal => {
                if let Some((byte_start, text)) = new_line_map.get(new_idx) {
                    let spans =
                        apply_highlights(text, *byte_start, &new_highlights, DiffStyle::Context);
                    result_lines.push(DiffLine {
                        old_line_no: Some(old_idx),
                        new_line_no: Some(new_idx),
                        style: DiffStyle::Context,
                        spans,
                    });
                }
                old_idx += 1;
                new_idx += 1;
                op_pos += 1;
            }
            LineOp::Remove => {
                // Collect consecutive removes followed by consecutive adds
                let mut removed_indices = Vec::new();
                while op_pos < line_ops.len() && line_ops[op_pos] == LineOp::Remove {
                    removed_indices.push(old_idx);
                    old_idx += 1;
                    op_pos += 1;
                }
                let mut added_indices = Vec::new();
                while op_pos < line_ops.len() && line_ops[op_pos] == LineOp::Add {
                    added_indices.push(new_idx);
                    new_idx += 1;
                    op_pos += 1;
                }

                // Pair up removed and added lines for word-level diff
                let paired_count = removed_indices.len().min(added_indices.len());

                for i in 0..paired_count {
                    let old_ln = removed_indices[i];
                    let new_ln = added_indices[i];
                    if let (Some((old_byte, old_text)), Some((new_byte, new_text))) =
                        (old_line_map.get(old_ln), new_line_map.get(new_ln))
                    {
                        let (rem_spans, add_spans) = word_diff_paired_line(
                            old_text,
                            *old_byte,
                            &old_highlights,
                            new_text,
                            *new_byte,
                            &new_highlights,
                        );
                        result_lines.push(DiffLine {
                            old_line_no: Some(old_ln),
                            new_line_no: None,
                            style: DiffStyle::Removed,
                            spans: rem_spans,
                        });
                        result_lines.push(DiffLine {
                            old_line_no: None,
                            new_line_no: Some(new_ln),
                            style: DiffStyle::Added,
                            spans: add_spans,
                        });
                    }
                }

                // Remaining unpaired removes
                for &old_ln in &removed_indices[paired_count..] {
                    if let Some((byte_start, text)) = old_line_map.get(old_ln) {
                        let spans = apply_highlights(
                            text,
                            *byte_start,
                            &old_highlights,
                            DiffStyle::Removed,
                        );
                        result_lines.push(DiffLine {
                            old_line_no: Some(old_ln),
                            new_line_no: None,
                            style: DiffStyle::Removed,
                            spans,
                        });
                    }
                }

                // Remaining unpaired adds
                for &new_ln in &added_indices[paired_count..] {
                    if let Some((byte_start, text)) = new_line_map.get(new_ln) {
                        let spans = apply_highlights(
                            text,
                            *byte_start,
                            &new_highlights,
                            DiffStyle::Added,
                        );
                        result_lines.push(DiffLine {
                            old_line_no: None,
                            new_line_no: Some(new_ln),
                            style: DiffStyle::Added,
                            spans,
                        });
                    }
                }
            }
            LineOp::Add => {
                if let Some((byte_start, text)) = new_line_map.get(new_idx) {
                    let spans =
                        apply_highlights(text, *byte_start, &new_highlights, DiffStyle::Added);
                    result_lines.push(DiffLine {
                        old_line_no: None,
                        new_line_no: Some(new_idx),
                        style: DiffStyle::Added,
                        spans,
                    });
                }
                new_idx += 1;
                op_pos += 1;
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

/// Produce word-level diff spans for a paired removed+added line.
///
/// Returns `(removed_spans, added_spans)` where each span has:
/// - `DiffStyle::Removed` / `DiffStyle::Added` for words that actually changed (highlighted)
/// - `DiffStyle::Unchanged` for matching text within the changed line (no word highlight)
///
/// Syntax tokens are overlaid on top of the word-level diff styles.
fn word_diff_paired_line(
    old_line: &str,
    old_byte_offset: usize,
    old_highlights: &[HighlightSpan],
    new_line: &str,
    new_byte_offset: usize,
    new_highlights: &[HighlightSpan],
) -> (Vec<DiffSpan>, Vec<DiffSpan>) {
    // Build per-character word-diff style maps for each side
    let old_word_styles = word_diff_style_map(old_line, new_line, 0); // index 0 = old side
    let new_word_styles = word_diff_style_map(old_line, new_line, 1); // index 1 = new side

    let removed_spans = apply_highlights_with_word_diff(
        old_line,
        old_byte_offset,
        old_highlights,
        &old_word_styles,
        DiffStyle::Removed,
    );
    let added_spans = apply_highlights_with_word_diff(
        new_line,
        new_byte_offset,
        new_highlights,
        &new_word_styles,
        DiffStyle::Added,
    );

    (removed_spans, added_spans)
}

/// Build a per-byte style map for one side of a word diff.
///
/// Each byte position maps to `true` if the word at that position differs
/// between old and new (should be highlighted), or `false` if it matches.
fn word_diff_style_map(old_line: &str, new_line: &str, side: usize) -> Vec<bool> {
    let line = if side == 0 { old_line } else { new_line };
    let mut changed = vec![false; line.len()];

    let word_diff = ContentDiff::by_word([old_line.as_bytes(), new_line.as_bytes()]);
    // Track position in each side
    let mut positions = [0usize; 2];

    for hunk in word_diff.hunks() {
        match hunk.kind {
            DiffHunkKind::Matching => {
                // contents[0] == contents[1] for matching hunks
                let len = hunk.contents[0].len();
                positions[0] += len;
                positions[1] += len;
            }
            DiffHunkKind::Different => {
                // contents[0] = old text, contents[1] = new text
                for (i, content) in hunk.contents.iter().enumerate() {
                    let start = positions[i];
                    let end = start + content.len();
                    if i == side {
                        for j in start..end {
                            if j < changed.len() {
                                changed[j] = true;
                            }
                        }
                    }
                    positions[i] = end;
                }
            }
        }
    }

    changed
}

/// Apply syntax highlights combined with word-level diff style information.
///
/// This works like `apply_highlights` but splits spans further based on
/// word-level diff boundaries. Text that changed gets `changed_style`
/// (Added/Removed), text that matched gets `DiffStyle::Unchanged`.
fn apply_highlights_with_word_diff(
    line: &str,
    byte_offset: usize,
    highlights: &[HighlightSpan],
    word_changed: &[bool],
    changed_style: DiffStyle,
) -> Vec<DiffSpan> {
    if line.is_empty() {
        return vec![];
    }

    // First, build syntax-aware spans with a uniform diff style (like apply_highlights)
    let base_spans = apply_highlights(line, byte_offset, highlights, changed_style);

    // Now split each base span further by word-diff boundaries
    let mut result = Vec::new();
    let mut line_pos = 0usize;

    for span in &base_spans {
        let span_len = span.text.len();
        let span_start = line_pos;
        let span_end = line_pos + span_len;

        // Split this span into runs of same word-diff status
        let mut pos = span_start;
        while pos < span_end {
            let is_changed = word_changed.get(pos).copied().unwrap_or(false);
            let style = if is_changed {
                changed_style
            } else {
                DiffStyle::Unchanged
            };

            // Find the end of this run (same changed status)
            let mut run_end = pos + 1;
            while run_end < span_end {
                let next_changed = word_changed.get(run_end).copied().unwrap_or(false);
                if next_changed != is_changed {
                    break;
                }
                run_end += 1;
            }

            let text_start = pos - span_start;
            let text_end = run_end - span_start;
            result.push(DiffSpan {
                text: span.text[text_start..text_end].to_owned(),
                style,
                token: span.token,
            });

            pos = run_end;
        }

        line_pos = span_end;
    }

    result
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LineOp {
    Equal,
    Remove,
    Add,
}

/// Myers diff on lines as atomic units. No word-level splitting.
fn line_diff(old: &[&str], new: &[&str]) -> Vec<LineOp> {
    let n = old.len();
    let m = new.len();

    if n == 0 {
        return vec![LineOp::Add; m];
    }
    if m == 0 {
        return vec![LineOp::Remove; n];
    }

    // Build LCS table
    let mut dp = vec![vec![0u32; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            dp[i][j] = if old[i] == new[j] {
                dp[i + 1][j + 1] + 1
            } else {
                dp[i + 1][j].max(dp[i][j + 1])
            };
        }
    }

    // Trace back to produce ops
    let mut ops = Vec::with_capacity(n + m);
    let mut i = 0;
    let mut j = 0;
    while i < n && j < m {
        if old[i] == new[j] {
            ops.push(LineOp::Equal);
            i += 1;
            j += 1;
        } else if dp[i + 1][j] >= dp[i][j + 1] {
            ops.push(LineOp::Remove);
            i += 1;
        } else {
            ops.push(LineOp::Add);
            j += 1;
        }
    }
    while i < n {
        ops.push(LineOp::Remove);
        i += 1;
    }
    while j < m {
        ops.push(LineOp::Add);
        j += 1;
    }

    ops
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identical_files_produce_no_changes() {
        let diff = compute_file_diff("test.txt", "hello\nworld\n", "hello\nworld\n");
        assert!(diff.lines.iter().all(|l| l.style == DiffStyle::Context));
    }

    #[test]
    fn test_added_line() {
        let diff = compute_file_diff("test.txt", "a\nc\n", "a\nb\nc\n");
        let styles: Vec<_> = diff.lines.iter().map(|l| l.style).collect();
        assert_eq!(styles, vec![DiffStyle::Context, DiffStyle::Added, DiffStyle::Context]);
    }

    #[test]
    fn test_removed_line() {
        let diff = compute_file_diff("test.txt", "a\nb\nc\n", "a\nc\n");
        let styles: Vec<_> = diff.lines.iter().map(|l| l.style).collect();
        assert_eq!(styles, vec![DiffStyle::Context, DiffStyle::Removed, DiffStyle::Context]);
    }

    #[test]
    fn test_modified_line() {
        let diff = compute_file_diff("test.txt", "a\nold\nc\n", "a\nnew\nc\n");
        let styles: Vec<_> = diff.lines.iter().map(|l| l.style).collect();
        assert_eq!(styles, vec![DiffStyle::Context, DiffStyle::Removed, DiffStyle::Added, DiffStyle::Context]);
    }

    #[test]
    fn test_no_phantom_changes_on_identical_lines() {
        let content = "line1\nline2\nline3\nline4\nline5\n";
        let diff = compute_file_diff("test.txt", content, content);
        let changed: Vec<_> = diff.lines.iter().filter(|l| l.style != DiffStyle::Context && l.style != DiffStyle::Separator).collect();
        assert!(changed.is_empty(), "Identical content should have no changes, got {:?}", changed.len());
    }

    #[test]
    fn test_cargo_toml_like_diff() {
        let old = r#"tree-sitter = "0.26"
tree-sitter-highlight = "0.26"
tree-sitter-rust = "0.24"
tree-sitter-typescript = "0.23"
tree-sitter-python = "0.23"
tree-sitter-json = "0.24"
tree-sitter-toml = "0.20"
tree-sitter-html = "0.23"
tree-sitter-go = "0.23"
tree-sitter-cpp = "0.23"
"#;
        let new = r#"tree-sitter = "0.26"
tree-sitter-highlight = "0.26"
tree-sitter-rust = "0.24"
tree-sitter-javascript = "0.25"
tree-sitter-typescript = "0.23"
tree-sitter-python = "0.23"
tree-sitter-json = "0.24"
tree-sitter-toml = "0.20"
tree-sitter-css = "0.23"
tree-sitter-html = "0.23"
tree-sitter-go = "0.23"
tree-sitter-c = "0.23"
tree-sitter-cpp = "0.23"
"#;
        let diff = compute_file_diff("Cargo.toml", old, new);

        // Context lines should not be duplicated
        let context_texts: Vec<_> = diff.lines.iter()
            .filter(|l| l.style == DiffStyle::Context)
            .map(|l| l.spans.iter().map(|s| s.text.as_str()).collect::<String>())
            .collect();

        // "tree-sitter-toml" should appear exactly once as context
        let toml_count = context_texts.iter().filter(|t| t.contains("tree-sitter-toml")).count();
        assert_eq!(toml_count, 1, "tree-sitter-toml should appear once as context, got {toml_count}");

        // No line should appear as both context AND removed
        for line in &diff.lines {
            if line.style == DiffStyle::Context {
                let text: String = line.spans.iter().map(|s| s.text.as_str()).collect();
                let also_removed = diff.lines.iter().any(|l| {
                    l.style == DiffStyle::Removed && l.spans.iter().map(|s| s.text.as_str()).collect::<String>() == text
                });
                assert!(!also_removed, "Line '{text}' is both context and removed");
            }
        }
    }

    #[test]
    fn test_line_numbers_are_correct() {
        let diff = compute_file_diff("test.txt", "a\nb\nc\n", "a\nx\nc\n");
        for line in &diff.lines {
            match line.style {
                DiffStyle::Context => {
                    assert!(line.old_line_no.is_some());
                    assert!(line.new_line_no.is_some());
                }
                DiffStyle::Removed => {
                    assert!(line.old_line_no.is_some());
                    assert!(line.new_line_no.is_none());
                }
                DiffStyle::Added => {
                    assert!(line.old_line_no.is_none());
                    assert!(line.new_line_no.is_some());
                }
                _ => {}
            }
        }
    }

    #[test]
    fn test_empty_to_content() {
        let diff = compute_file_diff("test.txt", "", "hello\nworld\n");
        assert!(diff.lines.iter().all(|l| l.style == DiffStyle::Added));
        assert_eq!(diff.lines.len(), 2);
    }

    #[test]
    fn test_content_to_empty() {
        let diff = compute_file_diff("test.txt", "hello\nworld\n", "");
        assert!(diff.lines.iter().all(|l| l.style == DiffStyle::Removed));
        assert_eq!(diff.lines.len(), 2);
    }

    #[test]
    fn test_context_collapsing() {
        let mut old_lines: Vec<String> = (1..=20).map(|i| format!("line {i}")).collect();
        let mut new_lines = old_lines.clone();
        new_lines[9] = "CHANGED".to_string(); // Change line 10

        let old = old_lines.join("\n") + "\n";
        let new = new_lines.join("\n") + "\n";
        let diff = compute_file_diff("test.txt", &old, &new);

        let separators: Vec<_> = diff.lines.iter().filter(|l| l.style == DiffStyle::Separator).collect();
        assert!(!separators.is_empty(), "Should have separator lines for collapsed context");
    }

    // ── Word-level diff highlighting tests ──────────────────────────

    /// Helper: collect (text, style) pairs from spans of a DiffLine.
    fn span_info(line: &DiffLine) -> Vec<(&str, DiffStyle)> {
        line.spans.iter().map(|s| (s.text.as_str(), s.style)).collect()
    }

    #[test]
    fn test_word_diff_single_word_change() {
        // "hello world" → "hello earth" — only the second word changes
        let diff = compute_file_diff("test.txt", "hello world\n", "hello earth\n");
        let styles: Vec<_> = diff.lines.iter().map(|l| l.style).collect();
        assert_eq!(styles, vec![DiffStyle::Removed, DiffStyle::Added]);

        // Removed line: "hello " is unchanged, "world" is removed
        let rem = span_info(&diff.lines[0]);
        assert!(
            rem.iter().any(|(t, s)| t.contains("hello") && *s == DiffStyle::Unchanged),
            "matching text 'hello' should be Unchanged in removed line, got: {rem:?}"
        );
        assert!(
            rem.iter().any(|(t, s)| t.contains("world") && *s == DiffStyle::Removed),
            "changed text 'world' should be Removed in removed line, got: {rem:?}"
        );

        // Added line: "hello " is unchanged, "earth" is added
        let add = span_info(&diff.lines[1]);
        assert!(
            add.iter().any(|(t, s)| t.contains("hello") && *s == DiffStyle::Unchanged),
            "matching text 'hello' should be Unchanged in added line, got: {add:?}"
        );
        assert!(
            add.iter().any(|(t, s)| t.contains("earth") && *s == DiffStyle::Added),
            "changed text 'earth' should be Added in added line, got: {add:?}"
        );
    }

    #[test]
    fn test_word_diff_preserves_line_level_style() {
        // Line-level style (DiffLine.style) should still be Removed/Added
        let diff = compute_file_diff("test.txt", "foo bar\n", "foo baz\n");
        assert_eq!(diff.lines[0].style, DiffStyle::Removed);
        assert_eq!(diff.lines[1].style, DiffStyle::Added);
    }

    #[test]
    fn test_word_diff_entirely_different_lines() {
        // Completely different content — all spans should be Removed/Added
        let diff = compute_file_diff("test.txt", "aaa\n", "zzz\n");
        let rem = span_info(&diff.lines[0]);
        assert!(
            rem.iter().all(|(_, s)| *s == DiffStyle::Removed),
            "entirely different removed line should have all Removed spans, got: {rem:?}"
        );
        let add = span_info(&diff.lines[1]);
        assert!(
            add.iter().all(|(_, s)| *s == DiffStyle::Added),
            "entirely different added line should have all Added spans, got: {add:?}"
        );
    }

    #[test]
    fn test_word_diff_prefix_change() {
        // Change at the start: "old_func(x)" → "new_func(x)"
        let diff = compute_file_diff("test.txt", "old_func(x)\n", "new_func(x)\n");
        let rem = span_info(&diff.lines[0]);
        let add = span_info(&diff.lines[1]);

        // "(x)" should be unchanged on both sides
        assert!(
            rem.iter().any(|(t, s)| t.contains("(") && *s == DiffStyle::Unchanged),
            "matching punctuation should be Unchanged in removed line, got: {rem:?}"
        );
        assert!(
            add.iter().any(|(t, s)| t.contains("(") && *s == DiffStyle::Unchanged),
            "matching punctuation should be Unchanged in added line, got: {add:?}"
        );
    }

    #[test]
    fn test_word_diff_unpaired_lines_have_no_word_highlight() {
        // 2 removes, 1 add: first pair gets word diff, second remove is unpaired
        let diff = compute_file_diff("test.txt", "aaa\nbbb\nccc\n", "AAA\nccc\n");
        // Lines: Removed(aaa), Added(AAA), Removed(bbb), Context(ccc)
        // The paired Remove(aaa)/Add(AAA) should have word-level styles
        // The unpaired Remove(bbb) should have all Removed spans (no Unchanged)
        let unpaired = diff.lines.iter().find(|l| {
            l.style == DiffStyle::Removed
                && l.spans.iter().map(|s| s.text.as_str()).collect::<String>() == "bbb"
        });
        assert!(unpaired.is_some(), "should find unpaired removed line 'bbb'");
        let unpaired = unpaired.unwrap();
        assert!(
            unpaired.spans.iter().all(|s| s.style == DiffStyle::Removed),
            "unpaired removed line should have all Removed spans, got: {:?}",
            span_info(unpaired)
        );
    }

    #[test]
    fn test_word_diff_multiple_changes_in_line() {
        // Multiple words change: "the quick brown fox" → "the slow brown cat"
        let diff = compute_file_diff("test.txt", "the quick brown fox\n", "the slow brown cat\n");
        let rem = span_info(&diff.lines[0]);
        let add = span_info(&diff.lines[1]);

        // "the" and "brown" should be unchanged on both sides
        // (word boundaries may include adjacent whitespace, so use contains)
        assert!(
            rem.iter().any(|(t, s)| t.contains("the") && *s == DiffStyle::Unchanged),
            "'the' should be Unchanged in removed, got: {rem:?}"
        );
        assert!(
            rem.iter().any(|(t, s)| t.contains("brown") && *s == DiffStyle::Unchanged),
            "'brown' should be Unchanged in removed, got: {rem:?}"
        );
        assert!(
            add.iter().any(|(t, s)| t.contains("the") && *s == DiffStyle::Unchanged),
            "'the' should be Unchanged in added, got: {add:?}"
        );
        assert!(
            add.iter().any(|(t, s)| t.contains("brown") && *s == DiffStyle::Unchanged),
            "'brown' should be Unchanged in added, got: {add:?}"
        );

        // "quick" and "fox" should be Removed; "slow" and "cat" should be Added
        assert!(
            rem.iter().any(|(t, s)| t.contains("quick") && *s == DiffStyle::Removed),
            "'quick' should be Removed, got: {rem:?}"
        );
        assert!(
            rem.iter().any(|(t, s)| t.contains("fox") && *s == DiffStyle::Removed),
            "'fox' should be Removed, got: {rem:?}"
        );
        assert!(
            add.iter().any(|(t, s)| t.contains("slow") && *s == DiffStyle::Added),
            "'slow' should be Added, got: {add:?}"
        );
        assert!(
            add.iter().any(|(t, s)| t.contains("cat") && *s == DiffStyle::Added),
            "'cat' should be Added, got: {add:?}"
        );
    }

    #[test]
    fn test_word_diff_all_text_accounted_for() {
        // All text from both lines should be present in spans
        let diff = compute_file_diff("test.txt", "hello world\n", "hello earth\n");
        let rem_text: String = diff.lines[0].spans.iter().map(|s| s.text.as_str()).collect();
        let add_text: String = diff.lines[1].spans.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(rem_text, "hello world");
        assert_eq!(add_text, "hello earth");
    }

    #[test]
    fn test_word_diff_whitespace_change() {
        // Only whitespace changes: "a  b" → "a b"
        let diff = compute_file_diff("test.txt", "a  b\n", "a b\n");
        let rem = span_info(&diff.lines[0]);
        let add = span_info(&diff.lines[1]);

        // "a" and "b" should be unchanged on both sides
        // (spans may include adjacent whitespace, so use contains)
        assert!(
            rem.iter().any(|(t, s)| t.contains('a') && *s == DiffStyle::Unchanged),
            "'a' should be Unchanged in removed, got: {rem:?}"
        );
        assert!(
            rem.iter().any(|(t, s)| t.contains('b') && *s == DiffStyle::Unchanged),
            "'b' should be Unchanged in removed, got: {rem:?}"
        );
        assert!(
            add.iter().any(|(t, s)| t.contains('a') && *s == DiffStyle::Unchanged),
            "'a' should be Unchanged in added, got: {add:?}"
        );
        assert!(
            add.iter().any(|(t, s)| t.contains('b') && *s == DiffStyle::Unchanged),
            "'b' should be Unchanged in added, got: {add:?}"
        );
    }

    #[test]
    fn test_word_diff_context_lines_unaffected() {
        // Context lines should still have Context style spans (no Unchanged)
        let diff = compute_file_diff("test.txt", "ctx\nold\n", "ctx\nnew\n");
        let ctx_line = &diff.lines[0];
        assert_eq!(ctx_line.style, DiffStyle::Context);
        assert!(
            ctx_line.spans.iter().all(|s| s.style == DiffStyle::Context),
            "context line spans should all be Context, got: {:?}",
            span_info(ctx_line)
        );
    }

    #[test]
    fn test_word_diff_empty_line_paired_with_content() {
        // Empty line changed to content (or vice versa)
        let diff = compute_file_diff("test.txt", "a\n\nc\n", "a\nhello\nc\n");
        // The empty line and "hello" line form a remove/add pair
        let rem_line = diff.lines.iter().find(|l| l.style == DiffStyle::Removed);
        let add_line = diff.lines.iter().find(|l| l.style == DiffStyle::Added);
        assert!(rem_line.is_some());
        assert!(add_line.is_some());
        // The added line should have "hello" marked as Added
        let add_spans = span_info(add_line.unwrap());
        assert!(
            add_spans.iter().any(|(t, s)| *t == "hello" && *s == DiffStyle::Added),
            "expected 'hello' as Added, got: {add_spans:?}"
        );
    }
}
