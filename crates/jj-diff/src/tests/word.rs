use super::*;

#[test]
fn word_diff_single_word_change() {
    let diff = compute_file_diff("test.txt", "hello world\n", "hello earth\n", false);
    let styles: Vec<_> = diff.lines.iter().map(|l| l.style).collect();
    assert_eq!(styles, vec![DiffSpanStyle::Removed, DiffSpanStyle::Added]);

    let rem = span_info(&diff.lines[0]);
    assert!(
        rem.iter()
            .any(|(t, s)| t.contains("hello") && *s == DiffSpanStyle::Unchanged),
        "matching text 'hello' should be Unchanged in removed line, got: {rem:?}"
    );
    assert!(
        rem.iter()
            .any(|(t, s)| t.contains("world") && *s == DiffSpanStyle::Removed),
        "changed text 'world' should be Removed in removed line, got: {rem:?}"
    );

    let add = span_info(&diff.lines[1]);
    assert!(
        add.iter()
            .any(|(t, s)| t.contains("hello") && *s == DiffSpanStyle::Unchanged),
        "matching text 'hello' should be Unchanged in added line, got: {add:?}"
    );
    assert!(
        add.iter()
            .any(|(t, s)| t.contains("earth") && *s == DiffSpanStyle::Added),
        "changed text 'earth' should be Added in added line, got: {add:?}"
    );
}

#[test]
fn word_diff_preserves_line_level_style() {
    let diff = compute_file_diff("test.txt", "foo bar\n", "foo baz\n", false);
    assert_eq!(diff.lines[0].style, DiffSpanStyle::Removed);
    assert_eq!(diff.lines[1].style, DiffSpanStyle::Added);
}

#[test]
fn word_diff_entirely_different_lines() {
    let diff = compute_file_diff("test.txt", "aaa\n", "zzz\n", false);
    let rem = span_info(&diff.lines[0]);
    assert!(
        rem.iter().all(|(_, s)| *s == DiffSpanStyle::Removed),
        "entirely different removed line should have all Removed spans, got: {rem:?}"
    );
    let add = span_info(&diff.lines[1]);
    assert!(
        add.iter().all(|(_, s)| *s == DiffSpanStyle::Added),
        "entirely different added line should have all Added spans, got: {add:?}"
    );
}

#[test]
fn word_diff_prefix_change() {
    let diff = compute_file_diff("test.txt", "old_func(x)\n", "new_func(x)\n", false);
    let rem = span_info(&diff.lines[0]);
    let add = span_info(&diff.lines[1]);
    assert_eq!(styled_text(&rem, DiffSpanStyle::Removed), "old");
    assert_eq!(styled_text(&add, DiffSpanStyle::Added), "new");
    assert!(
        rem.iter()
            .any(|(t, s)| t.contains("_func(x)") && *s == DiffSpanStyle::Unchanged),
        "shared suffix should stay unhighlighted, got: {rem:?}"
    );
    assert!(
        add.iter()
            .any(|(t, s)| t.contains("_func(x)") && *s == DiffSpanStyle::Unchanged),
        "shared suffix should stay unhighlighted, got: {add:?}"
    );
}

#[test]
fn word_diff_version_suffix_change_is_minimal() {
    let diff = compute_file_diff(
        "Package.swift",
        "version = \"0.3.5\"\n",
        "version = \"0.3.6\"\n",
        false,
    );
    let rem = span_info(&diff.lines[0]);
    let add = span_info(&diff.lines[1]);

    assert_eq!(styled_text(&rem, DiffSpanStyle::Removed), "5");
    assert_eq!(styled_text(&add, DiffSpanStyle::Added), "6");
    assert_eq!(
        styled_text(&rem, DiffSpanStyle::Unchanged),
        "version = \"0.3.\""
    );
    assert_eq!(
        styled_text(&add, DiffSpanStyle::Unchanged),
        "version = \"0.3.\""
    );
}

#[test]
fn word_diff_slash_separated_insert_is_minimal() {
    let diff = compute_file_diff(
        "README.md",
        "- Unified and side-by-side diffs with syntax highlighting, word-level changes, context collapsing, rename detection, and image/SVG previews.\n",
        "- Unified and side-by-side diffs with syntax highlighting, word-level changes, context collapsing, rename detection, and image/SVG/Markdown/HTML previews.\n",
        false,
    );
    let rem = span_info(&diff.lines[0]);
    let add = span_info(&diff.lines[1]);

    assert_eq!(styled_text(&rem, DiffSpanStyle::Removed), "");
    assert_eq!(styled_text(&add, DiffSpanStyle::Added), "/Markdown/HTML");
    assert!(
        add.iter()
            .any(|(t, s)| t.contains("image/SVG") && *s == DiffSpanStyle::Unchanged),
        "shared slash-separated prefix should stay unhighlighted, got: {add:?}"
    );
}

#[test]
fn word_diff_middle_change_is_minimal() {
    let diff = compute_file_diff(
        "test.txt",
        "let key = \"prefix-old-suffix\"\n",
        "let key = \"prefix-new-suffix\"\n",
        false,
    );
    let rem = span_info(&diff.lines[0]);
    let add = span_info(&diff.lines[1]);

    assert_eq!(styled_text(&rem, DiffSpanStyle::Removed), "old");
    assert_eq!(styled_text(&add, DiffSpanStyle::Added), "new");
}

#[test]
fn word_diff_unrelated_words_stay_whole_word_highlighted() {
    let diff = compute_file_diff("test.txt", "hello world\n", "hello earth\n", false);
    let rem = span_info(&diff.lines[0]);
    let add = span_info(&diff.lines[1]);

    assert_eq!(styled_text(&rem, DiffSpanStyle::Removed), "world");
    assert_eq!(styled_text(&add, DiffSpanStyle::Added), "earth");
}

#[test]
fn word_diff_unpaired_lines_have_no_word_highlight() {
    let diff = compute_file_diff("test.txt", "aaa\nbbb\nccc\n", "AAA\nccc\n", false);
    let unpaired = diff
        .lines
        .iter()
        .find(|l| l.style == DiffSpanStyle::Removed && l.text() == "bbb")
        .expect("should find unpaired removed line 'bbb'");
    assert!(
        unpaired
            .spans
            .iter()
            .all(|s| s.style == DiffSpanStyle::Unchanged),
        "unpaired removed line should have Unchanged spans, got: {:?}",
        span_info(unpaired)
    );
}

#[test]
fn word_diff_multiple_changes_in_line() {
    let diff = compute_file_diff(
        "test.txt",
        "the quick brown fox\n",
        "the slow brown cat\n",
        false,
    );
    let rem = span_info(&diff.lines[0]);
    let add = span_info(&diff.lines[1]);

    assert!(
        rem.iter()
            .any(|(t, s)| t.contains("the") && *s == DiffSpanStyle::Unchanged)
    );
    assert!(
        rem.iter()
            .any(|(t, s)| t.contains("brown") && *s == DiffSpanStyle::Unchanged)
    );
    assert!(
        add.iter()
            .any(|(t, s)| t.contains("the") && *s == DiffSpanStyle::Unchanged)
    );
    assert!(
        add.iter()
            .any(|(t, s)| t.contains("brown") && *s == DiffSpanStyle::Unchanged)
    );
    assert!(
        rem.iter()
            .any(|(t, s)| t.contains("quick") && *s == DiffSpanStyle::Removed)
    );
    assert!(
        rem.iter()
            .any(|(t, s)| t.contains("fox") && *s == DiffSpanStyle::Removed)
    );
    assert!(
        add.iter()
            .any(|(t, s)| t.contains("slow") && *s == DiffSpanStyle::Added)
    );
    assert!(
        add.iter()
            .any(|(t, s)| t.contains("cat") && *s == DiffSpanStyle::Added)
    );
}

#[test]
fn word_diff_all_text_accounted_for() {
    let diff = compute_file_diff("test.txt", "hello world\n", "hello earth\n", false);
    assert_eq!(diff.lines[0].text(), "hello world");
    assert_eq!(diff.lines[1].text(), "hello earth");
}

#[test]
fn word_diff_whitespace_change() {
    let diff = compute_file_diff("test.txt", "a  b\n", "a b\n", false);
    let rem = span_info(&diff.lines[0]);
    let add = span_info(&diff.lines[1]);
    assert!(
        rem.iter()
            .any(|(t, s)| t.contains('a') && *s == DiffSpanStyle::Unchanged)
    );
    assert!(
        rem.iter()
            .any(|(t, s)| t.contains('b') && *s == DiffSpanStyle::Unchanged)
    );
    assert!(
        add.iter()
            .any(|(t, s)| t.contains('a') && *s == DiffSpanStyle::Unchanged)
    );
    assert!(
        add.iter()
            .any(|(t, s)| t.contains('b') && *s == DiffSpanStyle::Unchanged)
    );
}

#[test]
fn word_diff_context_lines_unaffected() {
    let diff = compute_file_diff("test.txt", "ctx\nold\n", "ctx\nnew\n", false);
    let ctx_line = &diff.lines[0];
    assert_eq!(ctx_line.style, DiffSpanStyle::Context);
    assert!(
        ctx_line
            .spans
            .iter()
            .all(|s| s.style == DiffSpanStyle::Context)
    );
}

#[test]
fn word_diff_empty_line_paired_with_content() {
    let diff = compute_file_diff("test.txt", "a\n\nc\n", "a\nhello\nc\n", false);
    let add_line = diff
        .lines
        .iter()
        .find(|l| l.style == DiffSpanStyle::Added)
        .expect("added line");
    let add_spans = span_info(add_line);
    assert!(
        add_spans
            .iter()
            .any(|(t, s)| *t == "hello" && *s == DiffSpanStyle::Added),
        "expected 'hello' as Added, got: {add_spans:?}"
    );
}

fn styled_text(spans: &[(&str, DiffSpanStyle)], style: DiffSpanStyle) -> String {
    spans
        .iter()
        .filter_map(|(text, span_style)| (*span_style == style).then_some(*text))
        .collect()
}
