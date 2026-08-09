use std::fs;

use jj_test::{init_jj_repo, run_jj_in};

use crate::{CoreError, Repo};

#[test]
fn diff_marks_placeholder_prefixed_regular_text_as_editable() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    fs::write(repo_path.join("hello.txt"), "symlink -> literal text\n")
        .expect("write placeholder-prefixed text");
    let repo = Repo::open(&repo_path).expect("open repo");

    let hunk = repo.show_file("@", "hello.txt").expect("load file diff");

    assert!(hunk.supports_file_editor);
}

#[test]
fn edits_one_working_copy_file_and_preserves_unrelated_changes() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");
    let editor = repo
        .working_copy_file_editor("hello.txt")
        .expect("load file editor");

    fs::write(repo_path.join("unrelated.txt"), "keep me\n").expect("write unrelated file");
    repo.apply_working_copy_file_editor(&editor, "edited in JayJay\n")
        .expect("save file editor");

    assert_eq!(
        fs::read_to_string(repo_path.join("hello.txt")).expect("read edited file"),
        "edited in JayJay\n"
    );
    assert_eq!(
        fs::read_to_string(repo_path.join("unrelated.txt")).expect("read unrelated file"),
        "keep me\n"
    );
}

#[test]
fn rejects_an_edit_when_the_same_file_changed_externally() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");
    let editor = repo
        .working_copy_file_editor("hello.txt")
        .expect("load file editor");

    fs::write(repo_path.join("hello.txt"), "external edit\n").expect("write external edit");
    let error = repo
        .apply_working_copy_file_editor(&editor, "JayJay edit\n")
        .expect_err("stale edit should fail");

    assert!(matches!(error, CoreError::FileEditorStale { .. }));
    assert_eq!(
        fs::read_to_string(repo_path.join("hello.txt")).expect("read external edit"),
        "external edit\n"
    );
}

#[test]
fn rejects_an_edit_after_switching_the_working_copy_change() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");
    let editor = repo
        .working_copy_file_editor("hello.txt")
        .expect("load file editor");

    run_jj_in(&repo_path, &["new", "@"]);
    let error = repo
        .apply_working_copy_file_editor(&editor, "wrong change\n")
        .expect_err("different working copy should fail");

    assert!(matches!(error, CoreError::FileEditorStale { .. }));
}

#[test]
fn refuses_binary_files() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    fs::write(repo_path.join("binary.dat"), b"prefix\0suffix").expect("write binary file");
    let repo = Repo::open(&repo_path).expect("open repo");

    let error = repo
        .working_copy_file_editor("binary.dat")
        .expect_err("binary editor should fail");

    assert!(error.to_string().contains("binary files cannot be edited"));
    assert!(
        !repo
            .show_file("@", "binary.dat")
            .expect("load binary diff")
            .supports_file_editor
    );
}
