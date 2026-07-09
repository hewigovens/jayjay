use jayjay_core::diff::DiffSpanStyle;
use jayjay_review::{NoteSide, NoteStatus};

use super::super::{DiffRenderRow, NoteDotKind, build_diff_render_rows};
use super::fixtures::{display_lines, line, note, wrapped};

#[test]
fn no_notes_yields_one_line_row_per_wrapped_fragment() {
    let display = display_lines();
    let wrapped = wrapped(&display);
    let rendered = build_diff_render_rows(&wrapped, &display, &[], 80);
    assert_eq!(
        rendered.rows,
        vec![DiffRenderRow::Line(0), DiffRenderRow::Line(1)]
    );
    assert!(rendered.dots.is_empty());
}

#[test]
fn current_note_on_added_line_gets_a_row_and_an_active_dot() {
    let display = display_lines();
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
    assert_eq!(
        rendered.rows,
        vec![
            DiffRenderRow::Line(0),
            DiffRenderRow::Line(1),
            DiffRenderRow::NoteText {
                note_id: "n1".into(),
                text: "check this".into(),
                is_first: true,
                is_last: true,
            },
        ]
    );
    assert_eq!(rendered.dots.get(&1), Some(&NoteDotKind::Active));
}

#[test]
fn note_on_removed_line_matches_old_side_not_new() {
    let display = vec![
        line(DiffSpanStyle::Removed, Some(1), None, "removed line"),
        line(DiffSpanStyle::Context, Some(2), Some(1), "unchanged"),
    ];
    let wrapped = wrapped(&display);
    let notes = vec![note(
        "n1",
        NoteSide::Old,
        1,
        "old side note",
        NoteStatus::Current,
        false,
    )];
    let rendered = build_diff_render_rows(&wrapped, &display, &notes, 80);
    assert!(matches!(rendered.rows[1], DiffRenderRow::NoteText { .. }));
    assert_eq!(rendered.rows[2], DiffRenderRow::Line(1));
    assert_eq!(rendered.dots.get(&0), Some(&NoteDotKind::Active));
}

#[test]
fn resolved_note_gets_a_dimmed_dot_and_no_row() {
    let display = display_lines();
    let wrapped = wrapped(&display);
    let notes = vec![note(
        "n1",
        NoteSide::New,
        2,
        "resolved already",
        NoteStatus::Resolved,
        true,
    )];
    let rendered = build_diff_render_rows(&wrapped, &display, &notes, 80);
    assert_eq!(
        rendered.rows,
        vec![DiffRenderRow::Line(0), DiffRenderRow::Line(1)],
        "a resolved note must not add a NoteText row"
    );
    assert_eq!(rendered.dots.get(&1), Some(&NoteDotKind::Resolved));
}

#[test]
fn stale_or_orphaned_note_gets_no_dot_and_no_row() {
    let display = display_lines();
    let wrapped = wrapped(&display);
    for status in [NoteStatus::Stale, NoteStatus::Orphaned] {
        let notes = vec![note("n1", NoteSide::New, 2, "stale", status, false)];
        let rendered = build_diff_render_rows(&wrapped, &display, &notes, 80);
        assert_eq!(
            rendered.rows,
            vec![DiffRenderRow::Line(0), DiffRenderRow::Line(1)],
            "{status:?} notes must not render an in-diff marker"
        );
        assert!(
            rendered.dots.is_empty(),
            "{status:?} notes must not get a dot"
        );
    }
}
