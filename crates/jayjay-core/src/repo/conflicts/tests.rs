use std::fs;

use jj_test::{current_op_id, init_jj_repo, run_jj_in};

use crate::{MergeHunkSource, Repo, merge_result_use_source};

fn conflict_fixture() -> (tempfile::TempDir, Repo) {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");

    run_jj_in(&repo_path, &["new", "@"]);
    fs::write(repo_path.join("hello.txt"), "left\n").expect("write left");
    run_jj_in(&repo_path, &["describe", "-m", "left"]);
    run_jj_in(&repo_path, &["bookmark", "create", "left", "-r", "@"]);

    run_jj_in(&repo_path, &["new", "@-"]);
    fs::write(repo_path.join("hello.txt"), "right\n").expect("write right");
    run_jj_in(&repo_path, &["describe", "-m", "right"]);
    run_jj_in(&repo_path, &["new", "left", "@"]);

    let repo = Repo::open(&repo_path).expect("open conflicted repo");
    (temp_dir, repo)
}

#[test]
fn loads_and_applies_an_embedded_conflict_edit() {
    let (temp_dir, repo) = conflict_fixture();
    let summary = repo.show_summary("@").expect("show conflict summary");
    assert!(
        summary
            .diff
            .iter()
            .find(|hunk| hunk.path == "hello.txt")
            .is_some_and(|hunk| hunk.supports_conflict_editor)
    );
    let editor = repo.conflict_editor("@", "hello.txt").expect("load editor");

    assert_eq!(editor.base, "hello from jayjay\n");
    assert!(editor.left == "left\n" || editor.left == "right\n");
    assert!(editor.right == "left\n" || editor.right == "right\n");
    assert_eq!(editor.side_count, 2);
    assert!(editor.is_text);
    assert!(
        editor
            .result
            .contains(&"<".repeat(editor.marker_length as usize))
    );

    repo.apply_conflict_editor("@", &editor, "combined\n")
        .expect("apply editor");

    assert!(!repo.show("@").expect("show change").info.has_conflict);
    assert_eq!(
        repo.file_content("@", "hello.txt").expect("file content"),
        "combined"
    );
    assert_eq!(
        fs::read_to_string(temp_dir.path().join("repo/hello.txt")).expect("working-copy content"),
        "combined\n"
    );
}

#[test]
fn tree_conflicts_do_not_offer_the_file_conflict_editor() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let item = repo_path.join("item");

    run_jj_in(&repo_path, &["new", "@"]);
    fs::write(&item, "file side\n").expect("write file side");
    run_jj_in(&repo_path, &["describe", "-m", "file side"]);
    run_jj_in(&repo_path, &["bookmark", "create", "file-side", "-r", "@"]);

    run_jj_in(&repo_path, &["new", "@-"]);
    fs::create_dir(&item).expect("create directory side");
    fs::write(item.join("child.txt"), "directory side\n").expect("write directory side");
    run_jj_in(&repo_path, &["describe", "-m", "directory side"]);
    run_jj_in(&repo_path, &["new", "file-side", "@"]);

    let repo = Repo::open(&repo_path).expect("open tree conflict");
    let conflicts = repo.resolve_list("@").expect("list tree conflict");
    assert_eq!(conflicts, vec!["item".to_owned()]);
    let summary = repo.show_summary("@").expect("show tree conflict");
    let hunk = summary
        .diff
        .iter()
        .find(|hunk| hunk.path == "item")
        .expect("tree conflict summary");

    assert!(!hunk.supports_conflict_editor);
    assert!(repo.conflict_editor("@", "item").is_err());
}

#[test]
fn non_text_conflicts_do_not_offer_the_file_conflict_editor() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let path = repo_path.join("data.bin");
    run_jj_in(&repo_path, &["new", "@"]);
    fs::write(&path, b"left\0bytes").expect("write left binary");
    run_jj_in(
        &repo_path,
        &["bookmark", "create", "left-binary", "-r", "@"],
    );
    run_jj_in(&repo_path, &["new", "@-"]);
    fs::write(&path, b"right\0bytes").expect("write right binary");
    run_jj_in(&repo_path, &["new", "left-binary", "@"]);

    let repo = Repo::open(&repo_path).expect("open binary conflict");
    let summary = repo.show_summary("@").expect("show binary conflict");
    let hunk = summary
        .diff
        .iter()
        .find(|hunk| hunk.path == "data.bin")
        .expect("binary conflict summary");

    assert!(!hunk.supports_conflict_editor);
    assert!(!repo.conflict_editor("@", "data.bin").unwrap().is_text);
}

#[test]
fn saving_unchanged_markers_keeps_the_conflict() {
    let (_temp_dir, repo) = conflict_fixture();
    let editor = repo.conflict_editor("@", "hello.txt").expect("load editor");

    repo.apply_conflict_editor("@", &editor, &editor.result)
        .expect("save partial resolution");

    assert!(repo.show("@").expect("show change").info.has_conflict);
}

#[test]
fn applies_one_hunk_source_without_editing_marker_prefixes() {
    let (_temp_dir, repo) = conflict_fixture();
    let editor = repo.conflict_editor("@", "hello.txt").expect("load editor");
    let hunk = editor.hunks.first().expect("conflict hunk");

    let result = merge_result_use_source(&editor.result, hunk, MergeHunkSource::Right)
        .expect("use right hunk");
    repo.apply_conflict_editor("@", &editor, &result)
        .expect("apply hunk resolution");

    assert!(!repo.show("@").expect("show change").info.has_conflict);
    assert_eq!(
        repo.file_content("@", "hello.txt").unwrap(),
        hunk.right.trim_end()
    );
}

#[cfg(unix)]
#[test]
fn metadata_only_conflicts_do_not_offer_the_file_conflict_editor() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let script_path = repo_path.join("script.sh");
    let original = "#!/bin/sh\necho original\n";

    run_jj_in(&repo_path, &["new", "@"]);
    fs::write(&script_path, original).expect("write executable side");
    crate::filesystem::set_executable(&script_path, true).expect("make executable");
    run_jj_in(&repo_path, &["describe", "-m", "executable side"]);
    run_jj_in(&repo_path, &["bookmark", "create", "executable", "-r", "@"]);

    run_jj_in(&repo_path, &["new", "@-"]);
    fs::write(&script_path, original).expect("write non-executable side");
    crate::filesystem::set_executable(&script_path, false).expect("make non-executable");
    run_jj_in(&repo_path, &["describe", "-m", "non-executable side"]);
    run_jj_in(&repo_path, &["new", "executable", "@"]);

    let repo = Repo::open(&repo_path).expect("open executable conflict");
    let summary = repo.show_summary("@").expect("show executable conflict");
    let hunk = summary
        .diff
        .iter()
        .find(|hunk| hunk.path == "script.sh")
        .expect("executable conflict summary");
    let editor = repo
        .conflict_editor("@", "script.sh")
        .expect("load executable conflict");

    assert!(!hunk.supports_conflict_editor);
    assert!(!editor.is_text);
    assert!(editor.hunks.is_empty());
    assert!(
        repo.apply_conflict_editor("@", &editor, &editor.result)
            .is_err()
    );
}

#[test]
fn applies_after_the_same_change_was_amended() {
    let (temp_dir, repo) = conflict_fixture();
    let editor = repo.conflict_editor("@", "hello.txt").expect("load editor");
    repo.describe("@", "rewritten conflict")
        .expect("rewrite conflict");
    fs::write(temp_dir.path().join("repo/other.txt"), "unrelated\n").expect("write other");

    repo.apply_conflict_editor("@", &editor, "combined\n")
        .expect("apply after amend");

    assert!(!repo.show("@").expect("show change").info.has_conflict);
    assert_eq!(
        repo.file_content("@", "hello.txt").expect("file content"),
        "combined"
    );
}

#[test]
fn applies_to_a_divergent_working_copy_after_snapshots_rewrite_its_commit() {
    let (temp_dir, repo) = conflict_fixture();
    let repo_path = temp_dir.path().join("repo");
    drop(repo);

    let base_op = current_op_id(&repo_path);
    run_jj_in(&repo_path, &["describe", "-m", "first divergent version"]);
    run_jj_in(
        &repo_path,
        &[
            "--at-op",
            &base_op,
            "describe",
            "-m",
            "second divergent version",
        ],
    );

    let repo = Repo::open(&repo_path).expect("open divergent conflicted repo");
    let working_copy = repo.show("@").expect("show working copy").info;
    assert!(working_copy.is_divergent, "fixture must be divergent");
    let rev = working_copy.commit_id.id;

    fs::write(repo_path.join("other.txt"), "first snapshot\n").expect("write first edit");
    let editor = repo
        .conflict_editor(&rev, "hello.txt")
        .expect("load current divergent conflict");
    assert!(editor.is_working_copy);

    fs::write(repo_path.join("other.txt"), "second snapshot\n").expect("write second edit");
    repo.apply_conflict_editor(&rev, &editor, "combined\n")
        .expect("apply to the rewritten working copy");

    assert!(
        !repo
            .show("@")
            .expect("show resolved working copy")
            .info
            .has_conflict
    );
    assert_eq!(
        fs::read_to_string(repo_path.join("other.txt")).expect("read preserved edit"),
        "second snapshot\n"
    );
}

#[test]
fn rejects_a_conflict_edit_after_the_sides_changed_underneath() {
    let (temp_dir, repo) = conflict_fixture();
    let editor = repo.conflict_editor("@", "hello.txt").expect("load editor");
    let edited = editor.result.replace("right", "changed underneath");
    assert_ne!(edited, editor.result);
    fs::write(temp_dir.path().join("repo/hello.txt"), &edited).expect("write edited markers");

    let error = repo
        .apply_conflict_editor("@", &editor, "combined\n")
        .expect_err("a conflict rewritten underneath the editor must be stale");

    assert!(matches!(
        error,
        crate::CoreError::ConflictEditorStale { .. }
    ));
}

#[test]
fn rejects_a_conflict_edit_after_the_working_copy_moved_to_another_change() {
    let (_temp_dir, repo) = conflict_fixture();
    let editor = repo.conflict_editor("@", "hello.txt").expect("load editor");
    repo.new_change("@", "different change")
        .expect("move @ to a new change");

    let error = repo
        .apply_conflict_editor("@", &editor, "combined\n")
        .expect_err("stale edit should fail");

    assert!(matches!(
        error,
        crate::CoreError::ConflictEditorStale { .. }
    ));
}
