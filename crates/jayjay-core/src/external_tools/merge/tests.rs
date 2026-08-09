use super::*;

#[test]
fn builds_separate_conflict_hunks_with_resolved_context() {
    let directory = tempfile::tempdir().expect("directory");
    let left = directory.path().join("left");
    let base = directory.path().join("base");
    let right = directory.path().join("right");
    let output = directory.path().join("output");
    fs::write(&left, "before\nleft\nafter\n").expect("left");
    fs::write(&base, "before\nbase\nafter\n").expect("base");
    fs::write(&right, "before\nright\nafter\n").expect("right");
    fs::write(&output, "").expect("output");

    let merge = load_external_merge(&left, &base, &right, &output, false, 7).expect("merge");

    assert_eq!(merge.hunks.len(), 1);
    assert_eq!(merge.hunks[0].left, "left\n");
    assert_eq!(merge.hunks[0].base, "base\n");
    assert_eq!(merge.hunks[0].right, "right\n");
    assert!(merge.result.starts_with("before\n<<<<<<<"));
    assert!(merge.result.ends_with(">>>>>>> side #2\nafter\n"));
}

#[test]
fn resolves_external_conflicts_one_hunk_at_a_time() {
    let directory = tempfile::tempdir().expect("directory");
    let left = directory.path().join("left");
    let base = directory.path().join("base");
    let right = directory.path().join("right");
    let output = directory.path().join("output");
    fs::write(&left, "left one\nstable\nleft two\n").expect("left");
    fs::write(&base, "base one\nstable\nbase two\n").expect("base");
    fs::write(&right, "right one\nstable\nright two\n").expect("right");
    fs::write(&output, "").expect("output");
    let merge = load_external_merge(&left, &base, &right, &output, false, 7).expect("merge");

    assert_eq!(merge.hunks.len(), 2);
    let result = crate::merge_result_use_source(
        &merge.result,
        &merge.hunks[0],
        crate::MergeHunkSource::Left,
    )
    .expect("resolve first hunk");

    assert_eq!(conflict_marker_count(&result, 7), 1);
    assert!(result.starts_with("left one\nstable\n<<<<<<<"));
}

#[test]
fn counts_only_opening_markers_at_the_configured_length() {
    let content = "<<<<<<< Left\na\n=======\nb\n>>>>>>> Right\n<<<<<<<< literal\n";
    assert_eq!(conflict_marker_count(content, 7), 1);
    assert_eq!(conflict_marker_count(content, 8), 1);
}

#[test]
fn detects_complete_and_partial_conflict_marker_sequences() {
    let content = "<<<<<<< Left\na\n||||||| Base\nb\n=======\nc\n>>>>>>> Right\n";

    assert!(has_conflict_marker_remnants(content, 7));

    let without_opener = content.replacen("<<<<<<< Left\n", "", 1);
    assert_eq!(conflict_marker_count(&without_opener, 7), 0);
    assert!(has_conflict_marker_remnants(&without_opener, 7));
    assert!(has_conflict_marker_remnants("<<<<<<< Left\n=======\n", 7));
    assert!(has_conflict_marker_remnants("=======\n>>>>>>> Right\n", 7));
}

#[test]
fn ignores_isolated_literal_marker_lines() {
    for content in [
        "<<<<<<<\n",
        "|||||||\n",
        "=======\n",
        ">>>>>>>\n",
        "Markdown heading\n=======\nbody\n",
    ] {
        assert!(!has_conflict_marker_remnants(content, 7), "{content:?}");
    }
}

#[test]
fn ignores_longer_literal_marker_runs() {
    let content = "<<<<<<<< literal\n|||||||| literal\n======== literal\n>>>>>>>> literal\n";

    assert!(!has_conflict_marker_remnants(content, 7));
    assert!(has_conflict_marker_remnants(content, 8));
}

#[test]
fn oversized_output_is_not_editable_text() {
    let directory = tempfile::tempdir().expect("directory");
    let left = directory.path().join("left");
    let base = directory.path().join("base");
    let right = directory.path().join("right");
    let output = directory.path().join("output");
    fs::write(&left, "left\n").expect("left");
    fs::write(&base, "base\n").expect("base");
    fs::write(&right, "right\n").expect("right");
    let file = fs::File::create(&output).expect("output");
    file.set_len(crate::file_display::MAX_DIFF_BYTES as u64 + 1)
        .expect("grow output");
    drop(file);

    let merge = load_external_merge(&left, &base, &right, &output, false, 7).expect("merge");

    assert!(!merge.is_text, "placeholder output must not be saveable");
    assert!(merge.hunks.is_empty());
    assert!(merge.result.starts_with("<file too large"));
}

#[test]
fn real_text_starting_with_a_placeholder_prefix_stays_editable() {
    let directory = tempfile::tempdir().expect("directory");
    let left = directory.path().join("left");
    let base = directory.path().join("base");
    let right = directory.path().join("right");
    let output = directory.path().join("output");
    fs::write(&left, "symlink -> left\n").expect("left");
    fs::write(&base, "symlink -> base\n").expect("base");
    fs::write(&right, "symlink -> right\n").expect("right");
    fs::write(&output, "").expect("output");

    let merge = load_external_merge(&left, &base, &right, &output, false, 7).expect("merge");

    assert!(merge.is_text);
    assert_eq!(merge.hunks.len(), 1);
    assert!(merge.result.contains("symlink -> left"));
}

#[test]
fn missing_base_source_writes_an_empty_result() {
    let directory = tempfile::tempdir().expect("directory");
    let output = directory.path().join("merged");
    fs::write(&output, "conflicted").expect("output");

    save_external_merge(&output, ExternalMergeResolution::Source(Path::new("")))
        .expect("empty base");

    assert_eq!(fs::read_to_string(output).expect("output"), "");
}

#[test]
fn preserves_an_initialized_empty_merge_result() {
    let directory = tempfile::tempdir().expect("directory");
    let left = directory.path().join("left");
    let base = directory.path().join("base");
    let right = directory.path().join("right");
    let output = directory.path().join("output");
    fs::write(&left, "left\n").expect("left");
    fs::write(&base, "base\n").expect("base");
    fs::write(&right, "right\n").expect("right");
    fs::write(&output, "").expect("output");

    let merge = load_external_merge(&left, &base, &right, &output, true, 7).expect("merge");

    assert!(merge.is_text);
    assert!(merge.result.is_empty());
    assert!(merge.hunks.is_empty());
}

#[cfg(unix)]
#[test]
fn source_selection_preserves_the_merge_result_executable_bit() {
    let directory = tempfile::tempdir().expect("directory");
    let source = directory.path().join("source");
    let output = directory.path().join("output");
    fs::write(&source, "selected\n").expect("source");
    fs::write(&output, "conflicted\n").expect("output");
    crate::filesystem::set_executable(&source, false).expect("source mode");
    crate::filesystem::set_executable(&output, true).expect("output mode");

    save_external_merge(&output, ExternalMergeResolution::Source(&source)).expect("save");

    assert_eq!(fs::read_to_string(&output).unwrap(), "selected\n");
    assert!(crate::filesystem::is_executable(
        &fs::metadata(output).unwrap()
    ));
}
