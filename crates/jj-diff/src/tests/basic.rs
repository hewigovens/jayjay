use super::*;

#[test]
fn identical_files_produce_no_changes() {
    let diff = compute_file_diff("test.txt", "hello\nworld\n", "hello\nworld\n", false);
    assert!(diff.lines.iter().all(|l| l.style == DiffSpanStyle::Context));
}

#[test]
fn added_line() {
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
fn removed_line() {
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
fn modified_line() {
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
fn no_phantom_changes_on_identical_lines() {
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
fn cargo_toml_like_diff() {
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
    let context_texts: Vec<_> = diff
        .lines
        .iter()
        .filter(|l| l.style == DiffSpanStyle::Context)
        .map(DiffLine::text)
        .collect();
    let toml_count = context_texts
        .iter()
        .filter(|t| t.contains("tree-sitter-toml"))
        .count();
    assert_eq!(
        toml_count, 1,
        "tree-sitter-toml should appear once as context, got {toml_count}"
    );

    for line in &diff.lines {
        if line.style == DiffSpanStyle::Context {
            let text = line.text();
            let also_removed = diff
                .lines
                .iter()
                .any(|l| l.style == DiffSpanStyle::Removed && l.text() == text);
            assert!(!also_removed, "Line '{text}' is both context and removed");
        }
    }
}

#[test]
fn line_numbers_are_correct() {
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
fn empty_to_content() {
    let diff = compute_file_diff("test.txt", "", "hello\nworld\n", false);
    assert!(diff.lines.iter().all(|l| l.style == DiffSpanStyle::Added));
    assert_eq!(diff.lines.len(), 2);
}

#[test]
fn content_to_empty() {
    let diff = compute_file_diff("test.txt", "hello\nworld\n", "", false);
    assert!(diff.lines.iter().all(|l| l.style == DiffSpanStyle::Removed));
    assert_eq!(diff.lines.len(), 2);
}

#[test]
fn trailing_whitespace_trimmed() {
    let diff = compute_file_diff(
        "test.txt",
        "hello   \nworld  \n",
        "hello   \nchanged  \n",
        false,
    );
    assert_eq!(diff.lines[0].text(), "hello");
}

#[test]
fn ignore_whitespace() {
    let diff = compute_file_diff("test.txt", "a  b\n", "a b\n", false);
    assert!(
        diff.lines
            .iter()
            .any(|l| l.style == DiffSpanStyle::Removed || l.style == DiffSpanStyle::Added)
    );

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
    assert!(diff.whitespace_only_hidden);
}

#[test]
fn skip_highlight_for_lock_files() {
    let diff = compute_file_diff("Cargo.lock", "old\n", "new\n", false);
    assert_eq!(diff.lines.len(), 2);
    assert_eq!(diff.lines[0].style, DiffSpanStyle::Removed);
    assert_eq!(diff.lines[1].style, DiffSpanStyle::Added);
}
