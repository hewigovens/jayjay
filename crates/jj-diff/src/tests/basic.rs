use super::*;

#[test]
fn line_styles_and_numbers_for_add_remove_modify_and_empty_sides() {
    use DiffSpanStyle::{Added, Context, Removed};
    let cases = [
        ("hello\nworld\n", "hello\nworld\n", vec![Context, Context]),
        ("a\nc\n", "a\nb\nc\n", vec![Context, Added, Context]),
        ("a\nb\nc\n", "a\nc\n", vec![Context, Removed, Context]),
        (
            "a\nold\nc\n",
            "a\nnew\nc\n",
            vec![Context, Removed, Added, Context],
        ),
        ("", "hello\nworld\n", vec![Added, Added]),
        ("hello\nworld\n", "", vec![Removed, Removed]),
        (
            "a\nb\nc\nd\ne\nf\ng\n",
            "a\nb\nX\nd\ne\nf\ng\n",
            vec![
                Context, Context, Removed, Added, Context, Context, Context, Context,
            ],
        ),
    ];
    for (old, new, styles) in cases {
        let diff = compute_file_diff_full("test.txt", old, new, false);
        assert_eq!(
            diff.lines.iter().map(|l| l.style).collect::<Vec<_>>(),
            styles,
            "{old:?} -> {new:?}"
        );
        let mut old_no = 0;
        let mut new_no = 0;
        for line in &diff.lines {
            if line.style != Added {
                old_no += 1;
                assert_eq!(line.old_line_no, Some(old_no), "{old:?} -> {new:?}");
            } else {
                assert_eq!(line.old_line_no, None);
            }
            if line.style != Removed {
                new_no += 1;
                assert_eq!(line.new_line_no, Some(new_no), "{old:?} -> {new:?}");
            } else {
                assert_eq!(line.new_line_no, None);
            }
        }
    }
}

#[test]
fn histogram_diff_keeps_repeated_dependency_lines_as_context() {
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
    let changed: Vec<_> = diff
        .lines
        .iter()
        .filter(|l| l.is_changed())
        .map(|l| (l.style, l.text()))
        .collect();
    assert_eq!(
        changed,
        [
            "tree-sitter-javascript = \"0.25\"",
            "tree-sitter-css = \"0.23\"",
            "tree-sitter-c = \"0.23\""
        ]
        .map(|text| (DiffSpanStyle::Added, text.to_owned()))
    );
}

#[test]
fn change_groups_are_contiguous_runs_of_changed_lines() {
    let diff = compute_file_diff("test.txt", "a\nold\nc\nold2\n", "a\nnew\nc\nnew2\n", false);

    let groups = change_groups(&diff.lines);

    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].index, 0);
    assert_eq!(groups[0].start_line, 2);
    assert_eq!(groups[0].anchor_side, DiffSide::Old);
    assert_eq!(groups[0].anchor_line, 2);
    assert_eq!(groups[0].anchor_excerpt, "old");
    assert_eq!(groups[1].index, 1);
    assert_eq!(groups[1].start_line, 5);
}

#[test]
fn change_group_for_anchor_requires_matching_side_line_and_excerpt() {
    let diff = compute_file_diff("test.txt", "a\nold\nc\n", "a\nnew\nc\n", false);

    let group = change_group_for_anchor(&diff.lines, DiffSide::New, 2, "new")
        .expect("new-side anchor maps");
    assert_eq!(group.index, 0);
    assert!(change_group_for_anchor(&diff.lines, DiffSide::New, 2, "other").is_none());
    assert!(change_group_for_anchor(&diff.lines, DiffSide::Old, 2, "new").is_none());
}

#[test]
fn whitespace_is_trimmed_from_text_and_optionally_ignored_in_pairing() {
    let diff = compute_file_diff(
        "test.txt",
        "hello   \nworld  \n",
        "hello   \nchanged  \n",
        false,
    );
    assert_eq!(diff.lines[0].text(), "hello");

    let diff = compute_file_diff("test.txt", "a  b\n", "a b\n", false);
    assert!(diff.lines.iter().any(DiffLine::is_changed));
    assert!(!diff.whitespace_only_hidden);

    let diff = compute_file_diff("test.txt", "a  b\n", "a b\n", true);
    assert!(diff.lines.iter().all(|l| !l.is_changed()));
    assert!(diff.whitespace_only_hidden);
}

#[test]
fn unified_replacements_group_sides_without_changing_pairs_or_anchors() {
    let diff = compute_file_diff_full(
        "sample.rs",
        "head\nlet a = 1;\nlet b = 2;\nmiddle\nold one\nold two\nold three\ntail\n",
        "head\nlet a = 10;\nlet b = 20;\nlet c = 30;\nmiddle\nnew one\ntail\n",
        false,
    );
    let display = build_diff_display_lines(&diff.lines);
    assert_eq!(
        display.iter().map(DiffLine::text).collect::<Vec<_>>(),
        [
            "head",
            "let a = 1;",
            "let b = 2;",
            "let a = 10;",
            "let b = 20;",
            "let c = 30;",
            "middle",
            "old one",
            "old two",
            "old three",
            "new one",
            "tail"
        ],
    );
    for line in &display {
        let original = diff
            .lines
            .iter()
            .find(|original| {
                original.old_line_no == line.old_line_no && original.new_line_no == line.new_line_no
            })
            .unwrap();
        assert_eq!(span_info(line), span_info(original));
        if let Some((side, number)) = anchor_side_and_number(line) {
            let before = change_group_for_anchor(&diff.lines, side, number, &line.text()).unwrap();
            let after = change_group_for_anchor(&display, side, number, &line.text()).unwrap();
            assert_eq!(
                (before.index, before.start_line, before.end_line),
                (after.index, after.start_line, after.end_line)
            );
        }
    }
    assert_eq!(
        sbs_line_to_row(&diff.lines),
        vec![0, 1, 2, 1, 2, 3, 4, 5, 6, 7, 5, 8]
    );
    let rows = build_side_by_side_rows(&diff.lines);
    assert_eq!(
        rows.iter()
            .map(|row| (row.old.text(), row.new.text()))
            .collect::<Vec<_>>(),
        [
            ("head", "head"),
            ("let a = 1;", "let a = 10;"),
            ("let b = 2;", "let b = 20;"),
            ("", "let c = 30;"),
            ("middle", "middle"),
            ("old one", "new one"),
            ("old two", ""),
            ("old three", ""),
            ("tail", "tail")
        ]
        .map(|(old, new)| (old.to_owned(), new.to_owned())),
    );
}
