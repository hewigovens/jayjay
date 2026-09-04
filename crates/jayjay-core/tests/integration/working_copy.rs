use std::fs;

use jayjay_core::Repo;
use jj_test::{init_jj_repo, run_git, run_jj_in};

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
