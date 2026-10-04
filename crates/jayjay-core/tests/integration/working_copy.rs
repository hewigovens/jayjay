use std::fs;
use std::path::{Path, PathBuf};

use jayjay_core::Repo;
use jj_test::{init_jj_repo, run_git, run_jj_in};
use tempfile::TempDir;

#[test]
fn refresh_working_copy_rebases_a_conflicting_descendant_from_a_small_stack() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");

    fs::write(repo_path.join("hello.txt"), "base\nshared\nend\n").expect("write base");
    run_jj_in(&repo_path, &["describe", "-m", "base"]);
    run_jj_in(&repo_path, &["new", "-m", "descendant"]);
    fs::write(repo_path.join("hello.txt"), "base\ndescendant edit\nend\n")
        .expect("write descendant");
    run_jj_in(&repo_path, &["new", "-m", "tip"]);
    run_jj_in(&repo_path, &["edit", "@--"]);
    fs::write(
        repo_path.join("hello.txt"),
        "base\nworking-copy edit\nend\n",
    )
    .expect("rewrite ancestor");

    let repo = Repo::open(&repo_path).expect("open ancestor working copy");
    let repo = std::thread::Builder::new()
        .name("small-stack-caller".to_owned())
        .stack_size(512 * 1024)
        .spawn(move || {
            repo.refresh_working_copy()
                .expect("snapshot and rebase descendant");
            repo
        })
        .expect("spawn small-stack caller")
        .join()
        .expect("small-stack caller should not overflow");

    let descendant = repo
        .log("all()")
        .expect("load rebased changes")
        .into_iter()
        .find(|change| change.description.trim() == "descendant")
        .expect("rebased descendant");
    assert!(
        descendant.has_conflict,
        "the fixture must exercise jj's conflicting descendant merge"
    );
}

#[test]
fn refresh_working_copy_respects_git_excludes_file() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let excludes_path = temp_dir.path().join("global-ignore");
    fs::write(&excludes_path, ".claude/\n").expect("write excludes file");
    run_git(
        &repo_path,
        &[
            "config",
            "core.excludesFile",
            excludes_path.to_str().expect("excludes path utf-8"),
        ],
    );

    fs::create_dir(repo_path.join(".claude")).expect("create ignored dir");
    fs::write(repo_path.join(".claude/settings.json"), "{}\n").expect("write ignored file");
    fs::write(repo_path.join("visible.txt"), "visible\n").expect("write visible file");

    let repo = Repo::open(&repo_path).expect("open repo");
    assert!(
        !repo
            .has_unignored_working_copy_paths(&[repo_path
                .join(".claude/settings.json")
                .display()
                .to_string()])
            .expect("check ignored path"),
        "global git excludes should suppress ignored working-copy events"
    );
    assert!(
        !repo
            .has_unignored_working_copy_paths(&[repo_path
                .join(".jj/repo/op_heads/heads")
                .display()
                .to_string()])
            .expect("check jj metadata path"),
        "jj's own metadata writes must not look like working-copy edits"
    );
    assert!(
        repo.has_unignored_working_copy_paths(&[repo_path
            .join("visible.txt")
            .display()
            .to_string()])
            .expect("check visible path"),
        "ordinary new files should still trigger working-copy events"
    );
    repo.refresh_working_copy()
        .expect("snapshot working copy changes");

    let current = repo.show("@").expect("show refreshed working copy");
    assert!(
        current.diff.iter().any(|hunk| hunk.path == "visible.txt"),
        "ordinary new files should still be auto-tracked"
    );
    assert!(
        current
            .diff
            .iter()
            .all(|hunk| !hunk.path.starts_with(".claude/")),
        "git excludes file should prevent .claude files from being auto-tracked"
    );
}

#[test]
fn working_copy_event_filter_respects_local_gitignore() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    fs::write(repo_path.join(".gitignore"), "scratch/\n").expect("write gitignore");
    fs::create_dir(repo_path.join("scratch")).expect("create ignored dir");
    fs::write(repo_path.join("scratch/file.txt"), "ignored\n").expect("write ignored file");
    fs::write(repo_path.join("visible.txt"), "visible\n").expect("write visible file");

    let repo = Repo::open(&repo_path).expect("open repo");
    assert!(
        !repo
            .has_unignored_working_copy_paths(&[repo_path
                .join("scratch/file.txt")
                .display()
                .to_string()])
            .expect("check ignored path"),
        "local .gitignore should suppress ignored working-copy events"
    );
    assert!(
        repo.has_unignored_working_copy_paths(&[
            repo_path.join("scratch/file.txt").display().to_string(),
            repo_path.join("visible.txt").display().to_string(),
        ])
        .expect("check mixed paths"),
        "a batch with any unignored path should trigger a working-copy event"
    );
}

#[test]
fn working_copy_event_filter_preserves_tracked_ignored_paths() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    fs::create_dir(repo_path.join("tracked")).expect("create tracked dir");
    fs::write(repo_path.join("tracked/file.txt"), "tracked\n").expect("write tracked file");

    let repo = Repo::open(&repo_path).expect("open repo");
    repo.refresh_working_copy().expect("track file");

    fs::write(repo_path.join(".gitignore"), "tracked/\n").expect("write gitignore");
    fs::write(repo_path.join("tracked/file.txt"), "changed\n").expect("change tracked file");
    fs::write(repo_path.join("tracked/new.txt"), "ignored\n").expect("write ignored file");

    assert!(
        repo.has_unignored_working_copy_paths(&[repo_path
            .join("tracked/file.txt")
            .display()
            .to_string()])
            .expect("check tracked ignored path"),
        "tracked paths should still trigger working-copy events even when ignored"
    );
    assert!(
        !repo
            .has_unignored_working_copy_paths(&[repo_path
                .join("tracked/new.txt")
                .display()
                .to_string()])
            .expect("check untracked ignored path"),
        "untracked ignored paths should not trigger working-copy events"
    );
}
#[test]
fn working_copy_is_large_tracks_tree_state_size() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    assert!(
        !repo.working_copy_is_large(),
        "small working copy should not be flagged large"
    );

    // Only stats the file, so padding past the threshold flips the flag without a huge repo.
    let tree_state = repo_path.join(".jj/working_copy/tree_state");
    let padding = vec![0u8; 16 * 1024 * 1024];
    fs::write(&tree_state, padding).expect("pad tree_state");
    assert!(
        repo.working_copy_is_large(),
        "an oversized tree_state should be flagged large"
    );
}

#[test]
fn snapshot_adds_new_files_to_the_colocated_git_index() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");
    fs::write(repo_path.join("added.txt"), "new\n").expect("write added");

    repo.refresh_working_copy().expect("snapshot");

    let listed = run_git(&repo_path, &["ls-files", "--", "added.txt"]).stdout;
    assert_eq!(
        listed, b"added.txt\n",
        "git must see the file jj started tracking"
    );
}

/// `ws2` sits on `top` and `viewer` on `root()` while the default workspace inserts `fix` below `top`, which leaves only `ws2` stale.
fn rebase_under_sibling_workspaces() -> (TempDir, PathBuf, PathBuf) {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let ws2 = temp_dir.path().join("repo-ws2");
    let viewer = temp_dir.path().join("repo-viewer");
    run_jj_in(&repo_path, &["describe", "-m", "base"]);
    run_jj_in(&repo_path, &["new", "-m", "top"]);
    fs::write(repo_path.join("top.txt"), "top\n").expect("write top");
    run_jj_in(&repo_path, &["new"]);
    let ws2_root = ws2.to_str().expect("utf8 ws2");
    run_jj_in(
        &repo_path,
        &[
            "workspace",
            "add",
            "--name",
            "ws2",
            "-r",
            "subject(top)",
            ws2_root,
        ],
    );
    run_jj_in(&ws2, &["edit", "subject(top)"]);
    let viewer_root = viewer.to_str().expect("utf8 viewer");
    run_jj_in(
        &repo_path,
        &[
            "workspace",
            "add",
            "--name",
            "viewer",
            "-r",
            "root()",
            viewer_root,
        ],
    );
    run_jj_in(&repo_path, &["new", "subject(base)", "-m", "fix"]);
    fs::write(repo_path.join("fix.txt"), "fix\n").expect("write fix");
    run_jj_in(&repo_path, &["new"]);
    run_jj_in(
        &repo_path,
        &["rebase", "-s", "subject(top)", "-d", "subject(fix)"],
    );
    (temp_dir, ws2, viewer)
}

fn files_in_top(repo_path: &Path) -> String {
    let output = run_jj_in(
        repo_path,
        &[
            "--ignore-working-copy",
            "file",
            "list",
            "-r",
            "subject(top)",
        ],
    );
    String::from_utf8(output.stdout).expect("utf8 file list")
}

#[test]
fn refresh_refuses_a_stale_working_copy_instead_of_reverting_the_rewrite() {
    let (temp_dir, ws2, _viewer) = rebase_under_sibling_workspaces();
    let repo = Repo::open(&ws2).expect("open stale workspace");

    let error = repo
        .refresh_working_copy()
        .expect_err("a stale working copy must not be snapshotted");

    assert!(error.to_string().contains("update-stale"), "{error}");
    assert!(
        files_in_top(&temp_dir.path().join("repo")).contains("fix.txt"),
        "the snapshot reverted the rebased-in fix"
    );
}

#[test]
fn mutation_without_a_refresh_leaves_a_stale_working_copy_stale() {
    let (temp_dir, ws2, _viewer) = rebase_under_sibling_workspaces();
    let repo = Repo::open(&ws2).expect("open stale workspace");

    let error = repo
        .create_bookmark("probe", "@")
        .expect_err("a mutation must not finish a stale working copy");

    assert!(error.to_string().contains("update-stale"), "{error}");
    repo.refresh_working_copy()
        .expect_err("the working copy must still be stale");
    assert!(files_in_top(&temp_dir.path().join("repo")).contains("fix.txt"));
}

#[test]
fn refresh_snapshots_a_workspace_left_current_by_a_rewrite_elsewhere() {
    let (temp_dir, _ws2, viewer) = rebase_under_sibling_workspaces();
    let repo = Repo::open(&viewer).expect("open viewer workspace");
    fs::write(viewer.join("viewer.txt"), "edit\n").expect("write viewer edit");

    repo.refresh_working_copy()
        .expect("snapshot a workspace the rebase did not touch");

    assert!(
        repo.show_summary("@")
            .expect("summary")
            .diff
            .iter()
            .any(|hunk| hunk.path == "viewer.txt")
    );
    assert!(files_in_top(&temp_dir.path().join("repo")).contains("fix.txt"));
}
