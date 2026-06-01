use super::*;

#[test]
fn prefix_suffix_trimming_correctness() {
    let diff = compute_file_diff_full(
        "test.txt",
        "a\nb\nc\nd\ne\nf\ng\n",
        "a\nb\nX\nd\ne\nf\ng\n",
        false,
    );
    let styles: Vec<_> = diff.lines.iter().map(|l| l.style).collect();
    assert_eq!(
        styles,
        vec![
            DiffSpanStyle::Context,
            DiffSpanStyle::Context,
            DiffSpanStyle::Removed,
            DiffSpanStyle::Added,
            DiffSpanStyle::Context,
            DiffSpanStyle::Context,
            DiffSpanStyle::Context,
            DiffSpanStyle::Context,
        ]
    );
}
