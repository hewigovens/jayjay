use super::*;

#[test]
fn eof_newline_changes_synthesize_hunks_marked_on_the_side_without_a_newline() {
    use DiffSpanStyle::{Added, Context, Removed};
    let cases = [
        (
            "a\nb",
            "a\nb\n",
            vec![(Context, false), (Removed, true), (Added, false)],
        ),
        (
            "a\nb\n",
            "a\nb",
            vec![(Context, false), (Removed, false), (Added, true)],
        ),
        (
            "a",
            "a\nb\n",
            vec![(Removed, true), (Added, false), (Added, false)],
        ),
        (
            "a\nold",
            "a\nnew\n",
            vec![(Context, false), (Removed, true), (Added, false)],
        ),
        (
            "a\nchanged\nc",
            "a\nfixed\nc\n",
            vec![
                (Context, false),
                (Removed, false),
                (Added, false),
                (Removed, true),
                (Added, false),
            ],
        ),
    ];
    for (old, new, expected) in cases {
        let diff = compute_file_diff("test.txt", old, new, false);
        assert_eq!(
            diff.lines
                .iter()
                .map(|l| (l.style, l.no_eof_newline))
                .collect::<Vec<_>>(),
            expected,
            "{old:?} -> {new:?}"
        );
        assert!(!diff.whitespace_only_hidden, "{old:?} -> {new:?}");
    }
}
