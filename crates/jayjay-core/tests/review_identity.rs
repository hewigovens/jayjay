//! `review_identity` is the content key review marks hang on: it must stay
//! byte-identical when both diff sides are unchanged and change when either
//! moves. Exercised against a real repo since it derives from jj blob IDs.

use std::fs;

use jayjay_core::Repo;
use jj_test::{init_jj_repo, run_jj};

/// Read the `review_identity` of `path` in change `rev`.
fn identity(repo: &Repo, rev: &str, path: &str) -> String {
    repo.show(rev)
        .unwrap_or_else(|e| panic!("show {rev}: {e}"))
        .diff
        .into_iter()
        .find(|hunk| hunk.path == path)
        .unwrap_or_else(|| panic!("no diff for {path} in {rev}"))
        .review_identity
}

/// Build base -> {p1, p2} siblings sharing `feature.txt`, then change C on p1
/// that edits it. Returns C's change id so it stays findable across rebases.
fn setup() -> (tempfile::TempDir, Repo, String) {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo_str = repo_path.to_str().expect("repo path utf-8");

    // base: shared starting point with feature.txt.
    fs::write(repo_path.join("feature.txt"), "base line\n").expect("write base feature");
    run_jj(&["-R", repo_str, "describe", "-m", "base"]);
    run_jj(&["-R", repo_str, "bookmark", "create", "base", "-r", "@"]);

    // p1: sibling holding the identical feature.txt that C diffs against.
    run_jj(&["-R", repo_str, "new", "base", "-m", "p1"]);
    run_jj(&["-R", repo_str, "bookmark", "create", "p1", "-r", "@"]);

    // p2: sibling with byte-identical feature.txt — only its commit differs.
    run_jj(&["-R", repo_str, "new", "base", "-m", "p2"]);
    run_jj(&["-R", repo_str, "bookmark", "create", "p2", "-r", "@"]);

    // C: edits feature.txt on top of p1.
    run_jj(&["-R", repo_str, "new", "p1", "-m", "C"]);
    fs::write(repo_path.join("feature.txt"), "base line\nedited by C\n").expect("write C feature");

    let repo = Repo::open(&repo_path).expect("open repo");
    repo.refresh_working_copy().expect("snapshot C");
    let change_id = repo.show("@").expect("show C").info.change_id;

    (temp_dir, repo, change_id)
}

#[test]
fn rebase_onto_identical_base_preserves_identity() {
    let (temp_dir, repo, c) = setup();
    let repo_str = temp_dir.path().join("repo");
    let repo_str = repo_str.to_str().unwrap();

    let before = identity(&repo, &c, "feature.txt");
    assert!(!before.is_empty(), "identity must be recorded");

    // p2's feature.txt is byte-identical, so a rebase onto it leaves both diff sides unchanged.
    run_jj(&["-R", repo_str, "rebase", "-r", &c, "-d", "p2"]);
    let repo = Repo::open(&temp_dir.path().join("repo")).expect("reopen after rebase");
    let after = identity(&repo, &c, "feature.txt");

    assert_eq!(
        before, after,
        "review mark must survive a rebase that does not change content"
    );
}

#[test]
fn editing_content_changes_identity() {
    let (temp_dir, repo, c) = setup();
    let repo_path = temp_dir.path().join("repo");

    let before = identity(&repo, &c, "feature.txt");

    // Change only the `after` side: C's own content of feature.txt.
    fs::write(repo_path.join("feature.txt"), "base line\nrewritten by C\n")
        .expect("rewrite C feature");
    repo.refresh_working_copy().expect("snapshot edit");
    let after = identity(&repo, &c, "feature.txt");

    assert_ne!(
        before, after,
        "editing the file's content must invalidate the review mark"
    );
}

#[test]
fn rebase_onto_different_base_changes_identity() {
    let (temp_dir, repo, c) = setup();
    let repo_path = temp_dir.path().join("repo");
    let repo_str = repo_path.to_str().unwrap();

    let before = identity(&repo, &c, "feature.txt");

    // p3 holds a different feature.txt, so rebasing C onto it moves the diff's
    // `before` side; the key must change since the reviewed base changed.
    run_jj(&["-R", repo_str, "new", "base", "-m", "p3"]);
    run_jj(&["-R", repo_str, "bookmark", "create", "p3", "-r", "@"]);
    fs::write(repo_path.join("feature.txt"), "different base line\n").expect("write p3 feature");
    let repo = Repo::open(&repo_path).expect("reopen for p3 snapshot");
    repo.refresh_working_copy().expect("snapshot p3");

    run_jj(&["-R", repo_str, "rebase", "-r", &c, "-d", "p3"]);
    let repo = Repo::open(&repo_path).expect("reopen after rebase");
    let after = identity(&repo, &c, "feature.txt");

    assert_ne!(
        before, after,
        "rebasing onto a different base content must invalidate the review mark"
    );
}
