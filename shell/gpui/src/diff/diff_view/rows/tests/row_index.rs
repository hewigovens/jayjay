use super::super::{DiffRenderRow, row_index_for_line};

#[test]
fn row_index_for_line_maps_through_preceding_note_rows() {
    let rows = vec![
        DiffRenderRow::Line(0),
        DiffRenderRow::Line(1),
        DiffRenderRow::NoteText {
            note_id: "n1".into(),
            text: "note".into(),
            is_first: true,
            is_last: true,
        },
        DiffRenderRow::Line(2),
    ];
    assert_eq!(row_index_for_line(&rows, 0), 0);
    assert_eq!(row_index_for_line(&rows, 1), 1);
    assert_eq!(
        row_index_for_line(&rows, 2),
        3,
        "the note row above shifts line 2's row index by one"
    );
}

#[test]
fn row_index_for_line_falls_back_to_the_line_index_when_absent() {
    let rows = vec![DiffRenderRow::Line(0)];
    assert_eq!(row_index_for_line(&rows, 5), 5);
}
