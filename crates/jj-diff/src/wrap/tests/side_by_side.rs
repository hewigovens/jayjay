use crate::side_by_side::{RowSide, SideBySideRow, build_side_by_side_rows};
use crate::types::{ConflictLineKind, DiffSpanStyle};

use super::super::{
    WrappedSide, sbs_line_to_row, visual_index_for_line, visual_index_for_sbs_row, wrap_diff_lines,
    wrap_sbs_rows,
};
use super::fixtures::{conflict_line, diff_line, row_side, span};

#[test]
fn wrap_sbs_rows_pads_to_tallest_side_and_blanks_continuation_line_no() {
    let row = SideBySideRow {
        old: row_side("10", "abcdefgh", DiffSpanStyle::Removed),
        new: row_side("10", "wxyz", DiffSpanStyle::Added),
        full_width: false,
        context_region: None,
    };
    let separator = SideBySideRow {
        old: row_side("", "12 lines hidden", DiffSpanStyle::Separator),
        new: row_side("", "12 lines hidden", DiffSpanStyle::Separator),
        full_width: false,
        context_region: None,
    };
    let wrapped = wrap_sbs_rows(&[row, separator], 3, 3);

    assert_eq!(wrapped.len(), 4);
    let old_texts: Vec<String> = wrapped.iter().map(|w| w.row.old.text()).collect();
    let new_texts: Vec<String> = wrapped.iter().map(|w| w.row.new.text()).collect();
    assert_eq!(old_texts, vec!["abc", "def", "gh", "12 lines hidden"]);
    assert_eq!(new_texts, vec!["wxy", "z", "", "12 lines hidden"]);
    assert_eq!(wrapped[0].row.old.line_no, "10");
    assert_eq!(wrapped[1].row.old.line_no, "");
    assert_eq!(
        wrapped
            .iter()
            .map(|w| (w.row_ix, side_cols(&w.old), side_cols(&w.new)))
            .collect::<Vec<_>>(),
        vec![
            (0, (8, 0, 3), (4, 0, 3)),
            (0, (8, 3, 6), (4, 3, 4)),
            (0, (8, 6, 8), (4, 0, 0)),
            (1, (15, 0, 15), (15, 0, 15)),
        ]
    );
}

fn side_cols(side: &WrappedSide) -> (u32, u32, u32) {
    (side.line_len, side.col_start, side.col_end)
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
            context_region: None,
        },
        SideBySideRow {
            old: row_side("2", "ok", DiffSpanStyle::Context),
            new: row_side("2", "ok", DiffSpanStyle::Context),
            full_width: false,
            context_region: None,
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
        context_region: None,
    };

    let wrapped = wrap_sbs_rows(&[row], 4, 4);

    assert_eq!(wrapped.len(), 2);
    assert_eq!(wrapped[0].row.old.text(), "abcd");
    assert_eq!(wrapped[0].row.new.text(), "efgh");
    assert_eq!(wrapped[1].row.old.text(), "ijkl");
    assert_eq!(wrapped[1].row.new.text(), "");
    assert_eq!(
        wrapped
            .iter()
            .map(|w| (side_cols(&w.old), side_cols(&w.new)))
            .collect::<Vec<_>>(),
        vec![((12, 0, 4), (12, 4, 8)), ((12, 8, 12), (12, 12, 12))]
    );
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
fn sbs_line_to_row_keeps_conflict_blocks_unpaired() {
    let lines = vec![
        diff_line("ctx", Some(3), Some(3), DiffSpanStyle::Context),
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

    assert_eq!(
        rows.iter()
            .map(|row| (
                row.full_width,
                row.old.line_no.as_str(),
                row.new.line_no.as_str(),
                row.new.text()
            ))
            .collect::<Vec<_>>(),
        vec![
            (false, "3", "3", "ctx".to_owned()),
            (true, "", "", "Conflict 1 of 1".to_owned()),
            (true, "", "1", "-old".to_owned()),
            (true, "", "1", "+new".to_owned()),
        ]
    );
    assert_eq!(map, vec![0, 1, 2, 3]);
}
