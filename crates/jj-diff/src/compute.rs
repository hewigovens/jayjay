use super::conflicts::annotate_conflict_lines;
use super::context::collapse_context;
use super::highlights::apply_highlights;
use super::line_diff::{LineOp, line_diff};
use super::render_highlights::{HighlightInputs, apply_rendered_highlights, plain_spans};
use super::types::{ConflictLineKind, DiffLine, DiffSpan, DiffSpanStyle, FileDiff, LineMap};
use crate::syntax;

/// Standalone per-line highlight for blame/annotate — do not fold back into a diff-against-empty; that produced Added spans, collapsed context, and EOF markers in blame views.
pub fn highlight_file(path: &str, content: &str) -> Vec<Vec<DiffSpan>> {
    if content.is_empty() {
        return vec![];
    }
    let language = syntax::language_for_path(path);
    let highlights = if should_skip_highlight(path) {
        vec![]
    } else {
        syntax::highlight(content, language)
    };
    let line_map = LineMap::from_text(content);
    let mut lines = Vec::new();
    let mut n: u32 = 1;
    while let Some((byte_start, text)) = line_map.get(n) {
        lines.push(apply_highlights(
            text,
            *byte_start,
            &highlights,
            DiffSpanStyle::Context,
        ));
        n += 1;
    }
    lines
}

pub fn compute_file_diff(path: &str, old: &str, new: &str, ignore_whitespace: bool) -> FileDiff {
    compute_file_diff_impl(path, old, new, ignore_whitespace, true, false)
}

pub fn compute_file_diff_full(
    path: &str,
    old: &str,
    new: &str,
    ignore_whitespace: bool,
) -> FileDiff {
    compute_file_diff_impl(path, old, new, ignore_whitespace, false, false)
}

/// Full diff without syntax highlighting, for callers that only need line structure (e.g. selection sets) — tree-sitter setup costs tens of milliseconds per file.
pub fn compute_file_diff_full_plain(
    path: &str,
    old: &str,
    new: &str,
    ignore_whitespace: bool,
) -> FileDiff {
    compute_file_diff_impl(path, old, new, ignore_whitespace, false, true)
}

/// File extensions that are generated/data — skip syntax highlighting.
const SKIP_HIGHLIGHT_EXTENSIONS: &[&str] = &["lock", "csv", "tsv", "svg"];

fn should_skip_highlight(path: &str) -> bool {
    if let Some(ext) = path.rsplit('.').next() {
        return SKIP_HIGHLIGHT_EXTENSIONS.contains(&ext);
    }
    false
}

fn compute_file_diff_impl(
    path: &str,
    old: &str,
    new: &str,
    ignore_whitespace: bool,
    collapse: bool,
    force_skip_highlight: bool,
) -> FileDiff {
    let language = syntax::language_for_path(path);

    if old.is_empty() && new.is_empty() {
        return FileDiff {
            path: path.to_owned(),
            language: language.to_owned(),
            lines: vec![],
            whitespace_only_hidden: false,
        };
    }

    let old_line_map = LineMap::from_text(old);
    let new_line_map = LineMap::from_text(new);
    let skip_highlight = force_skip_highlight || should_skip_highlight(path);

    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();
    let line_ops = line_diff(&old_lines, &new_lines, ignore_whitespace);

    let mut result_lines = Vec::new();
    let mut old_idx: u32 = 1;
    let mut new_idx: u32 = 1;

    let mut op_pos = 0;
    while op_pos < line_ops.len() {
        match line_ops[op_pos] {
            LineOp::Equal => {
                if let Some((_byte_start, text)) = new_line_map.get(new_idx) {
                    result_lines.push(DiffLine {
                        old_line_no: Some(old_idx),
                        new_line_no: Some(new_idx),
                        style: DiffSpanStyle::Context,
                        spans: plain_spans(text, DiffSpanStyle::Context),
                        conflict_kind: ConflictLineKind::None,
                        no_eof_newline: false,
                    });
                }
                old_idx += 1;
                new_idx += 1;
                op_pos += 1;
            }
            LineOp::Remove => {
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

                let paired_count = removed_indices.len().min(added_indices.len());

                for i in 0..paired_count {
                    let old_ln = removed_indices[i];
                    let new_ln = added_indices[i];
                    if let (Some((_old_byte, old_text)), Some((_new_byte, new_text))) =
                        (old_line_map.get(old_ln), new_line_map.get(new_ln))
                    {
                        result_lines.push(DiffLine {
                            old_line_no: Some(old_ln),
                            new_line_no: None,
                            style: DiffSpanStyle::Removed,
                            spans: plain_spans(old_text, DiffSpanStyle::Removed),
                            conflict_kind: ConflictLineKind::None,
                            no_eof_newline: false,
                        });
                        result_lines.push(DiffLine {
                            old_line_no: None,
                            new_line_no: Some(new_ln),
                            style: DiffSpanStyle::Added,
                            spans: plain_spans(new_text, DiffSpanStyle::Added),
                            conflict_kind: ConflictLineKind::None,
                            no_eof_newline: false,
                        });
                    }
                }

                for &old_ln in &removed_indices[paired_count..] {
                    if let Some((_byte_start, text)) = old_line_map.get(old_ln) {
                        result_lines.push(DiffLine {
                            old_line_no: Some(old_ln),
                            new_line_no: None,
                            style: DiffSpanStyle::Removed,
                            spans: plain_spans(text, DiffSpanStyle::Unchanged),
                            conflict_kind: ConflictLineKind::None,
                            no_eof_newline: false,
                        });
                    }
                }

                for &new_ln in &added_indices[paired_count..] {
                    if let Some((_byte_start, text)) = new_line_map.get(new_ln) {
                        result_lines.push(DiffLine {
                            old_line_no: None,
                            new_line_no: Some(new_ln),
                            style: DiffSpanStyle::Added,
                            spans: plain_spans(text, DiffSpanStyle::Unchanged),
                            conflict_kind: ConflictLineKind::None,
                            no_eof_newline: false,
                        });
                    }
                }
            }
            LineOp::Add => {
                if let Some((_byte_start, text)) = new_line_map.get(new_idx) {
                    result_lines.push(DiffLine {
                        old_line_no: None,
                        new_line_no: Some(new_idx),
                        style: DiffSpanStyle::Added,
                        spans: plain_spans(text, DiffSpanStyle::Unchanged),
                        conflict_kind: ConflictLineKind::None,
                        no_eof_newline: false,
                    });
                }
                new_idx += 1;
                op_pos += 1;
            }
        }
    }

    // Rust's .lines() strips the trailing newline; reconcile bytes vs lines so EOF markers surface.
    let no_eof_old = !old.is_empty() && !old.ends_with('\n');
    let no_eof_new = !new.is_empty() && !new.ends_with('\n');
    let eof_differs = no_eof_old != no_eof_new;
    let any_change = result_lines
        .iter()
        .any(|l| matches!(l.style, DiffSpanStyle::Added | DiffSpanStyle::Removed));

    let mut whitespace_only_hidden = false;

    if eof_differs {
        apply_eof_markers(&mut result_lines, no_eof_old, no_eof_new);
    } else if !any_change && old != new && ignore_whitespace {
        whitespace_only_hidden = true;
    }

    annotate_conflict_lines(&mut result_lines);

    let mut lines = if collapse {
        collapse_context(result_lines)
    } else {
        result_lines
    };
    apply_rendered_highlights(
        &mut lines,
        HighlightInputs {
            old,
            new,
            old_line_map: &old_line_map,
            new_line_map: &new_line_map,
            language,
            skip_highlight,
            collapse,
        },
    );

    FileDiff {
        path: path.to_owned(),
        language: language.to_owned(),
        lines,
        whitespace_only_hidden,
    }
}

/// Mark each side's last line with `no_eof_newline`; split a shared-Context last line into a pair so the marker can attribute per side.
fn apply_eof_markers(lines: &mut Vec<DiffLine>, no_eof_old: bool, no_eof_new: bool) {
    let last_old_idx = lines.iter().rposition(|l| l.old_line_no.is_some());
    let last_new_idx = lines.iter().rposition(|l| l.new_line_no.is_some());

    if let (Some(oi), Some(ni)) = (last_old_idx, last_new_idx)
        && oi == ni
        && lines[oi].style == DiffSpanStyle::Context
    {
        split_context_for_eof(lines, oi, no_eof_old, no_eof_new);
        return;
    }

    if no_eof_old && let Some(idx) = last_old_idx {
        lines[idx].no_eof_newline = true;
    }
    if no_eof_new && let Some(idx) = last_new_idx {
        lines[idx].no_eof_newline = true;
    }
}

/// Spans stay as Context — the text is identical, so no word-level highlight.
fn split_context_for_eof(
    lines: &mut Vec<DiffLine>,
    idx: usize,
    no_eof_old: bool,
    no_eof_new: bool,
) {
    let original = lines.remove(idx);
    let mut removed = original.clone();
    removed.new_line_no = None;
    removed.style = DiffSpanStyle::Removed;
    removed.no_eof_newline = no_eof_old;

    let mut added = original;
    added.old_line_no = None;
    added.style = DiffSpanStyle::Added;
    added.no_eof_newline = no_eof_new;

    lines.insert(idx, removed);
    lines.insert(idx + 1, added);
}
