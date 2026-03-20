use jj_lib::diff::{ContentDiff, DiffHunkKind};

use crate::syntax::HighlightSpan;

use super::highlights::apply_highlights;
use super::types::{DiffSpan, DiffSpanStyle};

/// Produce word-level diff spans for a paired removed+added line.
///
/// Returns `(removed_spans, added_spans)` where each span has:
/// - `DiffSpanStyle::Removed` / `DiffSpanStyle::Added` for words that actually changed (highlighted)
/// - `DiffSpanStyle::Unchanged` for matching text within the changed line (no word highlight)
///
/// Syntax tokens are overlaid on top of the word-level diff styles.
pub(super) fn word_diff_paired_line(
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
        DiffSpanStyle::Removed,
    );
    let added_spans = apply_highlights_with_word_diff(
        new_line,
        new_byte_offset,
        new_highlights,
        &new_word_styles,
        DiffSpanStyle::Added,
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
/// (Added/Removed), text that matched gets `DiffSpanStyle::Unchanged`.
fn apply_highlights_with_word_diff(
    line: &str,
    byte_offset: usize,
    highlights: &[HighlightSpan],
    word_changed: &[bool],
    changed_style: DiffSpanStyle,
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
                DiffSpanStyle::Unchanged
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
