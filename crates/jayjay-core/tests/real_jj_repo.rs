use std::fs;
use std::process::{Command, Output, Stdio};

use jayjay_core::diff::compute_file_diff_full;
use jayjay_core::{DEFAULT_REVSET, DiffEditDestination, DiffEditFileSelection, DiffEditRange, Repo};
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

fn hunk_for_path(repo: &Repo, rev: &str, path: &str) -> jayjay_core::DiffHunk {
    repo.show(rev)
        .expect("show change")
        .diff
        .into_iter()
        .find(|hunk| hunk.path == path)
        .unwrap_or_else(|| panic!("missing diff for {path} in {rev}"))
}

fn whole_file_selection(repo: &Repo, rev: &str, path: &str) -> DiffEditFileSelection {
    let hunk = hunk_for_path(repo, rev, path);
    let old_text = hunk.old_content.as_deref().unwrap_or_default();
    let new_text = hunk.new_content.as_deref().unwrap_or_default();
    let line_count = compute_file_diff_full(path, old_text, new_text, false).lines.len() as u32;
    DiffEditFileSelection {
        path: hunk.path,
        old_path: hunk.old_path,
        old_content: hunk.old_content,
        new_content: hunk.new_content,
        hunk_type: hunk.hunk_type,
        line_ranges: vec![DiffEditRange {
            start_line: 1,
            end_line: line_count.max(1),
        }],
    }
}

fn setup_source_change_with_child() -> (TempDir, std::path::PathBuf, Repo) {
    let temp_dir = init_real_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    fs::write(repo_path.join("notes.md"), "# moved content\n\nline for diffedit\n")
        .expect("write notes file");
    repo.refresh_working_copy()
        .expect("snapshot source change");
    repo.describe("@", "source change")
        .expect("describe source change");
    repo.new_change("@", "working copy child")
        .expect("create working copy child");

    (temp_dir, repo_path, repo)
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

#[test]
fn default_revset_shows_nearby_heads() {
    if !jj_is_available() {
        eprintln!("skipping real jj repo test because `jj` is not installed");
        return;
    }

    let temp_dir = init_real_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo_str = repo_path.to_str().expect("repo path utf-8");

    run_jj(&["-R", repo_str, "new", "@", "-m", "current head"]);
    run_jj(&["-R", repo_str, "new", "@-", "-m", "parallel head"]);

    let repo = Repo::open(&repo_path).expect("open repo");

    let log = repo.log(DEFAULT_REVSET).expect("evaluate default revset");
    assert!(
        !log.is_empty(),
        "default revset should evaluate to visible changes"
    );
    assert!(
        log.iter()
            .any(|change| change.description.trim_end() == "parallel head"),
        "expected default revset to include the current head"
    );
    assert!(
        log.iter()
            .any(|change| change.description.trim_end() == "current head"),
        "expected default revset to include nearby sibling heads"
    );
    assert!(
        log.iter()
            .any(|change| change.description.trim_end() == "initial change"),
        "expected default revset to keep trunk/root context visible"
    );
}

#[test]
fn trunk_revset_function_is_available_in_app_parser() {
    if !jj_is_available() {
        eprintln!("skipping real jj repo test because `jj` is not installed");
        return;
    }

    let temp_dir = init_real_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    let log = repo.log("trunk() | @").expect("evaluate trunk() revset");
    assert!(
        log.iter()
            .any(|change| change.description.trim_end() == "initial change"),
        "expected trunk() expression to parse and include current visible work"
    );
}

#[test]
fn diffedit_remove_from_source_updates_working_copy() {
    if !jj_is_available() {
        eprintln!("skipping real jj repo test because `jj` is not installed");
        return;
    }

    let temp_dir = init_real_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    fs::write(
        repo_path.join("notes.md"),
        "# keep this file\n\nremove this whole file from source\n",
    )
    .expect("write new file");
    repo.refresh_working_copy()
        .expect("snapshot working copy changes");

    let selection = whole_file_selection(&repo, "@", "notes.md");
    repo.apply_diff_selection(
        "@",
        DiffEditDestination::RemoveFromSource,
        &[selection],
        "",
        false,
    )
    .expect("remove selected line from working copy");

    let current = repo.show("@").expect("show updated working copy");
    assert!(
        current.diff.iter().all(|hunk| hunk.path != "notes.md"),
        "notes.md should be removed from the working copy diff"
    );
    let hello = current
        .diff
        .iter()
        .find(|hunk| hunk.path == "hello.txt")
        .expect("hello.txt initial diff remains");
    assert_eq!(
        hello.new_content.as_deref(),
        Some("hello from jayjay\n")
    );
}

#[test]
fn diffedit_move_to_working_copy_moves_selected_file() {
    if !jj_is_available() {
        eprintln!("skipping real jj repo test because `jj` is not installed");
        return;
    }

    let (_temp_dir, _repo_path, repo) = setup_source_change_with_child();
    let selection = whole_file_selection(&repo, "@-", "notes.md");

    repo.apply_diff_selection(
        "@-",
        DiffEditDestination::MoveToWorkingCopy,
        &[selection],
        "",
        false,
    )
    .expect("move selected file to working copy");

    let source = repo.show("@-").expect("show rewritten source");
    assert!(
        source.diff.iter().all(|hunk| hunk.path != "notes.md"),
        "source change should no longer contain notes.md"
    );

    let current = repo.show("@").expect("show updated working copy");
    let notes = current
        .diff
        .iter()
        .find(|hunk| hunk.path == "notes.md")
        .expect("notes.md moved to working copy");
    assert_eq!(
        notes.new_content.as_deref(),
        Some("# moved content\n\nline for diffedit\n")
    );
}

#[test]
fn diffedit_new_child_extracts_selected_file_between_source_and_working_copy() {
    if !jj_is_available() {
        eprintln!("skipping real jj repo test because `jj` is not installed");
        return;
    }

    let (_temp_dir, _repo_path, repo) = setup_source_change_with_child();
    let selection = whole_file_selection(&repo, "@-", "notes.md");

    repo.apply_diff_selection(
        "@-",
        DiffEditDestination::NewChild,
        &[selection],
        "selected child",
        false,
    )
    .expect("extract selected file as child");

    let all = repo.log("all()").expect("read all changes");
    let child = all
        .iter()
        .find(|change| change.description == "selected child")
        .expect("selected child visible");
    let source = all
        .iter()
        .filter(|change| change.description == "source change")
        .find(|change| {
            repo.show(&change.commit_id)
                .expect("show candidate source")
                .diff
                .iter()
                .all(|hunk| hunk.path != "notes.md")
        })
        .expect("rewritten source change still visible");
    assert_eq!(child.parents, vec![source.commit_id.clone()]);

    let source_detail = repo.show(&source.commit_id).expect("show rewritten source");
    assert!(
        source_detail.diff.iter().all(|hunk| hunk.path != "notes.md"),
        "rewritten source should no longer contain notes.md"
    );

    let child_detail = repo.show(&child.commit_id).expect("show selected child");
    let notes = child_detail
        .diff
        .iter()
        .find(|hunk| hunk.path == "notes.md")
        .expect("notes.md extracted to child");
    assert_eq!(
        notes.new_content.as_deref(),
        Some("# moved content\n\nline for diffedit\n")
    );
}

#[test]
fn diffedit_new_parallel_extracts_selected_file_as_sibling() {
    if !jj_is_available() {
        eprintln!("skipping real jj repo test because `jj` is not installed");
        return;
    }

    let (_temp_dir, _repo_path, repo) = setup_source_change_with_child();
    let selection = whole_file_selection(&repo, "@-", "notes.md");

    repo.apply_diff_selection(
        "@-",
        DiffEditDestination::NewParallel,
        &[selection],
        "selected parallel",
        false,
    )
    .expect("extract selected file as parallel");

    let all = repo.log("all()").expect("read all changes");
    let parallel = all
        .iter()
        .find(|change| change.description == "selected parallel")
        .expect("selected parallel visible");
    let source = all
        .iter()
        .filter(|change| change.description == "source change")
        .find(|change| {
            repo.show(&change.commit_id)
                .expect("show candidate source")
                .diff
                .iter()
                .all(|hunk| hunk.path != "notes.md")
        })
        .expect("rewritten source change still visible");

    assert_eq!(parallel.parents, source.parents);

    let source_detail = repo.show(&source.commit_id).expect("show rewritten source");
    assert!(
        source_detail.diff.iter().all(|hunk| hunk.path != "notes.md"),
        "rewritten source should no longer contain notes.md"
    );

    let parallel_detail = repo.show(&parallel.commit_id).expect("show selected parallel");
    let notes = parallel_detail
        .diff
        .iter()
        .find(|hunk| hunk.path == "notes.md")
        .expect("notes.md extracted to parallel change");
    assert_eq!(
        notes.new_content.as_deref(),
        Some("# moved content\n\nline for diffedit\n")
    );
}
