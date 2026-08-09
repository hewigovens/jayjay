use super::*;

#[test]
fn applies_selected_external_diff_lines_to_the_right_tree() {
    let right = tempfile::tempdir().expect("right");
    fs::write(right.path().join("file.txt"), "a\nx\n").expect("right file");
    let selection = DiffEditFileSelection {
        path: "file.txt".to_owned(),
        old_path: None,
        old_content: Some("a\nb\n".to_owned()),
        new_content: Some("a\nx\n".to_owned()),
        hunk_type: HunkType::Modified,
        line_ranges: vec![DiffEditRange {
            start_line: 3,
            end_line: 3,
        }],
    };

    apply_external_diff_selections(right.path(), right.path(), &[external(selection)], false)
        .expect("apply");
    assert_eq!(
        fs::read_to_string(right.path().join("file.txt")).expect("read"),
        "a\nb\nx\n"
    );
}

#[test]
fn applies_scanner_validated_text_with_a_placeholder_prefix() {
    let left = tempfile::tempdir().expect("left");
    let right = tempfile::tempdir().expect("right");
    fs::write(left.path().join("literal.txt"), "symlink -> old\n").expect("left text");
    fs::write(right.path().join("literal.txt"), "symlink -> new\n").expect("right text");
    let selections = all_loaded_selections(left.path(), right.path());

    assert_eq!(selections[0].whole_file_side, None);
    apply_external_diff_selections(left.path(), right.path(), &selections, false).expect("apply");

    assert_eq!(
        fs::read_to_string(right.path().join("literal.txt")).expect("right text"),
        "symlink -> new\n"
    );
}

#[test]
fn empty_selection_restores_the_left_file() {
    let right = tempfile::tempdir().expect("right");
    fs::write(right.path().join("file.txt"), "after\n").expect("right file");
    let selection = DiffEditFileSelection {
        path: "file.txt".to_owned(),
        old_path: None,
        old_content: Some("before\n".to_owned()),
        new_content: Some("after\n".to_owned()),
        hunk_type: HunkType::Modified,
        line_ranges: vec![],
    };

    apply_external_diff_selections(right.path(), right.path(), &[external(selection)], false)
        .expect("apply");
    assert_eq!(
        fs::read_to_string(right.path().join("file.txt")).expect("read"),
        "before\n"
    );
}

#[test]
fn rejects_parent_path_components() {
    let right = tempfile::tempdir().expect("right");
    let selection = DiffEditFileSelection {
        path: "../escape".to_owned(),
        old_path: None,
        old_content: None,
        new_content: Some("x".to_owned()),
        hunk_type: HunkType::Added,
        line_ranges: vec![],
    };
    let error =
        apply_external_diff_selections(right.path(), right.path(), &[external(selection)], false)
            .expect_err("unsafe path");
    assert!(error.to_string().contains("invalid external diff path"));
}

#[cfg(unix)]
#[test]
fn refuses_to_write_through_a_symlink() {
    use std::os::unix::fs::symlink;

    let right = tempfile::tempdir().expect("right");
    let outside = tempfile::tempdir().expect("outside");
    let outside_file = outside.path().join("file.txt");
    fs::write(&outside_file, "outside\n").expect("outside file");
    symlink(&outside_file, right.path().join("file.txt")).expect("symlink");
    let selection = DiffEditFileSelection {
        path: "file.txt".to_owned(),
        old_path: None,
        old_content: Some("before\n".to_owned()),
        new_content: Some("after\n".to_owned()),
        hunk_type: HunkType::Modified,
        line_ranges: vec![],
    };

    let error =
        apply_external_diff_selections(right.path(), right.path(), &[external(selection)], false)
            .expect_err("symlink must be rejected");

    assert!(error.to_string().contains("refusing to write"));
    assert_eq!(
        fs::read_to_string(outside_file).expect("outside"),
        "outside\n"
    );
}

#[test]
fn applies_a_single_file_selection_to_the_supplied_output() {
    let root = tempfile::tempdir().expect("root");
    let right = root.path().join("post-image.tmp");
    fs::write(&right, "after\n").expect("right");
    let selection = DiffEditFileSelection {
        path: "source.txt".to_owned(),
        old_path: None,
        old_content: Some("before\n".to_owned()),
        new_content: Some("after\n".to_owned()),
        hunk_type: HunkType::Modified,
        line_ranges: vec![],
    };

    apply_external_diff_selections(&right, &right, &[external(selection)], false).expect("apply");

    assert_eq!(fs::read_to_string(right).expect("right"), "before\n");
}
