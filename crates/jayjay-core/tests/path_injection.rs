//! Repository-controlled filenames must reach jj as literal paths, never as
//! options or fileset expressions. Each test plants a hostile filename and
//! asserts the mutation touches only that file (and launches nothing).

use std::fs;

use jayjay_core::Repo;
use jj_test::{init_jj_repo, run_jj_in};

/// Paths in a change's diff, for asserting exactly which files an action touched.
fn diff_paths(repo: &Repo, rev: &str) -> Vec<String> {
    repo.show(rev)
        .expect("show change")
        .diff
        .into_iter()
        .map(|hunk| hunk.path)
        .collect()
}

/// Finding 1 (medium): a file named like a jj option must not be parsed as one.
/// Without the fix, `--config=ui.diff-editor=...` makes split launch that editor
/// (here a missing binary), so the call would error instead of splitting the file.
#[test]
fn split_treats_option_shaped_filename_as_literal_path() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    let hostile = "--config=ui.diff-editor=jayjay-pwned-editor";
    fs::write(repo_path.join("keep.txt"), "keep\n").expect("write keep");
    fs::write(repo_path.join(hostile), "payload\n").expect("write hostile file");
    repo.refresh_working_copy().expect("snapshot");

    repo.split("@", &[hostile.to_owned()], "split out hostile", false)
        .expect("split must succeed and not launch an editor");

    // The split-out parent holds the named file; the child keeps the rest.
    let parent = diff_paths(&repo, "@-");
    let child = diff_paths(&repo, "@");
    assert!(
        parent.contains(&hostile.to_owned()),
        "split-out parent must hold the named file: {parent:?}"
    );
    assert!(
        !child.contains(&hostile.to_owned()) && child.contains(&"keep.txt".to_owned()),
        "child must keep the rest and not the split-out file: {child:?}"
    );
}

/// Finding 3: a file named `all()` must restore only itself, not every file.
#[test]
fn restore_working_copy_matches_only_the_named_fileset_filename() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    // Parent holds the baseline for both files.
    fs::write(repo_path.join("keep.txt"), "base\n").expect("write keep base");
    fs::write(repo_path.join("all()"), "base\n").expect("write all() base");
    repo.refresh_working_copy().expect("snapshot parent");

    // Working copy edits both.
    repo.new_change("@", "edits").expect("new change");
    fs::write(repo_path.join("keep.txt"), "edited\n").expect("edit keep");
    fs::write(repo_path.join("all()"), "edited\n").expect("edit all()");
    repo.refresh_working_copy().expect("snapshot edits");

    repo.restore_files("@", &["all()".to_owned()])
        .expect("restore all()");

    // all() reverted to baseline (out of the diff); keep.txt's edit survives.
    let paths = diff_paths(&repo, "@");
    assert!(
        paths.contains(&"keep.txt".to_owned()),
        "keep.txt must remain modified: {paths:?}"
    );
    assert!(
        !paths.contains(&"all()".to_owned()),
        "only all() should have been restored: {paths:?}"
    );
}

/// Finding 3: untrack must drop only the named file, not every tracked file.
#[test]
fn ignore_and_untrack_matches_only_the_named_fileset_filename() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    fs::write(repo_path.join("keep.txt"), "keep\n").expect("write keep");
    fs::write(repo_path.join("all()"), "payload\n").expect("write all()");
    repo.refresh_working_copy().expect("snapshot");

    repo.ignore_and_untrack(&["all()".to_owned()])
        .expect("ignore and untrack all()");

    let tracked =
        String::from_utf8_lossy(&run_jj_in(&repo_path, &["file", "list"]).stdout).into_owned();
    assert!(
        tracked.contains("keep.txt"),
        "keep.txt must stay tracked: {tracked:?}"
    );
    assert!(
        !tracked.contains("all()"),
        "only all() should have been untracked: {tracked:?}"
    );
}

/// Finding 3: move-to-working-copy (squash) must move only the named file.
#[test]
fn move_to_working_copy_matches_only_the_named_fileset_filename() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    // Source change adds both files.
    fs::write(repo_path.join("keep.txt"), "keep\n").expect("write keep");
    fs::write(repo_path.join("all()"), "payload\n").expect("write all()");
    repo.refresh_working_copy().expect("snapshot source");
    repo.describe("@", "source").expect("describe source");
    let source = repo
        .log("description(\"source\")")
        .expect("log source")
        .into_iter()
        .find(|c| c.description.trim() == "source")
        .expect("find source");

    // Empty working-copy child on top.
    repo.new_change("@", "child").expect("new child");

    repo.move_to_working_copy(&source.change_id, &["all()".to_owned()])
        .expect("move all() to working copy");

    // all() moved into @; keep.txt stays in the source change.
    let child = diff_paths(&repo, "@");
    assert!(
        child.contains(&"all()".to_owned()),
        "all() must move into the working copy: {child:?}"
    );
    let source_paths = diff_paths(&repo, &source.change_id);
    assert!(
        source_paths.contains(&"keep.txt".to_owned())
            && !source_paths.contains(&"all()".to_owned()),
        "only all() should have moved out of the source: {source_paths:?}"
    );
}

/// Finding 2: a newline in a filename must not inject extra .gitignore patterns.
#[test]
fn ignore_and_untrack_rejects_control_character_paths() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    let hostile = "evil\n*.pem".to_owned();
    let result = repo.ignore_and_untrack(&[hostile]);
    assert!(result.is_err(), "newline-bearing path must be rejected");

    // No pattern was injected (.gitignore is untouched / has no *.pem line).
    let gitignore = fs::read_to_string(repo_path.join(".gitignore")).unwrap_or_default();
    assert!(
        !gitignore.lines().any(|line| line.trim() == "*.pem"),
        "no injected ignore pattern: {gitignore:?}"
    );
}

/// A leading `!` filename must be ignored literally, not become a negation rule.
#[test]
fn ignore_and_untrack_escapes_negation_filename() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    fs::write(repo_path.join("!important.txt"), "x\n").expect("write file");
    repo.refresh_working_copy().expect("snapshot");

    repo.ignore_and_untrack(&["!important.txt".to_owned()])
        .expect("ignore negation-shaped filename");

    let gitignore = fs::read_to_string(repo_path.join(".gitignore")).expect("read .gitignore");
    assert!(
        gitignore
            .lines()
            .any(|line| line.trim() == "\\!important.txt"),
        "leading ! must be escaped: {gitignore:?}"
    );
}
