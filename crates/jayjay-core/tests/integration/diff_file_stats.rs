use std::fs;

use jayjay_core::{HunkType, Repo};
use jj_test::{FormatFixture, init_jj_repo, run_jj_in};

#[test]
fn diff_stats_totals_the_per_file_counts() {
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

    let files = repo.diff_file_stats("@", false).expect("diff file stats");
    let mut counts: Vec<_> = files
        .iter()
        .map(|file| (file.path.as_str(), file.insertions, file.deletions))
        .collect();
    counts.sort_unstable();
    assert_eq!(counts, [("added.txt", 3, 0), ("hello.txt", 2, 1)]);
    let stats = repo.diff_stats("@").expect("diff stats");
    assert_eq!(stats.files_changed, 2);
    assert_eq!((stats.insertions, stats.deletions), (5, 1));
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
fn renames_pair_by_content_not_basename() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    fs::create_dir(repo_path.join("rules")).expect("create dir");
    let rules: String = (1..=30).map(|n| format!("rule {n}\n")).collect();
    fs::write(repo_path.join("rules/AGENTS.md"), rules).expect("write rules");
    run_jj_in(&repo_path, &["new", "-m", "replace agents"]);
    let repo = Repo::open(&repo_path).expect("open repo");

    fs::remove_file(repo_path.join("rules/AGENTS.md")).expect("remove rules");
    fs::write(repo_path.join("AGENTS.md"), "unrelated\n").expect("write agents");
    fs::rename(repo_path.join("hello.txt"), repo_path.join("greeting.txt"))
        .expect("rename hello.txt");
    repo.refresh_working_copy().expect("snapshot working copy");

    let mut cards: Vec<_> = repo
        .show_summary("@")
        .expect("summary")
        .diff
        .into_iter()
        .map(|hunk| (hunk.path, hunk.old_path, hunk.hunk_type))
        .collect();
    cards.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(
        cards,
        [
            ("AGENTS.md".to_owned(), None, HunkType::Added),
            (
                "greeting.txt".to_owned(),
                Some("hello.txt".to_owned()),
                HunkType::Renamed
            ),
            ("rules/AGENTS.md".to_owned(), None, HunkType::Removed),
        ]
    );
    let stats = repo.diff_file_stats("@", false).expect("diff file stats");
    assert_eq!(stats.len(), 3, "stats follow the cards: {stats:?}");
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
