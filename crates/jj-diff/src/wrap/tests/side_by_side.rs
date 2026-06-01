use crate::side_by_side::{RowSide, SideBySideRow, build_side_by_side_rows};
use crate::types::{ConflictLineKind, DiffSpanStyle};

use super::super::{
    sbs_line_to_row, visual_index_for_line, visual_index_for_sbs_row, wrap_diff_lines,
    wrap_sbs_rows,
};
use super::fixtures::{conflict_line, diff_line, row_side, span};

#[test]
fn wrap_sbs_rows_pads_to_tallest_side_and_blanks_continuation_line_no() {
    let row = SideBySideRow {
        old: row_side("10", "abcdefgh", DiffSpanStyle::Removed),
        new: row_side("10", "wxyz", DiffSpanStyle::Added),
        full_width: false,
    };
    let wrapped = wrap_sbs_rows(&[row], 3, 3);

    assert_eq!(wrapped.len(), 3);
    let old_texts: Vec<String> = wrapped.iter().map(|w| w.row.old.text()).collect();
    let new_texts: Vec<String> = wrapped.iter().map(|w| w.row.new.text()).collect();
    assert_eq!(old_texts, vec!["abc", "def", "gh"]);
    assert_eq!(new_texts, vec!["wxy", "z", ""]);
    assert_eq!(wrapped[0].row.old.line_no, "10");
    assert_eq!(wrapped[1].row.old.line_no, "");
}

#[test]
fn visual_index_finds_first_wrapped_position_for_unified_and_sbs() {
    // Unified: a long Added line in the middle inflates the visual count.
    let lines = vec![
        diff_line("foo", Some(1), Some(1), DiffSpanStyle::Context),
        diff_line("abcdefgh", Some(2), Some(2), DiffSpanStyle::Added),
        diff_line("bar", Some(3), Some(3), DiffSpanStyle::Context),
    ];
    let unified = wrap_diff_lines(&lines, 3);
    assert_eq!(visual_index_for_line(&unified, 0), 0);
    assert_eq!(visual_index_for_line(&unified, 1), 1);
    assert_eq!(visual_index_for_line(&unified, 2), 4);

    // SBS: same shape across the row pairing.
    let rows = vec![
        SideBySideRow {
            old: row_side("1", "abcdefgh", DiffSpanStyle::Removed),
            new: row_side("1", "wxyz", DiffSpanStyle::Added),
            full_width: false,
        },
        SideBySideRow {
            old: row_side("2", "ok", DiffSpanStyle::Context),
            new: row_side("2", "ok", DiffSpanStyle::Context),
            full_width: false,
        },
    ];
    let sbs = wrap_sbs_rows(&rows, 3, 3);
    assert_eq!(visual_index_for_sbs_row(&sbs, 0), 0);
    assert_eq!(visual_index_for_sbs_row(&sbs, 1), 3);
    // Out-of-range falls back to the requested ix.
    assert_eq!(visual_index_for_sbs_row(&sbs, 99), 99);
}

#[test]
fn wrap_sbs_rows_splits_full_width_conflicts_across_both_panes() {
    let row = SideBySideRow {
        old: RowSide::default(),
        new: RowSide {
            line_no: "4".to_owned(),
            spans: vec![span("abcdefghijkl", DiffSpanStyle::Added)],
            style: DiffSpanStyle::Added,
            conflict_kind: ConflictLineKind::Content,
        },
        full_width: true,
    };

    let wrapped = wrap_sbs_rows(&[row], 4, 4);

    assert_eq!(wrapped.len(), 2);
    assert_eq!(wrapped[0].row.old.text(), "abcd");
    assert_eq!(wrapped[0].row.new.text(), "efgh");
    assert_eq!(wrapped[1].row.old.text(), "ijkl");
    assert_eq!(wrapped[1].row.new.text(), "");
    assert!(wrapped.iter().all(|row| row.row.full_width));
    assert!(
        wrapped
            .iter()
            .all(|row| row.row.old.conflict_kind == ConflictLineKind::Content)
    );
    assert!(
        wrapped
            .iter()
            .all(|row| row.row.new.conflict_kind == ConflictLineKind::Content)
    );
}

#[test]
fn sbs_line_to_row_maps_all_styles() {
    // Context bracketing a 3-removed / 2-added block, plus separator and trailing context.
    let lines = vec![
        diff_line("ctx", Some(1), Some(1), DiffSpanStyle::Context),
        diff_line("r1", Some(2), None, DiffSpanStyle::Removed),
        diff_line("r2", Some(3), None, DiffSpanStyle::Removed),
        diff_line("r3", Some(4), None, DiffSpanStyle::Removed),
        diff_line("a1", None, Some(2), DiffSpanStyle::Added),
        diff_line("a2", None, Some(3), DiffSpanStyle::Added),
        diff_line("ctx2", Some(5), Some(4), DiffSpanStyle::Context),
        diff_line("sep", None, None, DiffSpanStyle::Separator),
        diff_line("a3", None, Some(5), DiffSpanStyle::Added),
    ];
    let map = sbs_line_to_row(&lines);
    // ctx -> row 0; r1/r2/r3 -> rows 1/2/3; a1/a2 pair into rows 1/2; ctx2 -> row 4;
    // separator -> row 5; trailing a3 -> row 6.
    assert_eq!(map, vec![0, 1, 2, 3, 1, 2, 4, 5, 6]);
}

#[test]
fn sbs_line_to_row_keeps_conflict_blocks_unpaired() {
    let lines = vec![
        conflict_line(
            "<<<<<<< conflict 1 of 1",
            DiffSpanStyle::Added,
            ConflictLineKind::Start,
        ),
        conflict_line("-old", DiffSpanStyle::Removed, ConflictLineKind::Removed),
        conflict_line("+new", DiffSpanStyle::Added, ConflictLineKind::Added),
        conflict_line(
            ">>>>>>> conflict 1 of 1 ends",
            DiffSpanStyle::Added,
            ConflictLineKind::End,
        ),
    ];

    let rows = build_side_by_side_rows(&lines);
    let map = sbs_line_to_row(&lines);

    assert_eq!(rows.len(), 3);
    assert!(rows.iter().all(|row| row.full_width));
    assert_eq!(map, vec![0, 1, 2]);
    assert_eq!(rows[0].new.text(), "Conflict 1 of 1");
    assert_eq!(rows[1].new.text(), "-old");
    assert_eq!(rows[2].new.text(), "+new");
}
