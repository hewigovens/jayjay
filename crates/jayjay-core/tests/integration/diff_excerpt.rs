use std::fs;

use jayjay_core::Repo;
use jj_test::{init_jj_repo, run_jj_in};

#[test]
fn diff_excerpt_leaves_lock_files_to_the_stat_unless_nothing_else_changed() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    run_jj_in(&repo_path, &["new"]);
    let repo = Repo::open(&repo_path).expect("open repo");
    assert!(repo.diff_excerpt().expect("empty excerpt").is_none());

    fs::create_dir(repo_path.join("web")).expect("create web dir");
    fs::write(repo_path.join("Cargo.lock"), "cargo lock body\n").expect("write Cargo.lock");
    fs::write(repo_path.join("web/package-lock.json"), "npm lock body\n").expect("write npm lock");
    let locks_only = repo
        .diff_excerpt()
        .expect("read excerpt")
        .expect("lock-only change");
    assert!(locks_only.diff.contains("cargo lock body"));

    fs::write(repo_path.join("hello.txt"), "real change\n").expect("edit hello.txt");
    let mixed = repo
        .diff_excerpt()
        .expect("read excerpt")
        .expect("mixed change");
    assert!(mixed.diff.contains("real change"));
    assert!(!mixed.diff.contains("cargo lock body"));
    assert!(!mixed.diff.contains("npm lock body"));
    assert!(mixed.stat.contains("Cargo.lock"));
    assert!(mixed.stat.contains("package-lock.json"));
}
