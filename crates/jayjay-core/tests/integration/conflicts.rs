use std::fs;

use jayjay_core::Repo;
use jj_test::{init_colocated, run_jj_in};

/// A conflicted path with a space must survive resolve_list intact and round-trip into resolve_use_ours.
#[test]
fn resolve_list_reports_full_path_with_spaces_and_resolves_it() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let repo_path = tmp.path().join("repo");
    init_colocated(&repo_path);
    run_jj_in(&repo_path, &["config", "set", "--repo", "user.name", "T"]);
    run_jj_in(
        &repo_path,
        &["config", "set", "--repo", "user.email", "t@e.com"],
    );

    let conflicted = "My Doc.txt";
    fs::write(repo_path.join(conflicted), "l1\nl2\nl3\n").expect("write base");
    run_jj_in(&repo_path, &["describe", "-m", "base"]);
    run_jj_in(&repo_path, &["bookmark", "create", "main", "-r", "@"]);

    run_jj_in(&repo_path, &["new", "-m", "left"]);
    fs::write(repo_path.join(conflicted), "l1\nLEFT\nl3\n").expect("write left");
    run_jj_in(&repo_path, &["bookmark", "set", "main", "-r", "@"]);

    run_jj_in(&repo_path, &["new", "-r", "main-", "-m", "right"]);
    fs::write(repo_path.join(conflicted), "l1\nRIGHT\nl3\n").expect("write right");
    run_jj_in(&repo_path, &["rebase", "-r", "@", "-d", "main"]);

    let repo = Repo::open(&repo_path).expect("open repo");
    let conflicts = repo.resolve_list("@").expect("list conflicts");
    assert_eq!(
        conflicts,
        vec![conflicted.to_owned()],
        "conflicted path with a space must be reported intact"
    );

    // The truncation bug would have passed "My" and missed the file.
    repo.resolve_use_ours("@", conflicted)
        .expect("resolve the space-containing path");
    let resolved = repo
        .file_content("@", conflicted)
        .expect("read resolved file");
    assert!(
        !resolved.contains("<<<<<<<"),
        "conflict markers should be gone after resolving ours: {resolved:?}"
    );
    assert!(
        resolved.contains("LEFT"),
        "resolve ours should keep the first side: {resolved:?}"
    );
}
