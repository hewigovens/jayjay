use super::*;

#[test]
fn counts_match_rendered_diff_rows() {
    // An EOF-newline change renders the last line as a removed/added pair; stats must agree with those rows.
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

#[test]
fn appending_to_a_terminated_file_adds_rows_only() {
    for ignore_whitespace in [false, true] {
        let rendered = compute_file_diff_full(
            "t.txt",
            "one\ntwo\n",
            "one\ntwo\nthree\nfour\n",
            ignore_whitespace,
        );
        assert_eq!(
            rendered.lines.iter().map(|l| l.style).collect::<Vec<_>>(),
            [
                DiffSpanStyle::Context,
                DiffSpanStyle::Context,
                DiffSpanStyle::Added,
                DiffSpanStyle::Added,
            ]
        );
        assert_eq!(
            count_changed_lines("one\ntwo\n", "one\ntwo\nthree\nfour\n", ignore_whitespace),
            (2, 0)
        );
    }
}
