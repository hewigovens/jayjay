use std::collections::HashMap;

use gpui::SharedString;
use jayjay_core::diff::{
    DiffLine, DiffSide, WrappedDiffLine, anchor_side_and_number, change_groups,
};
use jayjay_review::{NoteSide, NoteStatus, ReviewNoteStatus};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use super::diff_render_rows::{DiffRenderRow, DiffRenderRows, NoteDotKind};

/// `notes` must already be filtered to this file's path + identity.
pub fn build_diff_render_rows(
    wrapped: &[WrappedDiffLine],
    display_lines: &[DiffLine],
    notes: &[ReviewNoteStatus],
    cols: u32,
) -> DiffRenderRows {
    let by_line = notes_by_display_line(display_lines, notes);
    let mut rows = Vec::with_capacity(wrapped.len());
    let mut dots = HashMap::new();
    let mut note_indents = HashMap::new();

    for (ix, fragment) in wrapped.iter().enumerate() {
        rows.push(DiffRenderRow::Line(ix));
        // Only the wrap head anchors notes; continuation fragments carry no line number.
        if fragment.col_start != 0 {
            continue;
        }
        let Some(matches) = by_line.get(&(fragment.line_ix as usize)) else {
            continue;
        };
        if matches.iter().any(|m| m.status == NoteStatus::Current) {
            dots.insert(ix, NoteDotKind::Active);
        } else if matches.iter().any(|m| m.status == NoteStatus::Resolved) {
            dots.insert(ix, NoteDotKind::Resolved);
        }
        let indent_cols = display_lines
            .get(fragment.line_ix as usize)
            .map(leading_indent_cols)
            .unwrap_or(0);
        for note in matches
            .iter()
            .filter(|m| m.status == NoteStatus::Current && !m.note.resolved)
        {
            push_note_rows(&mut rows, &mut note_indents, note, cols, indent_cols);
        }
    }

    DiffRenderRows {
        rows,
        dots,
        note_indents,
    }
}

fn leading_indent_cols(line: &DiffLine) -> u32 {
    line.text()
        .chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .count() as u32
}

fn push_note_rows(
    rows: &mut Vec<DiffRenderRow>,
    note_indents: &mut HashMap<usize, u32>,
    note: &ReviewNoteStatus,
    cols: u32,
    indent_cols: u32,
) {
    // Narrow by the indent or the note text would overflow the diff's wrap column.
    let wrap_cols = cols.saturating_sub(indent_cols).max(1);
    let body_lines = wrap_note_body(&note.note.body, wrap_cols);
    let last_ix = body_lines.len().saturating_sub(1);
    let note_id: SharedString = note.note.id.clone().into();
    for (ix, text) in body_lines.into_iter().enumerate() {
        note_indents.insert(rows.len(), indent_cols);
        rows.push(DiffRenderRow::NoteText {
            note_id: note_id.clone(),
            text: text.into(),
            is_first: ix == 0,
            is_last: ix == last_ix,
        });
    }
}

/// A note's `(side, line)` is a file line number, not a display index.
fn notes_by_display_line<'a>(
    display_lines: &[DiffLine],
    notes: &'a [ReviewNoteStatus],
) -> HashMap<usize, Vec<&'a ReviewNoteStatus>> {
    if notes.is_empty() {
        return HashMap::new();
    }
    let mut by_anchor: HashMap<(DiffSide, u32), Vec<&ReviewNoteStatus>> = HashMap::new();
    for note in notes {
        let side = match note.note.side {
            NoteSide::Old => DiffSide::Old,
            NoteSide::New => DiffSide::New,
        };
        by_anchor
            .entry((side, note.note.line))
            .or_default()
            .push(note);
    }

    let mut result: HashMap<usize, Vec<&ReviewNoteStatus>> = HashMap::new();
    for group in change_groups(display_lines) {
        for line_number in group.start_line..=group.end_line {
            let ix = (line_number - 1) as usize;
            let Some(line) = display_lines.get(ix) else {
                continue;
            };
            let Some(key) = anchor_side_and_number(line) else {
                continue;
            };
            if let Some(matches) = by_anchor.get(&key) {
                result.insert(ix, matches.clone());
            }
        }
    }
    result
}

/// Pre-wraps at the diff's `cols` in display cells: `uniform_list` rows are fixed-height and cannot reflow.
fn wrap_note_body(body: &str, cols: u32) -> Vec<String> {
    let cols = cols.max(1) as usize;
    let mut out = Vec::new();
    for paragraph in body.split('\n') {
        let mut line = String::new();
        let mut width = 0;
        for word in paragraph.split_whitespace() {
            let word_width = word.width();
            let gap = usize::from(width > 0);
            if width + gap + word_width <= cols {
                if gap == 1 {
                    line.push(' ');
                }
                line.push_str(word);
                width += gap + word_width;
                continue;
            }
            if width > 0 {
                out.push(std::mem::take(&mut line));
                width = 0;
            }
            // A word wider than the note, such as a CJK sentence without spaces, breaks between graphemes.
            for grapheme in word.graphemes(true) {
                let grapheme_width = grapheme.width();
                if width > 0 && width + grapheme_width > cols {
                    out.push(std::mem::take(&mut line));
                    width = 0;
                }
                line.push_str(grapheme);
                width += grapheme_width;
            }
        }
        out.push(line);
    }
    if out.is_empty() {
        out.push(String::new());
    }
    out
}
