use super::*;

#[test]
fn eof_only_change_counts_the_split_context_pair() {
    assert_eq!(count_changed_lines("a\nb", "a\nb\n", false), (1, 1));
    assert_eq!(count_changed_lines("a\nb\n", "a\nb", false), (1, 1));
}

#[test]
fn whitespace_only_changes_follow_the_active_mode() {
    assert_eq!(count_changed_lines("a b\n", "a  b\n", false), (1, 1));
    assert_eq!(count_changed_lines("a b\n", "a  b\n", true), (0, 0));
}

#[test]
fn counts_match_rendered_diff_rows() {
    // The renderer re-pairs the old last line on appends and splits it on EOF-only changes; stats must agree with those rows, not with `jj diff --stat`.
    let cases = [
        ("", ""),
        ("", "one\ntwo\n"),
        ("one\ntwo", ""),
        ("a\nb\nc\n", "a\nB\nc\n"),
        ("a\nb\nc\n", "a\nB\nc\nd\n"),
        ("a\nb", "a\nb\n"),
        ("a\nb\n", "a\nb"),
        ("a\n", "a"),
        ("x\na\nb", "X\na\nb\n"),
        ("a\nb", "a\nb\nc"),
        ("a b\nc\n", "a  b\nc\nd\n"),
    ];
    for ignore_whitespace in [false, true] {
        for (old, new) in cases {
            let rendered = compute_file_diff_full("t.txt", old, new, ignore_whitespace);
            let added = rendered
                .lines
                .iter()
                .filter(|l| l.style == DiffSpanStyle::Added)
                .count() as u32;
            let removed = rendered
                .lines
                .iter()
                .filter(|l| l.style == DiffSpanStyle::Removed)
                .count() as u32;
            assert_eq!(
                count_changed_lines(old, new, ignore_whitespace),
                (added, removed),
                "case {old:?} -> {new:?} (ignore_whitespace: {ignore_whitespace})"
            );
        }
    }
}
