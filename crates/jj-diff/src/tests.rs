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

#[test]
fn test_context_collapsing_keeps_tiny_gap_between_hunks() {
    let old_lines: Vec<String> = (1..=14).map(|i| format!("line {i}")).collect();
    let mut new_lines = old_lines.clone();
    new_lines[3] = "CHANGED 4".to_string();
    new_lines[11] = "CHANGED 12".to_string();

    let old = old_lines.join("\n") + "\n";
    let new = new_lines.join("\n") + "\n";
    let full = compute_file_diff_full("test.txt", &old, &new, false);
    let collapsed = collapse_context_with_mapping(&full);

    assert_eq!(
        collapsed.diff.lines.len(),
        full.lines.len(),
        "a one-line context gap is clearer inline than behind a separator"
    );
    assert!(
        collapsed
            .diff
            .lines
            .iter()
            .all(|l| l.style != DiffSpanStyle::Separator),
        "tiny context gaps should not be collapsed"
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

    // Both lines should have some changed content
    assert!(
        rem.iter()
            .any(|(_, s)| *s == DiffSpanStyle::Removed || *s == DiffSpanStyle::Unchanged),
        "removed line should have word-level spans, got: {rem:?}"
    );
    assert!(
        add.iter()
            .any(|(_, s)| *s == DiffSpanStyle::Added || *s == DiffSpanStyle::Unchanged),
        "added line should have word-level spans, got: {add:?}"
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
    assert!(
        diff.whitespace_only_hidden,
        "whitespace_only_hidden should flag ignore_whitespace-suppressed diffs"
    );
}

#[test]
fn test_eof_newline_added_synthesizes_visible_hunk() {
    // File gains a trailing newline — Rust's .lines() would miss this, we synthesize a visible pair.
    let old = "a\nb";
    let new = "a\nb\n";
    let diff = compute_file_diff("test.txt", old, new, false);

    let removed = diff
        .lines
        .iter()
        .find(|l| l.style == DiffSpanStyle::Removed)
        .expect("should have a removed line for EOF newline change");
    let added = diff
        .lines
        .iter()
        .find(|l| l.style == DiffSpanStyle::Added)
        .expect("should have an added line for EOF newline change");

    assert!(removed.no_eof_newline, "old side lacks trailing newline");
    assert!(!added.no_eof_newline, "new side has trailing newline");
}

#[test]
fn test_eof_newline_removed_synthesizes_visible_hunk() {
    let old = "a\nb\n";
    let new = "a\nb";
    let diff = compute_file_diff("test.txt", old, new, false);

    let removed = diff
        .lines
        .iter()
        .find(|l| l.style == DiffSpanStyle::Removed)
        .expect("should have a removed line");
    let added = diff
        .lines
        .iter()
        .find(|l| l.style == DiffSpanStyle::Added)
        .expect("should have an added line");

    assert!(!removed.no_eof_newline, "old side has trailing newline");
    assert!(added.no_eof_newline, "new side lacks trailing newline");
}

#[test]
fn test_eof_newline_only_no_whitespace_flag() {
    // EOF-newline-only changes are visible via synthesized hunk, not hidden by the whitespace flag.
    let diff = compute_file_diff("test.txt", "a\nb", "a\nb\n", false);
    assert!(
        !diff.whitespace_only_hidden,
        "EOF-newline change should synthesize a visible hunk, not set whitespace flag"
    );
}

#[test]
fn test_eof_splits_shared_context_when_real_changes_exist() {
    let diff = compute_file_diff("test.txt", "a\nchanged\nc", "a\nfixed\nc\n", false);

    let tail_removed = diff
        .lines
        .iter()
        .rev()
        .find(|l| l.style == DiffSpanStyle::Removed && l.old_line_no == Some(3))
        .expect("trailing shared-context 'c' should split into a removed line");
    let tail_added = diff
        .lines
        .iter()
        .rev()
        .find(|l| l.style == DiffSpanStyle::Added && l.new_line_no == Some(3))
        .expect("trailing shared-context 'c' should split into an added line");
    assert!(tail_removed.no_eof_newline);
    assert!(!tail_added.no_eof_newline);

    let mid_removed = diff
        .lines
        .iter()
        .find(|l| l.style == DiffSpanStyle::Removed && l.old_line_no == Some(2))
        .expect("middle 'changed' removed line must still exist");
    assert!(
        !mid_removed.no_eof_newline,
        "earlier removed line must not carry the EOF marker"
    );
}

#[test]
fn test_eof_marker_lands_on_side_without_its_own_op() {
    // Pure addition: old has no Removed op; marker must land on the last line belonging to old.
    let diff = compute_file_diff("test.txt", "a", "a\nb\n", false);

    let last_old = diff
        .lines
        .iter()
        .rev()
        .find(|l| l.old_line_no.is_some())
        .expect("diff should contain a line representing old");
    assert!(last_old.no_eof_newline);
    for line in diff.lines.iter().filter(|l| l.new_line_no.is_some()) {
        assert!(!line.no_eof_newline);
    }
}

#[test]
fn test_eof_newline_marker_on_real_change() {
    // Real line change AND EOF newline differs — the last Removed/Added should be marked.
    let old = "a\nold";
    let new = "a\nnew\n";
    let diff = compute_file_diff("test.txt", old, new, false);

    let last_removed = diff
        .lines
        .iter()
        .rev()
        .find(|l| l.style == DiffSpanStyle::Removed)
        .expect("should have a removed line");
    let last_added = diff
        .lines
        .iter()
        .rev()
        .find(|l| l.style == DiffSpanStyle::Added)
        .expect("should have an added line");

    assert!(last_removed.no_eof_newline, "last old line lacks newline");
    assert!(!last_added.no_eof_newline, "last new line has newline");
}

// ── Regression tests from v0.2.6–v0.2.7 session ───────────────

#[test]
fn test_large_file_single_line_change_is_fast() {
    // Regression: O(n²) LCS was 10s+ for Cargo.lock-sized files.
    // Prefix/suffix trimming should make this near-instant.
    let mut lines: Vec<String> = (0..2000).map(|i| format!("line {i}")).collect();
    let old = lines.join("\n") + "\n";
    lines[1000] = "CHANGED".to_string();
    let new = lines.join("\n") + "\n";

    let start = std::time::Instant::now();
    let diff = compute_file_diff("Cargo.lock", &old, &new, false);
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 1000,
        "2000-line diff with 1 change took {}ms (should be <1s)",
        elapsed.as_millis()
    );

    let changed: Vec<_> = diff
        .lines
        .iter()
        .filter(|l| l.style == DiffSpanStyle::Removed || l.style == DiffSpanStyle::Added)
        .collect();
    assert_eq!(changed.len(), 2, "should have 1 removed + 1 added line");
}

#[test]
fn test_prefix_suffix_trimming_correctness() {
    // Ensure prefix/suffix trimming produces same result as full diff
    let old = "a\nb\nc\nd\ne\nf\ng\n";
    let new = "a\nb\nX\nd\ne\nf\ng\n";
    let diff = compute_file_diff_full("test.txt", old, new, false);

    let styles: Vec<_> = diff.lines.iter().map(|l| l.style).collect();
    assert_eq!(
        styles,
        vec![
            DiffSpanStyle::Context, // a
            DiffSpanStyle::Context, // b
            DiffSpanStyle::Removed, // c
            DiffSpanStyle::Added,   // X
            DiffSpanStyle::Context, // d
            DiffSpanStyle::Context, // e
            DiffSpanStyle::Context, // f
            DiffSpanStyle::Context, // g
        ]
    );
}

#[test]
fn test_skip_highlight_for_lock_files() {
    // .lock files should skip syntax highlighting but still produce correct diffs
    let diff = compute_file_diff("Cargo.lock", "old\n", "new\n", false);
    assert_eq!(diff.lines.len(), 2);
    assert_eq!(diff.lines[0].style, DiffSpanStyle::Removed);
    assert_eq!(diff.lines[1].style, DiffSpanStyle::Added);
}

#[test]
fn test_collapse_context_with_mapping_preserves_changed_lines() {
    // Build a 20-line diff with a change at line 10
    let old_lines: Vec<String> = (1..=20).map(|i| format!("line {i}")).collect();
    let mut new_lines = old_lines.clone();
    new_lines[9] = "CHANGED".to_string();

    let old = old_lines.join("\n") + "\n";
    let new = new_lines.join("\n") + "\n";
    let full = compute_file_diff_full("test.txt", &old, &new, false);
    let collapsed = collapse_context_with_mapping(&full);

    // Collapsed should have fewer lines than full
    assert!(
        collapsed.diff.lines.len() < full.lines.len(),
        "collapsed ({}) should have fewer lines than full ({})",
        collapsed.diff.lines.len(),
        full.lines.len()
    );

    // Should have separator lines
    let separators: Vec<_> = collapsed
        .diff
        .lines
        .iter()
        .filter(|l| l.style == DiffSpanStyle::Separator)
        .collect();
    assert!(!separators.is_empty(), "should have separator lines");

    // Changed lines should be preserved
    let changed: Vec<_> = collapsed
        .diff
        .lines
        .iter()
        .filter(|l| l.style == DiffSpanStyle::Removed || l.style == DiffSpanStyle::Added)
        .collect();
    assert_eq!(changed.len(), 2, "should preserve removed + added lines");

    // Mapping should cover all non-separator display lines
    let non_separator_count = collapsed
        .diff
        .lines
        .iter()
        .filter(|l| l.style != DiffSpanStyle::Separator)
        .count();
    assert_eq!(
        collapsed.display_to_full.len(),
        non_separator_count,
        "mapping should have an entry for each non-separator line"
    );

    // Mapping values should be valid indices into the full diff
    for m in &collapsed.display_to_full {
        assert!(
            (m.full_line as usize) <= full.lines.len(),
            "full_line {} out of range (max {})",
            m.full_line,
            full.lines.len()
        );
    }
}

#[test]
fn test_collapse_with_mapping_small_diff_no_collapse() {
    // A diff smaller than 2*context+1 lines should not be collapsed
    let diff = compute_file_diff_full("test.txt", "a\nb\nc\n", "a\nX\nc\n", false);
    let collapsed = collapse_context_with_mapping(&diff);

    assert_eq!(
        collapsed.diff.lines.len(),
        diff.lines.len(),
        "small diff should not be collapsed"
    );
    assert!(
        collapsed
            .diff
            .lines
            .iter()
            .all(|l| l.style != DiffSpanStyle::Separator),
        "small diff should have no separators"
    );
}
