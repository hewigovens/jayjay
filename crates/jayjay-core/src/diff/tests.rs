use super::*;

#[test]
fn test_identical_files_produce_no_changes() {
    let diff = compute_file_diff("test.txt", "hello\nworld\n", "hello\nworld\n", false);
    assert!(diff.lines.iter().all(|l| l.style == DiffSpanStyle::Context));
}

#[test]
fn test_added_line() {
    let diff = compute_file_diff("test.txt", "a\nc\n", "a\nb\nc\n", false);
    let styles: Vec<_> = diff.lines.iter().map(|l| l.style).collect();
    assert_eq!(
        styles,
        vec![
            DiffSpanStyle::Context,
            DiffSpanStyle::Added,
            DiffSpanStyle::Context
        ]
    );
}

#[test]
fn test_removed_line() {
    let diff = compute_file_diff("test.txt", "a\nb\nc\n", "a\nc\n", false);
    let styles: Vec<_> = diff.lines.iter().map(|l| l.style).collect();
    assert_eq!(
        styles,
        vec![
            DiffSpanStyle::Context,
            DiffSpanStyle::Removed,
            DiffSpanStyle::Context
        ]
    );
}

#[test]
fn test_modified_line() {
    let diff = compute_file_diff("test.txt", "a\nold\nc\n", "a\nnew\nc\n", false);
    let styles: Vec<_> = diff.lines.iter().map(|l| l.style).collect();
    assert_eq!(
        styles,
        vec![
            DiffSpanStyle::Context,
            DiffSpanStyle::Removed,
            DiffSpanStyle::Added,
            DiffSpanStyle::Context
        ]
    );
}

#[test]
fn test_no_phantom_changes_on_identical_lines() {
    let content = "line1\nline2\nline3\nline4\nline5\n";
    let diff = compute_file_diff("test.txt", content, content, false);
    let changed: Vec<_> = diff
        .lines
        .iter()
        .filter(|l| l.style != DiffSpanStyle::Context && l.style != DiffSpanStyle::Separator)
        .collect();
    assert!(
        changed.is_empty(),
        "Identical content should have no changes, got {:?}",
        changed.len()
    );
}

#[test]
fn test_cargo_toml_like_diff() {
    let old = r#"tree-sitter = "0.26"
tree-sitter-highlight = "0.26"
tree-sitter-rust = "0.24"
tree-sitter-typescript = "0.23"
tree-sitter-python = "0.23"
tree-sitter-json = "0.24"
tree-sitter-toml = "0.20"
tree-sitter-html = "0.23"
tree-sitter-go = "0.23"
tree-sitter-cpp = "0.23"
"#;
    let new = r#"tree-sitter = "0.26"
tree-sitter-highlight = "0.26"
tree-sitter-rust = "0.24"
tree-sitter-javascript = "0.25"
tree-sitter-typescript = "0.23"
tree-sitter-python = "0.23"
tree-sitter-json = "0.24"
tree-sitter-toml = "0.20"
tree-sitter-css = "0.23"
tree-sitter-html = "0.23"
tree-sitter-go = "0.23"
tree-sitter-c = "0.23"
tree-sitter-cpp = "0.23"
"#;
    let diff = compute_file_diff("Cargo.toml", old, new, false);

    // Context lines should not be duplicated
    let context_texts: Vec<_> = diff
        .lines
        .iter()
        .filter(|l| l.style == DiffSpanStyle::Context)
        .map(|l| l.spans.iter().map(|s| s.text.as_str()).collect::<String>())
        .collect();

    // "tree-sitter-toml" should appear exactly once as context
    let toml_count = context_texts
        .iter()
        .filter(|t| t.contains("tree-sitter-toml"))
        .count();
    assert_eq!(
        toml_count, 1,
        "tree-sitter-toml should appear once as context, got {toml_count}"
    );

    // No line should appear as both context AND removed
    for line in &diff.lines {
        if line.style == DiffSpanStyle::Context {
            let text: String = line.spans.iter().map(|s| s.text.as_str()).collect();
            let also_removed = diff.lines.iter().any(|l| {
                l.style == DiffSpanStyle::Removed
                    && l.spans.iter().map(|s| s.text.as_str()).collect::<String>() == text
            });
            assert!(!also_removed, "Line '{text}' is both context and removed");
        }
    }
}

#[test]
fn test_line_numbers_are_correct() {
    let diff = compute_file_diff("test.txt", "a\nb\nc\n", "a\nx\nc\n", false);
    for line in &diff.lines {
        match line.style {
            DiffSpanStyle::Context => {
                assert!(line.old_line_no.is_some());
                assert!(line.new_line_no.is_some());
            }
            DiffSpanStyle::Removed => {
                assert!(line.old_line_no.is_some());
                assert!(line.new_line_no.is_none());
            }
            DiffSpanStyle::Added => {
                assert!(line.old_line_no.is_none());
                assert!(line.new_line_no.is_some());
            }
            _ => {}
        }
    }
}

#[test]
fn test_empty_to_content() {
    let diff = compute_file_diff("test.txt", "", "hello\nworld\n", false);
    assert!(diff.lines.iter().all(|l| l.style == DiffSpanStyle::Added));
    assert_eq!(diff.lines.len(), 2);
}

#[test]
fn test_content_to_empty() {
    let diff = compute_file_diff("test.txt", "hello\nworld\n", "", false);
    assert!(diff.lines.iter().all(|l| l.style == DiffSpanStyle::Removed));
    assert_eq!(diff.lines.len(), 2);
}

#[test]
fn test_context_collapsing() {
    let old_lines: Vec<String> = (1..=20).map(|i| format!("line {i}")).collect();
    let mut new_lines = old_lines.clone();
    new_lines[9] = "CHANGED".to_string(); // Change line 10

    let old = old_lines.join("\n") + "\n";
    let new = new_lines.join("\n") + "\n";
    let diff = compute_file_diff("test.txt", &old, &new, false);

    let separators: Vec<_> = diff
        .lines
        .iter()
        .filter(|l| l.style == DiffSpanStyle::Separator)
        .collect();
    assert!(
        !separators.is_empty(),
        "Should have separator lines for collapsed context"
    );
}

// ── Word-level diff highlighting tests ──────────────────────────

/// Helper: collect (text, style) pairs from spans of a DiffLine.
fn span_info(line: &DiffLine) -> Vec<(&str, DiffSpanStyle)> {
    line.spans
        .iter()
        .map(|s| (s.text.as_str(), s.style))
        .collect()
}

#[test]
fn test_word_diff_single_word_change() {
    // "hello world" → "hello earth" — only the second word changes
    let diff = compute_file_diff("test.txt", "hello world\n", "hello earth\n", false);
    let styles: Vec<_> = diff.lines.iter().map(|l| l.style).collect();
    assert_eq!(styles, vec![DiffSpanStyle::Removed, DiffSpanStyle::Added]);

    // Removed line: "hello " is unchanged, "world" is removed
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

    // Added line: "hello " is unchanged, "earth" is added
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
fn test_word_diff_preserves_line_level_style() {
    // Line-level style (DiffLine.style) should still be Removed/Added
    let diff = compute_file_diff("test.txt", "foo bar\n", "foo baz\n", false);
    assert_eq!(diff.lines[0].style, DiffSpanStyle::Removed);
    assert_eq!(diff.lines[1].style, DiffSpanStyle::Added);
}

#[test]
fn test_word_diff_entirely_different_lines() {
    // Completely different content — all spans should be Removed/Added
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
fn test_word_diff_prefix_change() {
    // Change at the start: "old_func(x)" → "new_func(x)"
    let diff = compute_file_diff("test.txt", "old_func(x)\n", "new_func(x)\n", false);
    let rem = span_info(&diff.lines[0]);
    let add = span_info(&diff.lines[1]);

    // "(x)" should be unchanged on both sides
    assert!(
        rem.iter()
            .any(|(t, s)| t.contains("(") && *s == DiffSpanStyle::Unchanged),
        "matching punctuation should be Unchanged in removed line, got: {rem:?}"
    );
    assert!(
        add.iter()
            .any(|(t, s)| t.contains("(") && *s == DiffSpanStyle::Unchanged),
        "matching punctuation should be Unchanged in added line, got: {add:?}"
    );
}

#[test]
fn test_word_diff_unpaired_lines_have_no_word_highlight() {
    // 2 removes, 1 add: first pair gets word diff, second remove is unpaired
    let diff = compute_file_diff("test.txt", "aaa\nbbb\nccc\n", "AAA\nccc\n", false);
    // Lines: Removed(aaa), Added(AAA), Removed(bbb), Context(ccc)
    // The paired Remove(aaa)/Add(AAA) should have word-level styles
    // The unpaired Remove(bbb) should have all Removed spans (no Unchanged)
    let unpaired = diff.lines.iter().find(|l| {
        l.style == DiffSpanStyle::Removed
            && l.spans.iter().map(|s| s.text.as_str()).collect::<String>() == "bbb"
    });
    assert!(
        unpaired.is_some(),
        "should find unpaired removed line 'bbb'"
    );
    let unpaired = unpaired.unwrap();
    assert!(
        unpaired
            .spans
            .iter()
            .all(|s| s.style == DiffSpanStyle::Unchanged),
        "unpaired removed line should have Unchanged spans (no word highlight), got: {:?}",
        span_info(unpaired)
    );
}

#[test]
fn test_word_diff_multiple_changes_in_line() {
    // Multiple words change: "the quick brown fox" → "the slow brown cat"
    let diff = compute_file_diff(
        "test.txt",
        "the quick brown fox\n",
        "the slow brown cat\n",
        false,
    );
    let rem = span_info(&diff.lines[0]);
    let add = span_info(&diff.lines[1]);

    // "the" and "brown" should be unchanged on both sides
    // (word boundaries may include adjacent whitespace, so use contains)
    assert!(
        rem.iter()
            .any(|(t, s)| t.contains("the") && *s == DiffSpanStyle::Unchanged),
        "'the' should be Unchanged in removed, got: {rem:?}"
    );
    assert!(
        rem.iter()
            .any(|(t, s)| t.contains("brown") && *s == DiffSpanStyle::Unchanged),
        "'brown' should be Unchanged in removed, got: {rem:?}"
    );
    assert!(
        add.iter()
            .any(|(t, s)| t.contains("the") && *s == DiffSpanStyle::Unchanged),
        "'the' should be Unchanged in added, got: {add:?}"
    );
    assert!(
        add.iter()
            .any(|(t, s)| t.contains("brown") && *s == DiffSpanStyle::Unchanged),
        "'brown' should be Unchanged in added, got: {add:?}"
    );

    // "quick" and "fox" should be Removed; "slow" and "cat" should be Added
    assert!(
        rem.iter()
            .any(|(t, s)| t.contains("quick") && *s == DiffSpanStyle::Removed),
        "'quick' should be Removed, got: {rem:?}"
    );
    assert!(
        rem.iter()
            .any(|(t, s)| t.contains("fox") && *s == DiffSpanStyle::Removed),
        "'fox' should be Removed, got: {rem:?}"
    );
    assert!(
        add.iter()
            .any(|(t, s)| t.contains("slow") && *s == DiffSpanStyle::Added),
        "'slow' should be Added, got: {add:?}"
    );
    assert!(
        add.iter()
            .any(|(t, s)| t.contains("cat") && *s == DiffSpanStyle::Added),
        "'cat' should be Added, got: {add:?}"
    );
}

#[test]
fn test_word_diff_all_text_accounted_for() {
    // All text from both lines should be present in spans
    let diff = compute_file_diff("test.txt", "hello world\n", "hello earth\n", false);
    let rem_text: String = diff.lines[0]
        .spans
        .iter()
        .map(|s| s.text.as_str())
        .collect();
    let add_text: String = diff.lines[1]
        .spans
        .iter()
        .map(|s| s.text.as_str())
        .collect();
    assert_eq!(rem_text, "hello world");
    assert_eq!(add_text, "hello earth");
}

#[test]
fn test_word_diff_whitespace_change() {
    // Only whitespace changes: "a  b" → "a b"
    let diff = compute_file_diff("test.txt", "a  b\n", "a b\n", false);
    let rem = span_info(&diff.lines[0]);
    let add = span_info(&diff.lines[1]);

    // "a" and "b" should be unchanged on both sides
    // (spans may include adjacent whitespace, so use contains)
    assert!(
        rem.iter()
            .any(|(t, s)| t.contains('a') && *s == DiffSpanStyle::Unchanged),
        "'a' should be Unchanged in removed, got: {rem:?}"
    );
    assert!(
        rem.iter()
            .any(|(t, s)| t.contains('b') && *s == DiffSpanStyle::Unchanged),
        "'b' should be Unchanged in removed, got: {rem:?}"
    );
    assert!(
        add.iter()
            .any(|(t, s)| t.contains('a') && *s == DiffSpanStyle::Unchanged),
        "'a' should be Unchanged in added, got: {add:?}"
    );
    assert!(
        add.iter()
            .any(|(t, s)| t.contains('b') && *s == DiffSpanStyle::Unchanged),
        "'b' should be Unchanged in added, got: {add:?}"
    );
}

#[test]
fn test_word_diff_context_lines_unaffected() {
    // Context lines should still have Context style spans (no Unchanged)
    let diff = compute_file_diff("test.txt", "ctx\nold\n", "ctx\nnew\n", false);
    let ctx_line = &diff.lines[0];
    assert_eq!(ctx_line.style, DiffSpanStyle::Context);
    assert!(
        ctx_line
            .spans
            .iter()
            .all(|s| s.style == DiffSpanStyle::Context),
        "context line spans should all be Context, got: {:?}",
        span_info(ctx_line)
    );
}

#[test]
fn test_word_diff_empty_line_paired_with_content() {
    // Empty line changed to content (or vice versa)
    let diff = compute_file_diff("test.txt", "a\n\nc\n", "a\nhello\nc\n", false);
    // The empty line and "hello" line form a remove/add pair
    let rem_line = diff
        .lines
        .iter()
        .find(|l| l.style == DiffSpanStyle::Removed);
    let add_line = diff.lines.iter().find(|l| l.style == DiffSpanStyle::Added);
    assert!(rem_line.is_some());
    assert!(add_line.is_some());
    // The added line should have "hello" marked as Added
    let add_spans = span_info(add_line.unwrap());
    assert!(
        add_spans
            .iter()
            .any(|(t, s)| *t == "hello" && *s == DiffSpanStyle::Added),
        "expected 'hello' as Added, got: {add_spans:?}"
    );
}

#[test]
fn test_trailing_whitespace_trimmed() {
    // Lines with trailing spaces should be trimmed for display (regression: red
    // background extending beyond text when trailing spaces were present).
    let diff = compute_file_diff(
        "test.txt",
        "hello   \nworld  \n",
        "hello   \nchanged  \n",
        false,
    );
    // The context line "hello" should have no trailing spaces in spans
    let context_line = &diff.lines[0];
    let text: String = context_line.spans.iter().map(|s| s.text.as_str()).collect();
    assert_eq!(text, "hello", "trailing whitespace should be trimmed");
}

#[test]
fn test_default_revset_not_empty() {
    assert!(!crate::DEFAULT_REVSET.is_empty());
    assert!(
        crate::DEFAULT_REVSET.contains("@"),
        "default revset should contain '@'"
    );
}

#[test]
fn test_ignore_whitespace() {
    // With ignore_whitespace=false, different spacing is a change
    let diff = compute_file_diff("test.txt", "a  b\n", "a b\n", false);
    assert!(
        diff.lines
            .iter()
            .any(|l| l.style == DiffSpanStyle::Removed || l.style == DiffSpanStyle::Added)
    );

    // With ignore_whitespace=true, different spacing is treated as equal
    let diff = compute_file_diff("test.txt", "a  b\n", "a b\n", true);
    let changed: Vec<_> = diff
        .lines
        .iter()
        .filter(|l| l.style == DiffSpanStyle::Removed || l.style == DiffSpanStyle::Added)
        .collect();
    assert!(
        changed.is_empty(),
        "whitespace-only change should be hidden when ignoring whitespace, got {} changes",
        changed.len()
    );
}
