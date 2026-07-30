use crate::syntax::SyntaxToken;

use super::types::{
    CONTEXT_LINES, CollapsedDiff, ConflictLineKind, ContextRegion, DiffLine, DiffSpan,
    DiffSpanStyle, DisplayLineMapping, FileDiff,
};

const COLLAPSED_CONTEXT_THRESHOLD: usize = 2;

pub(super) fn collapse_context(lines: Vec<DiffLine>) -> Vec<DiffLine> {
    let full = FileDiff {
        path: String::new(),
        language: String::new(),
        lines,
        whitespace_only_hidden: false,
    };
    collapse_context_with_mapping(&full).diff.lines
}

pub fn collapse_context_with_mapping(full_diff: &FileDiff) -> CollapsedDiff {
    let lines = &full_diff.lines;
    if lines.is_empty() {
        return CollapsedDiff {
            diff: full_diff.clone(),
            display_to_full: vec![],
        };
    }

    let changed_indices: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.is_changed())
        .map(|(i, _)| i)
        .collect();

    let mut keep = vec![false; lines.len()];
    if changed_indices.is_empty() {
        let leading_end = CONTEXT_LINES.min(lines.len());
        keep[..leading_end].fill(true);
        let trailing_start = lines.len().saturating_sub(CONTEXT_LINES);
        keep[trailing_start..].fill(true);
    } else {
        for idx in changed_indices.iter().copied() {
            let start = idx.saturating_sub(CONTEXT_LINES);
            let end = (idx + CONTEXT_LINES + 1).min(lines.len());
            for slot in &mut keep[start..end] {
                *slot = true;
            }
        }
    }

    // Conflict presentation requires a complete Start..=End block; keeping every block visible also guarantees later context expansion can never expose a partial committed conflict.
    keep_all_conflict_blocks(lines, &mut keep);

    if keep.iter().all(|&slot| slot) {
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
            let hidden = i - start;
            // Short runs stay inline, and a non-context run must not become an expandable separator.
            let region = (hidden > COLLAPSED_CONTEXT_THRESHOLD)
                .then(|| context_region(lines, start, i))
                .flatten();
            if let Some(region) = region {
                result.push(separator_line(region));
            } else {
                for (offset, line) in lines[start..i].iter().enumerate() {
                    result.push(line.clone());
                    mapping.push(DisplayLineMapping {
                        display_line: result.len() as u32,
                        full_line: (start + offset + 1) as u32,
                    });
                }
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

fn context_region(lines: &[DiffLine], start: usize, end: usize) -> Option<ContextRegion> {
    let first = lines.get(start)?;
    if lines[start..end].iter().any(|line| {
        line.style != DiffSpanStyle::Context
            || line.old_line_no.is_none()
            || line.new_line_no.is_none()
            || line.conflict_kind != ConflictLineKind::None
    }) {
        return None;
    }
    Some(ContextRegion {
        id: (start + 1) as u32,
        old_start_line: first.old_line_no?,
        new_start_line: first.new_line_no?,
        line_count: (end - start) as u32,
        initial_line_count: (end - start) as u32,
    })
}

fn keep_all_conflict_blocks(lines: &[DiffLine], keep: &mut [bool]) {
    let mut i = 0usize;
    while i < lines.len() {
        if lines[i].conflict_kind != ConflictLineKind::Start {
            i += 1;
            continue;
        }
        let block_start = i;
        let mut block_end = i + 1;
        while block_end < lines.len() {
            let kind = lines[block_end].conflict_kind;
            block_end += 1;
            if kind == ConflictLineKind::End {
                break;
            }
        }
        for slot in &mut keep[block_start..block_end] {
            *slot = true;
        }
        i = block_end;
    }
}

pub(super) fn separator_line(region: ContextRegion) -> DiffLine {
    DiffLine {
        old_line_no: None,
        new_line_no: None,
        style: DiffSpanStyle::Separator,
        spans: vec![DiffSpan {
            text: format!("{} unmodified lines", region.line_count),
            style: DiffSpanStyle::Separator,
            token: SyntaxToken::Plain,
        }],
        conflict_kind: ConflictLineKind::None,
        no_eof_newline: false,
        context_region: Some(region),
    }
}
