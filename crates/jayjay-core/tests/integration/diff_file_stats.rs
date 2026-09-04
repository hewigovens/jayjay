use std::fs;

use jayjay_core::Repo;
use jj_test::{FormatFixture, init_jj_repo, run_jj_in};

#[test]
fn diff_file_stats_reports_per_file_line_counts() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    run_jj_in(&repo_path, &["new", "-m", "edit files"]);
    let repo = Repo::open(&repo_path).expect("open repo");

    fs::write(
        repo_path.join("hello.txt"),
        "hello from tests\nsecond line\n",
    )
    .expect("modify hello.txt");
    fs::write(repo_path.join("added.txt"), "one\ntwo\nthree\n").expect("write added.txt");
    repo.refresh_working_copy().expect("snapshot working copy");

    let stats = repo.diff_file_stats("@", false).expect("diff file stats");
    let for_path = |path: &str| {
        stats
            .iter()
            .find(|file| file.path == path)
            .unwrap_or_else(|| panic!("missing stats for {path}"))
    };

    let added = for_path("added.txt");
    assert_eq!((added.insertions, added.deletions), (3, 0));

    let modified = for_path("hello.txt");
    assert_eq!(
        (modified.insertions, modified.deletions),
        (2, 1),
        "all stats: {stats:?}"
    );
}

#[test]
fn diff_file_stats_pairs_same_basename_moves_like_the_card_list() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    run_jj_in(&repo_path, &["new", "-m", "move hello"]);
    let repo = Repo::open(&repo_path).expect("open repo");

    fs::create_dir(repo_path.join("moved")).expect("create dir");
    fs::rename(
        repo_path.join("hello.txt"),
        repo_path.join("moved/hello.txt"),
    )
    .expect("move hello.txt");
    repo.refresh_working_copy().expect("snapshot working copy");

    let stats = repo.diff_file_stats("@", false).expect("diff file stats");
    assert_eq!(stats.len(), 1, "same-basename move pairs: {stats:?}");
    assert_eq!(stats[0].path, "moved/hello.txt");
    assert_eq!((stats[0].insertions, stats[0].deletions), (0, 0));
}

#[test]
fn diff_file_stats_keeps_cross_basename_renames_as_two_entries() {
    // diff_file_list pairs renames without content, so the cards show a deletion and an addition; stats must describe those cards, not a content-aware merge.
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    run_jj_in(&repo_path, &["new", "-m", "rename hello"]);
    let repo = Repo::open(&repo_path).expect("open repo");

    fs::rename(repo_path.join("hello.txt"), repo_path.join("greeting.txt"))
        .expect("rename hello.txt");
    repo.refresh_working_copy().expect("snapshot working copy");

    let stats = repo.diff_file_stats("@", false).expect("diff file stats");
    assert_eq!(stats.len(), 2, "cards stay split: {stats:?}");
    let removed = stats
        .iter()
        .find(|s| s.path == "hello.txt")
        .expect("removed entry");
    let added = stats
        .iter()
        .find(|s| s.path == "greeting.txt")
        .expect("added entry");
    assert_eq!((removed.insertions, removed.deletions), (0, 1));
    assert_eq!((added.insertions, added.deletions), (1, 0));
}

#[cfg(unix)]
#[test]
fn diff_file_stats_counts_symlinks_as_zero() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    std::os::unix::fs::symlink("hello.txt", repo_path.join("link.txt")).expect("create symlink");
    repo.refresh_working_copy().expect("snapshot working copy");

    let stats = repo.diff_file_stats("@", false).expect("diff file stats");
    let link = stats
        .iter()
        .find(|file| file.path == "link.txt")
        .expect("missing stats for link.txt");
    assert_eq!((link.insertions, link.deletions), (0, 0));
}

#[test]
fn diff_file_stats_counts_auto_projected_plists_in_processed_mode() {
    let fixture = FormatFixture::build();
    let repo = Repo::open(&fixture.path).expect("open repo");
    repo.refresh_working_copy().expect("snapshot working copy");

    let stats = repo.diff_file_stats("@", false).expect("diff file stats");
    let plist = stats
        .iter()
        .find(|file| file.path == FormatFixture::PLIST)
        .expect("missing stats for the binary plist");
    let projected = repo
        .show_file("@", FormatFixture::PLIST)
        .expect("projected plist hunk");
    let xml_lines = projected
        .new
        .content
        .as_deref()
        .expect("projected content")
        .lines()
        .count() as u32;
    assert_eq!((plist.insertions, plist.deletions), (xml_lines, 0));
}

#[test]
fn diff_file_stats_counts_binary_files_as_zero() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    fs::write(repo_path.join("blob.bin"), [0u8, 159, 146, 150, 0, 1]).expect("write binary file");
    repo.refresh_working_copy().expect("snapshot working copy");

    let stats = repo.diff_file_stats("@", false).expect("diff file stats");
    let blob = stats
        .iter()
        .find(|file| file.path == "blob.bin")
        .expect("missing stats for blob.bin");
    assert_eq!((blob.insertions, blob.deletions), (0, 0));
}
