use super::conflicts::annotate_conflict_lines;
use super::context::collapse_context;
use super::line_diff::{LineOp, line_diff};
use super::render_highlights::{HighlightInputs, apply_rendered_highlights, plain_spans};
use super::types::{ConflictLineKind, DiffLine, DiffSpanStyle, FileDiff, LineIndex};
use crate::syntax;

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

#[allow(clippy::too_many_lines)]
fn compute_file_diff_impl(
    path: &str,
    old: &str,
    new: &str,
    ignore_whitespace: bool,
    collapse: bool,
    skip_highlight: bool,
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

    let old_line_index = LineIndex::from_text(old);
    let new_line_index = LineIndex::from_text(new);

    let line_ops = line_diff(old, new, ignore_whitespace);

    let mut result_lines = Vec::new();
    let mut old_idx: u32 = 1;
    let mut new_idx: u32 = 1;

    let mut op_pos = 0;
    while op_pos < line_ops.len() {
        match line_ops[op_pos] {
            LineOp::Equal => {
                if let Some((_byte_start, text)) = new_line_index.get(new, new_idx) {
                    result_lines.push(DiffLine {
                        old_line_no: Some(old_idx),
                        new_line_no: Some(new_idx),
                        style: DiffSpanStyle::Context,
                        spans: plain_spans(text, DiffSpanStyle::Context),
                        conflict_kind: ConflictLineKind::None,
                        no_eof_newline: false,
                        context_region: None,
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
                    if let (Some((_old_byte, old_text)), Some((_new_byte, new_text))) = (
                        old_line_index.get(old, old_ln),
                        new_line_index.get(new, new_ln),
                    ) {
                        result_lines.push(DiffLine {
                            old_line_no: Some(old_ln),
                            new_line_no: None,
                            style: DiffSpanStyle::Removed,
                            spans: plain_spans(old_text, DiffSpanStyle::Removed),
                            conflict_kind: ConflictLineKind::None,
                            no_eof_newline: false,
                            context_region: None,
                        });
                        result_lines.push(DiffLine {
                            old_line_no: None,
                            new_line_no: Some(new_ln),
                            style: DiffSpanStyle::Added,
                            spans: plain_spans(new_text, DiffSpanStyle::Added),
                            conflict_kind: ConflictLineKind::None,
                            no_eof_newline: false,
                            context_region: None,
                        });
                    }
                }

                for &old_ln in &removed_indices[paired_count..] {
                    if let Some((_byte_start, text)) = old_line_index.get(old, old_ln) {
                        result_lines.push(DiffLine {
                            old_line_no: Some(old_ln),
                            new_line_no: None,
                            style: DiffSpanStyle::Removed,
                            spans: plain_spans(text, DiffSpanStyle::Unchanged),
                            conflict_kind: ConflictLineKind::None,
                            no_eof_newline: false,
                            context_region: None,
                        });
                    }
                }

                for &new_ln in &added_indices[paired_count..] {
                    if let Some((_byte_start, text)) = new_line_index.get(new, new_ln) {
                        result_lines.push(DiffLine {
                            old_line_no: None,
                            new_line_no: Some(new_ln),
                            style: DiffSpanStyle::Added,
                            spans: plain_spans(text, DiffSpanStyle::Unchanged),
                            conflict_kind: ConflictLineKind::None,
                            no_eof_newline: false,
                            context_region: None,
                        });
                    }
                }
            }
            LineOp::Add => {
                if let Some((_byte_start, text)) = new_line_index.get(new, new_idx) {
                    result_lines.push(DiffLine {
                        old_line_no: None,
                        new_line_no: Some(new_idx),
                        style: DiffSpanStyle::Added,
                        spans: plain_spans(text, DiffSpanStyle::Unchanged),
                        conflict_kind: ConflictLineKind::None,
                        no_eof_newline: false,
                        context_region: None,
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
            old_line_index: &old_line_index,
            new_line_index: &new_line_index,
            language,
            skip_highlight,
        },
    );

    FileDiff {
        path: path.to_owned(),
        language: language.to_owned(),
        lines,
        whitespace_only_hidden,
    }
}

/// Mark each side's last line with `no_eof_newline`; `line_diff` never matches an unterminated last line, so the two sides' last lines are always distinct rows here.
fn apply_eof_markers(lines: &mut [DiffLine], no_eof_old: bool, no_eof_new: bool) {
    if no_eof_old && let Some(idx) = lines.iter().rposition(|l| l.old_line_no.is_some()) {
        lines[idx].no_eof_newline = true;
    }
    if no_eof_new && let Some(idx) = lines.iter().rposition(|l| l.new_line_no.is_some()) {
        lines[idx].no_eof_newline = true;
    }
}
