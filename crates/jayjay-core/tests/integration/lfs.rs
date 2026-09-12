//! LFS diff-hiding must follow real LFS registration, not repository-controlled
//! `.gitattributes` — otherwise a malicious repo could mark a source file as LFS
//! to hide its real diff from review.

use std::fs;
use std::process::Command;

use jayjay_core::Repo;
use jj_test::{init_jj_repo, run_git, run_jj_in};

fn git_lfs_available() -> bool {
    Command::new("git")
        .args(["lfs", "version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// A `.gitattributes filter=lfs` line alone (no real LFS object) must NOT cause
/// the path to be treated as LFS — the lying attribute can't fake a stored pointer.
#[test]
fn attribute_only_lfs_is_not_reported() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    fs::write(repo_path.join("source.txt"), "real source line\n").expect("write source");
    fs::write(repo_path.join(".gitattributes"), "source.txt filter=lfs\n").expect("write attrs");

    // The attribute really is set (so the filter, not a missing attribute, is what matters).
    let attr = String::from_utf8_lossy(
        &run_git(&repo_path, &["check-attr", "filter", "--", "source.txt"]).stdout,
    )
    .into_owned();
    assert!(
        attr.contains("filter: lfs"),
        "attribute should mark lfs: {attr}"
    );

    assert!(
        repo.git_lfs_paths(&["source.txt".to_owned()])
            .expect("git_lfs_paths")
            .is_empty(),
        "a non-registered file must not be hidden as LFS"
    );
}

/// A genuinely LFS-tracked binary must still be reported, even though its working
/// copy holds the smudged bytes rather than a pointer.
#[test]
fn genuine_lfs_object_follows_attributes_across_refreshes() {
    if !git_lfs_available() {
        eprintln!("skipping: git-lfs not installed");
        return;
    }

    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    // Asked before tracking: this is the cached answer that must not survive the commit.
    assert!(
        repo.git_lfs_paths(&["asset.bin".to_owned()])
            .expect("git_lfs_paths")
            .is_empty()
    );

    run_git(&repo_path, &["config", "user.email", "test@example.com"]);
    run_git(&repo_path, &["config", "user.name", "Test User"]);
    run_git(&repo_path, &["lfs", "install", "--local"]);
    run_git(&repo_path, &["lfs", "track", "*.bin"]);
    fs::write(repo_path.join("asset.bin"), vec![0u8, 1, 2, 3, 4, 5, 6, 7]).expect("write asset");
    run_git(&repo_path, &["add", ".gitattributes", "asset.bin"]);
    run_git(&repo_path, &["commit", "-m", "add lfs asset"]);
    repo.refresh_working_copy().expect("snapshot working copy");

    assert_eq!(
        repo.git_lfs_paths(&["asset.bin".to_owned()])
            .expect("git_lfs_paths"),
        vec!["asset.bin".to_owned()],
        "a real LFS object must still be reported"
    );

    let operation = || {
        run_jj_in(
            &repo_path,
            &[
                "--ignore-working-copy",
                "op",
                "log",
                "--no-graph",
                "--limit",
                "1",
                "-T",
                "id",
            ],
        )
        .stdout
    };
    let before = operation();
    let info_attributes = repo_path.join(".git/info/attributes");
    for (attributes, expected) in [
        ("asset.bin -filter\n", vec![]),
        ("asset.bin filter=lfs\n", vec!["asset.bin".to_owned()]),
    ] {
        fs::write(&info_attributes, attributes).expect("write local attributes");
        repo.refresh_working_copy()
            .expect("refresh local attributes");
        assert_eq!(operation(), before);
        assert_eq!(
            repo.git_lfs_paths(&["asset.bin".to_owned()])
                .expect("git_lfs_paths"),
            expected,
        );
    }
}
