use super::conflicts::annotate_conflict_lines;
use super::context::collapse_context;
use super::highlights::apply_highlights;
use super::line_diff::{LineOp, line_diff};
use super::types::{
    ChangeGroup, ConflictLineKind, DiffLine, DiffSide, DiffSpan, DiffSpanStyle, FileDiff, LineMap,
};
use super::word_diff::word_diff_paired_line;
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
    compute_file_diff_impl(path, old, new, ignore_whitespace, true)
}

pub fn compute_file_diff_full(
    path: &str,
    old: &str,
    new: &str,
    ignore_whitespace: bool,
) -> FileDiff {
    compute_file_diff_impl(path, old, new, ignore_whitespace, false)
}

/// The canonical group index used by review marks and notes across SwiftUI and CLI surfaces.
pub fn change_groups(lines: &[DiffLine]) -> Vec<ChangeGroup> {
    change_group_ranges(lines)
        .into_iter()
        .map(|(index, start, end)| build_change_group(index, start, end, lines))
        .collect()
}

/// Scans ranges first and materializes only the matching group's payload, so reconciling a note does not clone every changed line's text.
pub fn change_group_for_anchor(
    lines: &[DiffLine],
    side: DiffSide,
    line_number: u32,
    anchor_excerpt: &str,
) -> Option<ChangeGroup> {
    change_group_ranges(lines)
        .into_iter()
        .find(|(_, start, end)| {
            lines[*start..=*end]
                .iter()
                .any(|line| line_matches_anchor(line, side, line_number, anchor_excerpt))
        })
        .map(|(index, start, end)| build_change_group(index, start, end, lines))
}

/// 0-based inclusive (index, start, end) spans of contiguous changed lines.
fn change_group_ranges(lines: &[DiffLine]) -> Vec<(u32, usize, usize)> {
    let mut ranges: Vec<(u32, usize, usize)> = Vec::new();
    let mut start: Option<usize> = None;

    for (line_index, line) in lines.iter().enumerate() {
        if line.is_changed() {
            if start.is_none() {
                start = Some(line_index);
            }
        } else if let Some(start_index) = start.take() {
            ranges.push((ranges.len() as u32, start_index, line_index - 1));
        }
    }

    if let Some(start_index) = start {
        ranges.push((
            ranges.len() as u32,
            start_index,
            lines.len().saturating_sub(1),
        ));
    }

    ranges
}

fn build_change_group(
    index: u32,
    start_index: usize,
    end_index: usize,
    lines: &[DiffLine],
) -> ChangeGroup {
    let group_lines = &lines[start_index..=end_index];
    let anchor = group_lines
        .iter()
        .find_map(anchor_for_line)
        .expect("change group must contain a changed line with a line number");

    ChangeGroup {
        index,
        start_line: (start_index + 1) as u32,
        end_line: (end_index + 1) as u32,
        anchor_side: anchor.0,
        anchor_line: anchor.1,
        anchor_excerpt: anchor.2,
        anchor_context: group_lines.iter().map(DiffLine::text).collect(),
    }
}

fn anchor_for_line(line: &DiffLine) -> Option<(DiffSide, u32, String)> {
    match line.style {
        DiffSpanStyle::Removed => line
            .old_line_no
            .map(|line_no| (DiffSide::Old, line_no, line.text())),
        DiffSpanStyle::Added => line
            .new_line_no
            .map(|line_no| (DiffSide::New, line_no, line.text())),
        _ => None,
    }
}

fn line_matches_anchor(
    line: &DiffLine,
    side: DiffSide,
    line_number: u32,
    anchor_excerpt: &str,
) -> bool {
    match (side, line.style) {
        (DiffSide::Old, DiffSpanStyle::Removed) => {
            line.old_line_no == Some(line_number) && line.text() == anchor_excerpt
        }
        (DiffSide::New, DiffSpanStyle::Added) => {
            line.new_line_no == Some(line_number) && line.text() == anchor_excerpt
        }
        _ => false,
    }
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
    let skip_highlight = should_skip_highlight(path);
    let old_highlights = if skip_highlight {
        vec![]
    } else {
        syntax::highlight(old, language)
    };
    let new_highlights = if skip_highlight {
        vec![]
    } else {
        syntax::highlight(new, language)
    };

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
                            conflict_kind: ConflictLineKind::None,
                            no_eof_newline: false,
                        });
                        result_lines.push(DiffLine {
                            old_line_no: None,
                            new_line_no: Some(new_ln),
                            style: DiffSpanStyle::Added,
                            spans: add_spans,
                            conflict_kind: ConflictLineKind::None,
                            no_eof_newline: false,
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
                            conflict_kind: ConflictLineKind::None,
                            no_eof_newline: false,
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
                            conflict_kind: ConflictLineKind::None,
                            no_eof_newline: false,
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

    let lines = if collapse {
        collapse_context(result_lines)
    } else {
        result_lines
    };

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
