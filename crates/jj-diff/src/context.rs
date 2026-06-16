use crate::syntax::SyntaxToken;

use super::types::{
    CONTEXT_LINES, CollapsedDiff, ConflictLineKind, DiffLine, DiffSpan, DiffSpanStyle,
    DisplayLineMapping, FileDiff,
};

const COLLAPSED_CONTEXT_THRESHOLD: usize = 2;

/// Collapse long runs of context lines, keeping only CONTEXT_LINES around changes.
pub(super) fn collapse_context(lines: Vec<DiffLine>) -> Vec<DiffLine> {
    let full = FileDiff {
        path: String::new(),
        language: String::new(),
        lines,
        whitespace_only_hidden: false,
    };
    collapse_context_with_mapping(&full).diff.lines
}

/// Collapse context and return a mapping from display lines to full diff lines.
pub fn collapse_context_with_mapping(full_diff: &FileDiff) -> CollapsedDiff {
    let lines = &full_diff.lines;
    if lines.is_empty() {
        return CollapsedDiff {
            diff: full_diff.clone(),
            display_to_full: vec![],
        };
    }

    let is_changed =
        |l: &DiffLine| l.style == DiffSpanStyle::Added || l.style == DiffSpanStyle::Removed;
    let changed_indices: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| is_changed(l))
        .map(|(i, _)| i)
        .collect();

    if changed_indices.is_empty() {
        if lines.len() <= CONTEXT_LINES * 2 + COLLAPSED_CONTEXT_THRESHOLD {
            let mapping = (0..lines.len())
                .map(|i| DisplayLineMapping {
                    display_line: (i + 1) as u32,
                    full_line: (i + 1) as u32,
                })
                .collect();
            return CollapsedDiff {
                diff: full_diff.clone(),
                display_to_full: mapping,
            };
        }
        let hidden = lines.len() - CONTEXT_LINES * 2;

        let mut result: Vec<DiffLine> = lines[..CONTEXT_LINES].to_vec();
        let mut mapping: Vec<DisplayLineMapping> = (0..CONTEXT_LINES)
            .map(|i| DisplayLineMapping {
                display_line: (i + 1) as u32,
                full_line: (i + 1) as u32,
            })
            .collect();
        result.push(separator_line(hidden));
        // separator has no mapping entry
        for i in 0..CONTEXT_LINES {
            result.push(lines[lines.len() - CONTEXT_LINES + i].clone());
            mapping.push(DisplayLineMapping {
                display_line: (result.len()) as u32,
                full_line: (lines.len() - CONTEXT_LINES + i + 1) as u32,
            });
        }
        return CollapsedDiff {
            diff: FileDiff {
                path: full_diff.path.clone(),
                language: full_diff.language.clone(),
                lines: result,
                whitespace_only_hidden: full_diff.whitespace_only_hidden,
            },
            display_to_full: mapping,
        };
    }

    let mut keep = vec![false; lines.len()];
    for idx in changed_indices.iter().copied() {
        let start = idx.saturating_sub(CONTEXT_LINES);
        let end = (idx + CONTEXT_LINES + 1).min(lines.len());
        for slot in &mut keep[start..end] {
            *slot = true;
        }
    }

    keep_whole_conflict_blocks(lines, &mut keep);

    let mut result: Vec<DiffLine> = Vec::new();
    let mut mapping: Vec<DisplayLineMapping> = Vec::new();
    let mut i = 0usize;
    while i < lines.len() {
        if keep[i] {
            result.push(lines[i].clone());
            mapping.push(DisplayLineMapping {
                display_line: result.len() as u32,
                full_line: (i + 1) as u32,
            });
            i += 1;
        } else {
            let start = i;
            while i < lines.len() && !keep[i] {
                i += 1;
            }
            // `i` advanced at least one step in the while loop above, so `hidden >= 1`.
            let hidden = i - start;
            if hidden <= COLLAPSED_CONTEXT_THRESHOLD {
                for (offset, line) in lines[start..i].iter().enumerate() {
                    result.push(line.clone());
                    mapping.push(DisplayLineMapping {
                        display_line: result.len() as u32,
                        full_line: (start + offset + 1) as u32,
                    });
                }
            } else {
                result.push(separator_line(hidden));
            }
        }
    }

    CollapsedDiff {
        diff: FileDiff {
            path: full_diff.path.clone(),
            language: full_diff.language.clone(),
            lines: result,
            whitespace_only_hidden: full_diff.whitespace_only_hidden,
        },
        display_to_full: mapping,
    }
}

/// Keep any partly-kept conflict block whole; collapsing inside a `Start..=End`
/// span strands a marker and makes `conflict_block` swallow the rest of the diff.
fn keep_whole_conflict_blocks(lines: &[DiffLine], keep: &mut [bool]) {
    let mut i = 0usize;
    while i < lines.len() {
        if lines[i].conflict_kind != ConflictLineKind::Start {
            i += 1;
            continue;
        }
        let block_start = i;
        let mut block_end = i + 1;
        while block_end < lines.len() && lines[block_end].conflict_kind != ConflictLineKind::Start {
            let kind = lines[block_end].conflict_kind;
            block_end += 1;
            if kind == ConflictLineKind::End {
                break;
            }
        }
        if keep[block_start..block_end].iter().any(|&k| k) {
            for slot in &mut keep[block_start..block_end] {
                *slot = true;
            }
        }
        i = block_end;
    }
}

pub(super) fn separator_line(hidden_count: usize) -> DiffLine {
    DiffLine {
        old_line_no: None,
        new_line_no: None,
        style: DiffSpanStyle::Separator,
        spans: vec![DiffSpan {
            text: format!("{hidden_count} unmodified lines"),
            style: DiffSpanStyle::Separator,
            token: SyntaxToken::Plain,
        }],
        conflict_kind: ConflictLineKind::None,
        no_eof_newline: false,
    }
}
