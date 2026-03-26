use super::context::collapse_context;
use super::highlights::apply_highlights;
use super::line_diff::{LineOp, line_diff};
use super::types::{DiffLine, DiffSpanStyle, FileDiff, LineMap};
use super::word_diff::word_diff_paired_line;
use crate::syntax;

pub fn compute_file_diff(path: &str, old: &str, new: &str, ignore_whitespace: bool) -> FileDiff {
    compute_file_diff_impl(path, old, new, ignore_whitespace, true)
}

pub fn compute_file_diff_full(path: &str, old: &str, new: &str, ignore_whitespace: bool) -> FileDiff {
    compute_file_diff_impl(path, old, new, ignore_whitespace, false)
}

fn compute_file_diff_impl(
    path: &str,
    old: &str,
    new: &str,
    ignore_whitespace: bool,
    collapse: bool,
) -> FileDiff {
    let language = syntax::language_for_path(path);

    if old.is_empty() && new.is_empty() {
        return FileDiff {
            path: path.to_owned(),
            language: language.to_owned(),
            lines: vec![],
        };
    }

    let old_line_map = LineMap::from_text(old);
    let new_line_map = LineMap::from_text(new);
    let old_highlights = syntax::highlight(old, language);
    let new_highlights = syntax::highlight(new, language);

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
                if let Some((byte_start, text)) = new_line_map.get(new_idx) {
                    let spans = apply_highlights(
                        text,
                        *byte_start,
                        &new_highlights,
                        DiffSpanStyle::Context,
                    );
                    result_lines.push(DiffLine {
                        old_line_no: Some(old_idx),
                        new_line_no: Some(new_idx),
                        style: DiffSpanStyle::Context,
                        spans,
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
                            style: DiffSpanStyle::Removed,
                            spans: rem_spans,
                        });
                        result_lines.push(DiffLine {
                            old_line_no: None,
                            new_line_no: Some(new_ln),
                            style: DiffSpanStyle::Added,
                            spans: add_spans,
                        });
                    }
                }

                for &old_ln in &removed_indices[paired_count..] {
                    if let Some((byte_start, text)) = old_line_map.get(old_ln) {
                        let spans = apply_highlights(
                            text,
                            *byte_start,
                            &old_highlights,
                            DiffSpanStyle::Unchanged,
                        );
                        result_lines.push(DiffLine {
                            old_line_no: Some(old_ln),
                            new_line_no: None,
                            style: DiffSpanStyle::Removed,
                            spans,
                        });
                    }
                }

                for &new_ln in &added_indices[paired_count..] {
                    if let Some((byte_start, text)) = new_line_map.get(new_ln) {
                        let spans = apply_highlights(
                            text,
                            *byte_start,
                            &new_highlights,
                            DiffSpanStyle::Unchanged,
                        );
                        result_lines.push(DiffLine {
                            old_line_no: None,
                            new_line_no: Some(new_ln),
                            style: DiffSpanStyle::Added,
                            spans,
                        });
                    }
                }
            }
            LineOp::Add => {
                if let Some((byte_start, text)) = new_line_map.get(new_idx) {
                    let spans = apply_highlights(
                        text,
                        *byte_start,
                        &new_highlights,
                        DiffSpanStyle::Unchanged,
                    );
                    result_lines.push(DiffLine {
                        old_line_no: None,
                        new_line_no: Some(new_idx),
                        style: DiffSpanStyle::Added,
                        spans,
                    });
                }
                new_idx += 1;
                op_pos += 1;
            }
        }
    }

    let lines = if collapse {
        collapse_context(result_lines)
    } else {
        result_lines
    };

    FileDiff {
        path: path.to_owned(),
        language: language.to_owned(),
        lines,
    }
}
