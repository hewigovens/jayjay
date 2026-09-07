use super::*;

#[test]
fn word_diff_marks_only_the_minimal_changed_run_of_paired_lines() {
    let cases = [
        ("hello world\n", "hello earth\n", "world", "earth"),
        ("aaa\n", "zzz\n", "aaa", "zzz"),
        ("old_func(x)\n", "new_func(x)\n", "old", "new"),
        ("version = \"0.3.5\"\n", "version = \"0.3.6\"\n", "5", "6"),
        (
            "let key = \"prefix-old-suffix\"\n",
            "let key = \"prefix-new-suffix\"\n",
            "old",
            "new",
        ),
        (
            "- diffs with image/SVG previews.\n",
            "- diffs with image/SVG/Markdown/HTML previews.\n",
            "",
            "/Markdown/HTML",
        ),
        (
            "the quick brown fox\n",
            "the slow brown cat\n",
            "quickfox",
            "slowcat",
        ),
        ("a  b\n", "a b\n", " ", ""),
        ("a\n\nc\n", "a\nhello\nc\n", "", "hello"),
        ("version 1.0\n", "version 1.00\n", "", "0"),
        ("naïve\n", "naïveté\n", "", "té"),
    ];
    for (old, new, removed, added) in cases {
        let diff = compute_file_diff("test.txt", old, new, false);
        let rem = diff
            .lines
            .iter()
            .find(|l| l.style == DiffSpanStyle::Removed)
            .unwrap_or_else(|| panic!("removed line for {old:?} -> {new:?}"));
        let add = diff
            .lines
            .iter()
            .find(|l| l.style == DiffSpanStyle::Added)
            .unwrap_or_else(|| panic!("added line for {old:?} -> {new:?}"));
        assert_eq!(
            styled_text(&span_info(rem), DiffSpanStyle::Removed),
            removed,
            "{old:?} -> {new:?}: {:?}",
            span_info(rem)
        );
        assert_eq!(
            styled_text(&span_info(add), DiffSpanStyle::Added),
            added,
            "{old:?} -> {new:?}: {:?}",
            span_info(add)
        );
        for line in [rem, add] {
            assert!(
                line.spans
                    .iter()
                    .all(|s| s.style == line.style || s.style == DiffSpanStyle::Unchanged),
                "{old:?} -> {new:?}: {:?}",
                span_info(line)
            );
        }
        assert_eq!(rem.text(), source_line(old, rem.old_line_no));
        assert_eq!(add.text(), source_line(new, add.new_line_no));
    }
}

#[test]
fn unpaired_and_context_lines_carry_no_word_highlight() {
    let diff = compute_file_diff("test.txt", "ctx\naaa\nbbb\nccc\n", "ctx\nAAA\nccc\n", false);
    assert_eq!(diff.lines[0].style, DiffSpanStyle::Context);
    assert!(
        diff.lines[0]
            .spans
            .iter()
            .all(|s| s.style == DiffSpanStyle::Context)
    );
    let unpaired = diff
        .lines
        .iter()
        .find(|l| l.style == DiffSpanStyle::Removed && l.text() == "bbb")
        .expect("unpaired removed line 'bbb'");
    assert!(
        unpaired
            .spans
            .iter()
            .all(|s| s.style == DiffSpanStyle::Unchanged),
        "{:?}",
        span_info(unpaired)
    );
}

fn source_line(text: &str, line_no: Option<u32>) -> &str {
    text.lines()
        .nth(line_no.unwrap() as usize - 1)
        .unwrap()
        .trim_end()
}

fn styled_text(spans: &[(&str, DiffSpanStyle)], style: DiffSpanStyle) -> String {
    spans
        .iter()
        .filter_map(|(text, span_style)| (*span_style == style).then_some(*text))
        .collect()
}
