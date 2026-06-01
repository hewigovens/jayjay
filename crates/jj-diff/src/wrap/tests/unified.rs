use crate::types::DiffSpanStyle;

use super::super::wrap_diff_lines;
use super::fixtures::diff_line;

#[test]
fn wrap_diff_lines_emits_continuation_rows_without_line_numbers() {
    let line = diff_line("abcdefghij", Some(12), Some(14), DiffSpanStyle::Added);
    let wrapped = wrap_diff_lines(&[line], 4);

    assert_eq!(wrapped.len(), 3);
    let texts: Vec<String> = wrapped.iter().map(|w| w.line.text()).collect();
    assert_eq!(texts, vec!["abcd", "efgh", "ij"]);
    assert_eq!(wrapped[0].line.new_line_no, Some(14));
    assert_eq!(wrapped[1].line.new_line_no, None);
    assert_eq!((wrapped[2].col_start, wrapped[2].col_end), (8, 10));
}
