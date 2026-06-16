use similar::ChangeTag;

use crate::syntax::HighlightSpan;
use crate::text_diff_config;

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
    let (old_word_styles, new_word_styles) = word_diff_style_maps(old_line, new_line);

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

/// Per-byte changed-word maps `(old_changed, new_changed)` from one `similar`
/// pass (no jj-lib dependency).
fn word_diff_style_maps(old_line: &str, new_line: &str) -> (Vec<bool>, Vec<bool>) {
    let mut old_changed = vec![false; old_line.len()];
    let mut new_changed = vec![false; new_line.len()];

    let diff = text_diff_config().diff_words(old_line, new_line);

    let mut old_pos = 0usize;
    let mut new_pos = 0usize;

    for change in diff.iter_all_changes() {
        let len = change.value().len();
        match change.tag() {
            ChangeTag::Equal => {
                old_pos += len;
                new_pos += len;
            }
            ChangeTag::Delete => {
                mark_changed(&mut old_changed, old_pos, len);
                old_pos += len;
            }
            ChangeTag::Insert => {
                mark_changed(&mut new_changed, new_pos, len);
                new_pos += len;
            }
        }
    }

    (old_changed, new_changed)
}

fn mark_changed(changed: &mut [bool], start: usize, len: usize) {
    let end = (start + len).min(changed.len());
    if start < end {
        changed[start..end].fill(true);
    }
}

/// Apply syntax highlights combined with word-level diff style information.
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

    let base_spans = apply_highlights(line, byte_offset, highlights, changed_style);

    let mut result = Vec::new();
    let mut line_pos = 0usize;

    for span in &base_spans {
        let span_len = span.text.len();
        let span_start = line_pos;
        let span_end = line_pos + span_len;

        let mut pos = span_start;
        while pos < span_end {
            let is_changed = word_changed.get(pos).copied().unwrap_or(false);
            let style = if is_changed {
                changed_style
            } else {
                DiffSpanStyle::Unchanged
            };

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
