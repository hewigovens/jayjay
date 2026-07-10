use std::fs;
use std::path::PathBuf;

use jayjay_core::{CoreError, DiffEditDestination, Repo};
use jj_test::{
    init_colocated, init_jj_repo, run_jj_in, selection_for_lines, setup_source_change_with_child,
    whole_file_selection,
};
use tempfile::TempDir;

/// Working copy modifies hello.txt on top of the initial change, so selections carry a real parent side.
fn setup_modified_tracked_file() -> (TempDir, PathBuf, Repo) {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    repo.new_change("@", "modify hello")
        .expect("new change on top of initial");
    fs::write(
        repo_path.join("hello.txt"),
        "hello from jayjay\nsecond line\n",
    )
    .expect("modify tracked file");
    repo.refresh_working_copy()
        .expect("snapshot working copy changes");

    (temp_dir, repo_path, repo)
}

#[test]
fn diffedit_rejects_selection_when_source_changed_since_render() {
    // Regression: a stale selection must be rejected, not silently applied to reconstruct the file from outdated content.
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    fs::write(
        repo_path.join("notes.md"),
        "first line\nremove me\nlast line\n",
    )
    .expect("write new file");
    repo.refresh_working_copy()
        .expect("snapshot working copy changes");

    let selection = selection_for_lines(&repo, "@", "notes.md", &[(2, 2)]);

    let changed = "first line\nremove me\nlast line\nappended after render\n";
    fs::write(repo_path.join("notes.md"), changed).expect("simulate edit after diff rendered");

    let err = repo
        .apply_diff_selection(
            "@",
            DiffEditDestination::RemoveFromSource,
            &[selection],
            "",
            false,
        )
        .expect_err("stale selection must be rejected");
    assert!(
        matches!(err, CoreError::DiffSelectionStale { .. }),
        "expected DiffSelectionStale, got {err:?}"
    );

    assert_eq!(
        fs::read_to_string(repo_path.join("notes.md")).expect("read notes.md"),
        changed,
        "on-disk content must be untouched when the guard rejects a stale selection"
    );
}

#[test]
fn diffedit_rejects_selection_when_parent_content_changed_since_render() {
    // Regression: partition_file_selection rebuilds unselected lines from old_content, so a parent rewritten after render must be rejected even when the new side still matches.
    let (_temp_dir, repo_path, repo) = setup_modified_tracked_file();

    let mut selection = whole_file_selection(&repo, "@", "hello.txt");
    assert_eq!(
        selection.old_content.as_deref(),
        Some("hello from jayjay\n"),
        "sanity: selection rendered against the current parent"
    );
    // Inject staleness directly: the parent was rewritten after render while the file's new bytes stayed identical.
    selection.old_content = Some("hello from a rewritten parent\n".to_owned());

    let err = repo
        .apply_diff_selection(
            "@",
            DiffEditDestination::RemoveFromSource,
            &[selection],
            "",
            false,
        )
        .expect_err("stale parent side must be rejected");
    assert!(
        matches!(err, CoreError::DiffSelectionStale { .. }),
        "expected DiffSelectionStale, got {err:?}"
    );

    assert_eq!(
        fs::read_to_string(repo_path.join("hello.txt")).expect("read hello.txt"),
        "hello from jayjay\nsecond line\n",
        "on-disk content must be untouched when the guard rejects a stale parent side"
    );
}

#[test]
fn diffedit_rejects_stale_parent_side_for_every_destination() {
    for destination in [
        DiffEditDestination::RemoveFromSource,
        DiffEditDestination::MoveToWorkingCopy,
        DiffEditDestination::NewChild,
        DiffEditDestination::NewParallel,
    ] {
        let (_temp_dir, _repo_path, repo) = setup_source_change_with_child();
        let mut selection = whole_file_selection(&repo, "@-", "notes.md");
        assert!(
            selection.old_content.is_none(),
            "sanity: added file has no parent side"
        );
        // Inject staleness directly: render claimed a parent side the current parent tree does not have.
        selection.old_content = Some("phantom parent content\n".to_owned());

        let err = repo
            .apply_diff_selection("@-", destination, &[selection], "selected", false)
            .expect_err("stale parent side must be rejected");
        assert!(
            matches!(err, CoreError::DiffSelectionStale { .. }),
            "expected DiffSelectionStale for {destination:?}, got {err:?}"
        );

        let source = repo.show("@-").expect("show source change");
        let notes = source
            .diff
            .iter()
            .find(|hunk| hunk.path == "notes.md")
            .unwrap_or_else(|| panic!("source must be untouched for {destination:?}"));
        assert_eq!(
            notes.new.content.as_deref(),
            Some("# moved content\n\nline for diffedit\n"),
            "source content must be untouched for {destination:?}"
        );
    }
}

#[test]
fn diffedit_modified_file_with_current_parent_passes_guard() {
    let (_temp_dir, repo_path, repo) = setup_modified_tracked_file();

    let selection = whole_file_selection(&repo, "@", "hello.txt");
    repo.apply_diff_selection(
        "@",
        DiffEditDestination::RemoveFromSource,
        &[selection],
        "",
        false,
    )
    .expect("selection matching both sides must pass the guard");

    assert_eq!(
        fs::read_to_string(repo_path.join("hello.txt")).expect("read hello.txt"),
        "hello from jayjay\n",
        "removing the whole modification restores the parent content"
    );
}

/// Working copy sits on a conflicted parent and resolves conflicted.txt, so the rendered old side is materialized marker text.
fn setup_conflicted_parent_resolved_in_working_copy() -> (TempDir, PathBuf, Repo) {
    let temp_dir = tempfile::tempdir().expect("create tempdir");
    let repo_path = temp_dir.path().join("repo");
    init_colocated(&repo_path);
    run_jj_in(&repo_path, &["config", "set", "--repo", "user.name", "T"]);
    run_jj_in(
        &repo_path,
        &["config", "set", "--repo", "user.email", "t@e.com"],
    );

    fs::write(repo_path.join("conflicted.txt"), "l1\nl2\nl3\n").expect("write base");
    run_jj_in(&repo_path, &["describe", "-m", "base"]);
    run_jj_in(&repo_path, &["bookmark", "create", "main", "-r", "@"]);

    run_jj_in(&repo_path, &["new", "-m", "left"]);
    fs::write(repo_path.join("conflicted.txt"), "l1\nLEFT\nl3\n").expect("write left");
    run_jj_in(&repo_path, &["bookmark", "set", "main", "-r", "@"]);

    run_jj_in(&repo_path, &["new", "-r", "main-", "-m", "right"]);
    fs::write(repo_path.join("conflicted.txt"), "l1\nRIGHT\nl3\n").expect("write right");
    run_jj_in(&repo_path, &["rebase", "-r", "@", "-d", "main"]);

    run_jj_in(&repo_path, &["new", "-m", "resolve conflict"]);
    fs::write(repo_path.join("conflicted.txt"), "l1\nRESOLVED\nl3\n").expect("resolve conflict");

    let repo = Repo::open(&repo_path).expect("open repo");
    repo.refresh_working_copy()
        .expect("snapshot conflict resolution");
    (temp_dir, repo_path, repo)
}

#[test]
fn diffedit_rejects_selection_whose_parent_side_is_a_conflict() {
    // Regression: the old side of this diff is materialized conflict markers; partitioning would write those markers into the rewritten tree as literal resolved content, silently destroying the conflict structure.
    let (_temp_dir, repo_path, repo) = setup_conflicted_parent_resolved_in_working_copy();

    let selection = whole_file_selection(&repo, "@", "conflicted.txt");
    assert!(
        selection
            .old_content
            .as_deref()
            .is_some_and(|old| old.contains("<<<<<<<")),
        "sanity: the rendered parent side must be marker text, got {:?}",
        selection.old_content
    );

    let err = repo
        .apply_diff_selection(
            "@",
            DiffEditDestination::RemoveFromSource,
            &[selection],
            "",
            false,
        )
        .expect_err("conflicted parent side must be rejected");
    assert!(
        matches!(&err, CoreError::Internal { message } if message.contains("conflicted")),
        "expected conflicted-file rejection, got {err:?}"
    );

    assert_eq!(
        fs::read_to_string(repo_path.join("conflicted.txt")).expect("read conflicted.txt"),
        "l1\nRESOLVED\nl3\n",
        "the resolution must be untouched when the guard rejects a conflicted parent"
    );
}

#[test]
fn diffedit_rejects_selection_on_a_conflicted_source_commit() {
    // The conflicted "right" commit renders its new side as marker text; the guard must refuse it before any tree write instead of failing deep in the rewrite.
    let (_temp_dir, _repo_path, repo) = setup_conflicted_parent_resolved_in_working_copy();

    let selection = whole_file_selection(&repo, "@-", "conflicted.txt");
    let err = repo
        .apply_diff_selection(
            "@-",
            DiffEditDestination::RemoveFromSource,
            &[selection],
            "",
            false,
        )
        .expect_err("conflicted source side must be rejected");
    assert!(
        matches!(&err, CoreError::Internal { message } if message.contains("conflicted")),
        "expected conflicted-file rejection, got {err:?}"
    );
}

#[test]
fn diffedit_removed_file_selection_passes_guard_and_restores_file() {
    // A Removed-file selection has new_content None; the guard must treat absent-in-source as matching while still checking the old side against the parent.
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    repo.new_change("@", "delete hello")
        .expect("new change on top of initial");
    fs::remove_file(repo_path.join("hello.txt")).expect("delete tracked file");
    repo.refresh_working_copy()
        .expect("snapshot working copy changes");

    let selection = whole_file_selection(&repo, "@", "hello.txt");
    repo.apply_diff_selection(
        "@",
        DiffEditDestination::RemoveFromSource,
        &[selection],
        "",
        false,
    )
    .expect("removing the deletion restores the file");

    assert_eq!(
        fs::read_to_string(repo_path.join("hello.txt")).expect("read restored file"),
        "hello from jayjay\n"
    );
    let current = repo.show("@").expect("show working copy");
    assert!(
        current.diff.iter().all(|hunk| hunk.path != "hello.txt"),
        "the deletion should no longer be part of the source change"
    );
}
