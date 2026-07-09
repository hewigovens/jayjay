use jayjay_core::diff::DiffSpanStyle;
use jayjay_review::{NoteSide, NoteStatus};

use super::super::{DiffRenderRow, build_diff_render_rows};
use super::fixtures::{display_lines, line, note, wrapped};

#[test]
fn note_row_indent_matches_anchor_lines_leading_whitespace() {
    let display = vec![
        line(DiffSpanStyle::Context, Some(1), Some(1), "unchanged"),
        line(DiffSpanStyle::Added, None, Some(2), "    indented line"),
    ];
    let wrapped = wrapped(&display);
    let notes = vec![note(
        "n1",
        NoteSide::New,
        2,
        "check this",
        NoteStatus::Current,
        false,
    )];
    let rendered = build_diff_render_rows(&wrapped, &display, &notes, 80);
    assert!(matches!(rendered.rows[2], DiffRenderRow::NoteText { .. }));
    assert_eq!(rendered.note_indents.get(&2), Some(&4));
}

#[test]
fn long_note_body_wraps_at_cols_with_first_and_last_flags() {
    let display = display_lines();
    let wrapped = wrapped(&display);
    let notes = vec![note(
        "n1",
        NoteSide::New,
        2,
        "one two three four five",
        NoteStatus::Current,
        false,
    )];
    let rendered = build_diff_render_rows(&wrapped, &display, &notes, 11);
    let note_rows: Vec<&DiffRenderRow> = rendered.rows[2..].iter().collect();
    assert_eq!(
        note_rows.len(),
        3,
        "11 cols wraps five short words into three lines"
    );
    assert!(matches!(
        note_rows[0],
        DiffRenderRow::NoteText {
            is_first: true,
            is_last: false,
            ..
        }
    ));
    assert!(matches!(
        note_rows[1],
        DiffRenderRow::NoteText {
            is_first: false,
            is_last: false,
            ..
        }
    ));
    assert!(matches!(
        note_rows[2],
        DiffRenderRow::NoteText {
            is_first: false,
            is_last: true,
            ..
        }
    ));
}

#[test]
fn note_wrap_narrows_by_the_anchor_lines_indent() {
    let display = vec![
        line(DiffSpanStyle::Context, Some(1), Some(1), "unchanged"),
        line(DiffSpanStyle::Added, None, Some(2), "    indented line"),
    ];
    let wrapped = wrapped(&display);
    let notes = vec![note(
        "n1",
        NoteSide::New,
        2,
        "one two three four five",
        NoteStatus::Current,
        false,
    )];
    // 15 cols minus the 4-col indent leaves 11 usable cols — the same wrap boundary the un-indented case above hits directly.
    let rendered = build_diff_render_rows(&wrapped, &display, &notes, 15);
    let note_rows: Vec<&DiffRenderRow> = rendered.rows[2..].iter().collect();
    assert_eq!(
        note_rows.len(),
        3,
        "the indent should narrow the wrap width by 4 cols"
    );
}
