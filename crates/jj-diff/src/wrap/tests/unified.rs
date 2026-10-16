use crate::types::{DiffSpanStyle, MovedRow};

use super::super::wrap_diff_lines;
use super::fixtures::diff_line;

#[test]
fn wrap_diff_lines_emits_continuation_rows_without_line_numbers() {
    let line = diff_line("abcdefghij", Some(12), Some(14), DiffSpanStyle::Added);
    let wrapped = wrap_diff_lines(&[line], 4, 0);

    assert_eq!(wrapped.len(), 3);
    let texts: Vec<String> = wrapped.iter().map(|w| w.line.text()).collect();
    assert_eq!(texts, vec!["abcd", "efgh", "ij"]);
    assert_eq!(wrapped[0].line.new_line_no, Some(14));
    assert_eq!(wrapped[1].line.new_line_no, None);
    assert_eq!((wrapped[2].col_start, wrapped[2].col_end), (8, 10));
}

#[test]
fn moved_block_first_rows_leave_room_for_their_jump_label() {
    let moved = |line_no, len, starts_block| {
        let mut line = diff_line(
            &"a".repeat(len),
            Some(line_no),
            None,
            DiffSpanStyle::Removed,
        );
        line.moved = Some(MovedRow {
            counterpart_line: 9,
            starts_block,
        });
        line
    };
    let rows: Vec<(u32, u32)> = wrap_diff_lines(&[moved(1, 45, true), moved(2, 25, false)], 30, 10)
        .iter()
        .map(|w| (w.line_ix, w.col_end - w.col_start))
        .collect();
    assert_eq!(rows, [(0, 20), (0, 25), (1, 25)]);
}
