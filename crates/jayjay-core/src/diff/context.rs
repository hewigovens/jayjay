use crate::syntax::SyntaxToken;

use super::types::{CONTEXT_LINES, DiffLine, DiffSpan, DiffSpanStyle};

/// Collapse long runs of context lines, keeping only CONTEXT_LINES around changes.
pub(super) fn collapse_context(lines: Vec<DiffLine>) -> Vec<DiffLine> {
    if lines.is_empty() {
        return lines;
    }

    // Find indices of all changed (non-context) lines
    let changed_indices: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.style != DiffSpanStyle::Context)
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
        for slot in &mut keep[start..end] {
            *slot = true;
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

pub(super) fn separator_line(hidden_count: usize) -> DiffLine {
    DiffLine {
        old_line_no: None,
        new_line_no: None,
        style: DiffSpanStyle::Separator,
        spans: vec![DiffSpan {
            text: format!("{hidden_count} hidden lines"),
            style: DiffSpanStyle::Separator,
            token: SyntaxToken::Plain,
        }],
    }
}
