use std::fs;
use std::path::PathBuf;

use jayjay_core::{JayError, Repo};
use jj_test::{changed_paths, git_stdout, init_jj_repo, run_git, run_jj_in};
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

/// The stale side edits `top` while the other workspace rebases `top` onto `fix`, which rewrites hello.txt.
fn stale_workspace(stale_default: bool) -> (TempDir, PathBuf, PathBuf) {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    run_jj_in(&repo_path, &["new", "-m", "top"]);
    fs::write(repo_path.join("top.txt"), "top\n").expect("write top.txt");
    run_jj_in(&repo_path, &["new", "@-", "-m", "fix"]);
    fs::write(repo_path.join("hello.txt"), "fixed\n").expect("write fix");
    let second = temp_dir.path().join("second");
    run_jj_in(
        &repo_path,
        &[
            "workspace",
            "add",
            "--name",
            "second",
            "-r",
            "root()",
            second.to_str().expect("utf8 path"),
        ],
    );
    let (stale, other) = if stale_default {
        (&repo_path, &second)
    } else {
        (&second, &repo_path)
    };
    run_jj_in(stale, &["edit", "subject(top)"]);
    run_jj_in(
        other,
        &["rebase", "-r", "subject(top)", "-d", "subject(fix)"],
    );
    (temp_dir, repo_path, second)
}

#[test]
fn a_stale_working_copy_is_refused_instead_of_reverting_the_rewrite() {
    let (_temp_dir, _repo_path, second) = stale_workspace(false);
    let repo = Repo::open(&second).expect("open second workspace");

    for (action, result) in [
        ("describe", repo.describe("@", "edited")),
        ("create bookmark", repo.create_bookmark("marker", "@")),
        ("refresh", repo.refresh_working_copy()),
    ] {
        assert!(
            matches!(result, Err(JayError::WorkingCopyStale)),
            "{action} on a stale working copy: {result:?}"
        );
    }
    assert_eq!(
        changed_paths(&repo, "subject(top)"),
        ["top.txt"],
        "the stale files must not be recorded over the rebased change"
    );
}

#[test]
fn updating_a_stale_workspace_checks_out_the_rewrite_and_keeps_edits_made_meanwhile() {
    let (_temp_dir, _repo_path, second) = stale_workspace(false);
    let repo = Repo::open(&second).expect("open second workspace");
    fs::write(second.join("hello.txt"), "edited while stale\n").expect("edit hello.txt");

    repo.update_stale_workspace()
        .expect("update stale workspace");
    repo.refresh_working_copy()
        .expect("snapshot the updated working copy");

    assert_eq!(
        fs::read_to_string(second.join("hello.txt")).expect("read hello.txt"),
        "fixed\n"
    );
    assert_eq!(changed_paths(&repo, "@"), ["top.txt"]);
    let edited = repo
        .log("subject(top) ~ @")
        .expect("log the other version of top");
    assert_eq!(
        edited.len(),
        1,
        "the edit is kept as its own version of top"
    );
    assert_eq!(
        changed_paths(&repo, &edited[0].commit_id.id),
        ["hello.txt", "top.txt"]
    );
}

#[test]
fn updating_a_workspace_whose_operation_is_gone_checks_out_a_recovery_commit() {
    let (_temp_dir, repo_path, second) = stale_workspace(false);
    run_jj_in(&repo_path, &["op", "abandon", "..@-"]);
    run_jj_in(&repo_path, &["util", "gc", "--expire=now"]);
    let repo = Repo::open(&second).expect("open second workspace");
    assert!(matches!(
        repo.refresh_working_copy(),
        Err(JayError::WorkingCopyStale)
    ));

    repo.update_stale_workspace().expect("recover workspace");
    repo.refresh_working_copy()
        .expect("snapshot the recovered working copy");

    assert_eq!(changed_paths(&repo, "subject(top)"), ["top.txt"]);
    let top = &repo.log("subject(top)").expect("log top")[0];
    assert_eq!(
        repo.log("@").expect("log @")[0].parents,
        std::slice::from_ref(&top.commit_id.id)
    );
    assert_eq!(
        changed_paths(&repo, "@"),
        ["hello.txt"],
        "the stale file stays as an edit on the recovery commit"
    );
}

#[test]
fn updating_a_stale_colocated_workspace_moves_git_head_to_the_new_parent() {
    for operation_gone in [false, true] {
        let (_temp_dir, repo_path, second) = stale_workspace(true);
        if operation_gone {
            run_jj_in(&second, &["op", "abandon", "..@-"]);
            run_jj_in(&second, &["util", "gc", "--expire=now"]);
        }
        let repo = Repo::open(&repo_path).expect("open default workspace");

        repo.update_stale_workspace()
            .expect("update stale workspace");

        let parent = &repo.log("@-").expect("log parent")[0].commit_id.id;
        assert_eq!(
            git_stdout(&repo_path, &["rev-parse", "HEAD"]),
            *parent,
            "operation gone: {operation_gone}"
        );
        if !operation_gone {
            assert!(
                !git_stdout(&repo_path, &["status", "--short"]).contains("hello.txt"),
                "the inherited fix must not look like an uncommitted edit"
            );
        }
    }
}
