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
    assert!(mixed.diff.contains("Modified hello.txt:\n"));
    assert!(mixed.diff.contains("-hello from jayjay\n+real change\n"));
    assert!(!mixed.diff.contains("cargo lock body"));
    assert!(!mixed.diff.contains("npm lock body"));
    assert!(mixed.stat.contains("Cargo.lock | 1 +\n"));
    assert!(mixed.stat.contains("package-lock.json"));
    assert!(
        mixed
            .stat
            .ends_with("3 files changed, 3 insertions(+), 1 deletions(-)")
    );
}

#[test]
fn diff_excerpt_describes_a_content_free_rename_as_a_rename() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    run_jj_in(&repo_path, &["new"]);
    let repo = Repo::open(&repo_path).expect("open repo");
    fs::rename(repo_path.join("hello.txt"), repo_path.join("greeting.txt")).expect("rename");

    let excerpt = repo
        .diff_excerpt()
        .expect("excerpt")
        .expect("rename is a change");

    assert!(
        excerpt
            .diff
            .contains("Renamed hello.txt => greeting.txt:\n"),
        "{}",
        excerpt.diff
    );
    assert!(!excerpt.diff.contains("(binary)"), "{}", excerpt.diff);
}
