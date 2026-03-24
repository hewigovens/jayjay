use std::fs;
use std::process::{Command, Output, Stdio};

use jayjay_core::Repo;
use tempfile::TempDir;

fn jj_is_available() -> bool {
    Command::new("jj")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn run_jj(args: &[&str]) -> Output {
    let output = Command::new("jj")
        .args(args)
        .output()
        .unwrap_or_else(|err| panic!("failed to run jj {:?}: {err}", args));

    if !output.status.success() {
        panic!(
            "jj {:?} failed\nstatus: {:?}\nstdout:\n{}\nstderr:\n{}",
            args,
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }

    output
}

fn init_real_repo() -> TempDir {
    let temp_dir = tempfile::tempdir().expect("create tempdir");
    let repo_path = temp_dir.path().join("repo");
    let repo_str = repo_path.to_str().expect("repo path utf-8");

    run_jj(&["git", "init", repo_str]);
    run_jj(&[
        "-R",
        repo_str,
        "config",
        "set",
        "--repo",
        "user.name",
        "Test User",
    ]);
    run_jj(&[
        "-R",
        repo_str,
        "config",
        "set",
        "--repo",
        "user.email",
        "test@example.com",
    ]);

    fs::write(repo_path.join("hello.txt"), "hello from jayjay\n").expect("write initial file");
    run_jj(&["-R", repo_str, "describe", "-m", "initial change"]);

    temp_dir
}

#[test]
fn core_repo_works_against_a_real_jj_repo() {
    if !jj_is_available() {
        eprintln!("skipping real jj repo test because `jj` is not installed");
        return;
    }

    let temp_dir = init_real_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    let current = repo.show("@").expect("show working copy");
    assert_eq!(current.info.description.trim_end(), "initial change");
    assert!(current.info.is_working_copy);
    assert_eq!(current.diff.len(), 1, "expected initial file diff");
    assert_eq!(current.diff[0].path, "hello.txt");
    assert_eq!(current.diff[0].hunk_type, jayjay_core::HunkType::Added);
    assert_eq!(
        current.diff[0].new_content.as_deref(),
        Some("hello from jayjay\n")
    );

    let changes = repo.log("@ | ancestors(@, 5)").expect("read log");
    assert!(!changes.is_empty(), "expected at least one visible change");
    assert!(
        changes
            .iter()
            .any(|change| change.description.trim_end() == "initial change")
    );

    repo.describe("@", "renamed from core test")
        .expect("describe change");
    let renamed = repo.show("@").expect("show renamed change");
    assert_eq!(renamed.info.description, "renamed from core test");

    repo.new_change("@", "child change from core test")
        .expect("create child change");
    let child = repo.show("@").expect("show new working copy");
    assert_eq!(child.info.description, "child change from core test");
    assert!(child.info.is_working_copy);

    let updated_log = repo.log("@ | ancestors(@, 5)").expect("read updated log");
    assert!(
        updated_log
            .iter()
            .any(|change| change.description == "renamed from core test"),
        "expected parent change to remain visible after creating a child"
    );

    repo.create_bookmark("test-bookmark", "@")
        .expect("create bookmark");
    let bookmarks = repo.list_bookmarks().expect("list bookmarks");
    assert!(bookmarks.iter().any(|bookmark| {
        bookmark.name == "test-bookmark" && bookmark.change_id == child.info.change_id
    }));
}

#[test]
fn refresh_working_copy_snapshots_uncommitted_changes() {
    if !jj_is_available() {
        eprintln!("skipping real jj repo test because `jj` is not installed");
        return;
    }

    let temp_dir = init_real_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    fs::write(
        repo_path.join("hello.txt"),
        "hello from jayjay\nupdated in working copy\n",
    )
    .expect("update tracked file");
    fs::write(
        repo_path.join("notes.md"),
        "# scratch\n\nworking copy only\n",
    )
    .expect("write new file");

    repo.refresh_working_copy()
        .expect("snapshot working copy changes");

    let current = repo.show("@").expect("show refreshed working copy");
    assert!(current.info.is_working_copy);
    assert_eq!(
        current.diff.len(),
        2,
        "expected tracked and new file in diff"
    );

    let hello = current
        .diff
        .iter()
        .find(|hunk| hunk.path == "hello.txt")
        .expect("hello.txt diff");
    assert_eq!(hello.hunk_type, jayjay_core::HunkType::Added);
    assert_eq!(
        hello.new_content.as_deref(),
        Some("hello from jayjay\nupdated in working copy\n")
    );

    let notes = current
        .diff
        .iter()
        .find(|hunk| hunk.path == "notes.md")
        .expect("notes.md diff");
    assert_eq!(notes.hunk_type, jayjay_core::HunkType::Added);
    assert_eq!(
        notes.new_content.as_deref(),
        Some("# scratch\n\nworking copy only\n")
    );
}

#[test]
fn backout_uses_jj_revert_and_creates_reverse_change() {
    if !jj_is_available() {
        eprintln!("skipping real jj repo test because `jj` is not installed");
        return;
    }

    let temp_dir = init_real_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    repo.new_change("@", "child change")
        .expect("create child working copy");
    repo.backout("@-").expect("revert parent change");

    let reverted = repo.show("@-").expect("show reverted parent");
    assert!(
        reverted.info.description.contains("Revert"),
        "expected revert description, got {:?}",
        reverted.info.description
    );

    let current = repo.show("@").expect("show rebased working copy");
    assert_eq!(current.info.description, "child change");
}
