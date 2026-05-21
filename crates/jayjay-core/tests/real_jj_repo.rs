use std::fs;
use std::path::Path;
use std::process::{Command, Output, Stdio};

use jayjay_core::diff::compute_file_diff_full;
use jayjay_core::{
    DEFAULT_REVSET, DiffEditDestination, DiffEditFileSelection, DiffEditRange, Repo,
};
use tempfile::TempDir;

fn jj_is_available() -> bool {
    Command::new("jj")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn git_is_available() -> bool {
    Command::new("git")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn run_command(program: &str, display_args: &[String], command: &mut Command) -> Output {
    let output = command
        .output()
        .unwrap_or_else(|err| panic!("failed to run {program} {display_args:?}: {err}"));

    if !output.status.success() {
        panic!(
            "{program} {display_args:?} failed\nstatus: {:?}\nstdout:\n{}\nstderr:\n{}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }

    output
}

fn run_jj(args: &[&str]) -> Output {
    let mut command = Command::new("jj");
    command.args(args);
    let display_args = args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>();
    run_command("jj", &display_args, &mut command)
}

fn run_git(repo_path: &Path, args: &[&str]) -> Output {
    let mut command = Command::new("git");
    command.arg("-C").arg(repo_path).args(args);
    let display_args = std::iter::once("-C".to_string())
        .chain(std::iter::once(repo_path.display().to_string()))
        .chain(args.iter().map(|arg| arg.to_string()))
        .collect::<Vec<_>>();
    run_command("git", &display_args, &mut command)
}

fn init_real_repo() -> TempDir {
    let temp_dir = tempfile::tempdir().expect("create tempdir");
    let repo_path = temp_dir.path().join("repo");
    let repo_str = repo_path.to_str().expect("repo path utf-8");

    run_jj(&["git", "init", "--colocate", repo_str]);
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
    let line_count = compute_file_diff_full(path, old_text, new_text, false)
        .lines
        .len() as u32;
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

fn selection_for_lines(
    repo: &Repo,
    rev: &str,
    path: &str,
    line_ranges: &[(u32, u32)],
) -> DiffEditFileSelection {
    let hunk = hunk_for_path(repo, rev, path);
    DiffEditFileSelection {
        path: hunk.path,
        old_path: hunk.old_path,
        old_content: hunk.old_content,
        new_content: hunk.new_content,
        hunk_type: hunk.hunk_type,
        line_ranges: line_ranges
            .iter()
            .map(|(start_line, end_line)| DiffEditRange {
                start_line: *start_line,
                end_line: *end_line,
            })
            .collect(),
    }
}

fn setup_source_change_with_child() -> (TempDir, std::path::PathBuf, Repo) {
    let temp_dir = init_real_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    fs::write(
        repo_path.join("notes.md"),
        "# moved content\n\nline for diffedit\n",
    )
    .expect("write notes file");
    repo.refresh_working_copy().expect("snapshot source change");
    repo.describe("@", "source change")
        .expect("describe source change");
    repo.new_change("@", "working copy child")
        .expect("create working copy child");

    (temp_dir, repo_path, repo)
}

#[test]
fn refresh_working_copy_respects_git_excludes_file() {
    if !jj_is_available() || !git_is_available() {
        eprintln!("skipping real jj repo test because `jj` or `git` is not installed");
        return;
    }

    let temp_dir = init_real_repo();
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
    if !jj_is_available() {
        eprintln!("skipping real jj repo test because `jj` is not installed");
        return;
    }

    let temp_dir = init_real_repo();
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
    if !jj_is_available() {
        eprintln!("skipping real jj repo test because `jj` is not installed");
        return;
    }

    let temp_dir = init_real_repo();
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
    assert_eq!(child.info.description.trim(), "child change from core test");
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
fn image_file_is_cached_and_surfaced_as_diff_preview() {
    if !jj_is_available() {
        eprintln!("skipping image preview test because `jj` is not installed");
        return;
    }

    let temp_dir = init_real_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    // Minimum-viable PNG-ish binary. extract_image_preview doesn't decode the
    // bytes — it only routes by extension — so any non-empty binary works.
    let png_bytes: Vec<u8> = vec![
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0xff, 0xfe, 0xfd, 0xfc,
    ];
    fs::write(repo_path.join("icon.png"), &png_bytes).expect("write png");
    repo.refresh_working_copy().expect("snapshot png add");

    let current = repo.show("@").expect("show working copy");
    let icon = current
        .diff
        .iter()
        .find(|hunk| hunk.path == "icon.png")
        .expect("icon.png in diff");

    assert_eq!(icon.hunk_type, jayjay_core::HunkType::Added);
    assert!(
        icon.old_preview.is_none(),
        "added file has no old side preview"
    );

    let Some(jayjay_core::DiffPreview::Image { path: cache_path }) = &icon.new_preview else {
        panic!(
            "expected DiffPreview::Image on new side, got {:?}",
            icon.new_preview
        );
    };

    let cached_bytes =
        fs::read(cache_path).unwrap_or_else(|err| panic!("read cache file {cache_path}: {err}"));
    assert_eq!(
        cached_bytes, png_bytes,
        "cache file contents must match the original bytes"
    );

    let new_content = icon.new_content.as_deref().unwrap_or("");
    assert!(
        new_content.starts_with("<image "),
        "new_content should be the image placeholder, got {new_content:?}"
    );
}

#[test]
fn revert_change_uses_jj_revert_and_creates_reverse_change() {
    if !jj_is_available() {
        eprintln!("skipping real jj repo test because `jj` is not installed");
        return;
    }

    let temp_dir = init_real_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    repo.new_change("@", "child change")
        .expect("create child working copy");
    repo.revert_change("@-").expect("revert parent change");

    let current = repo.show("@").expect("show unchanged working copy");
    assert_eq!(current.info.description.trim(), "child change");

    let changes = repo.log("all()").expect("log changes");
    let reverted = changes
        .iter()
        .find(|change| change.description.contains("Revert"))
        .expect("revert change");
    assert!(
        reverted.description.contains("Revert"),
        "expected revert description, got {:?}",
        reverted.description
    );
    assert_eq!(reverted.parents, vec![current.info.commit_id]);
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
fn trunk_revset_alias_is_available_in_app_parser() {
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
fn immutable_heads_revset_alias_is_available_in_app_parser() {
    if !jj_is_available() {
        eprintln!("skipping real jj repo test because `jj` is not installed");
        return;
    }

    let temp_dir = init_real_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    let log = repo
        .log("present(@) | ancestors(immutable_heads().., 20) | trunk()")
        .expect("evaluate immutable_heads() revset");
    assert!(
        log.iter().any(|change| change.is_working_copy),
        "expected immutable_heads() expression to include the working copy"
    );
    assert!(
        log.iter()
            .any(|change| change.description.trim_end() == "initial change"),
        "expected immutable_heads() expression to parse alongside trunk()"
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
    assert_eq!(hello.new_content.as_deref(), Some("hello from jayjay\n"));

    assert!(
        !repo_path.join("notes.md").exists(),
        "notes.md should be removed from disk after updating the working copy"
    );

    repo.refresh_working_copy()
        .expect("refresh updated working copy");
    let refreshed = repo.show("@").expect("show refreshed working copy");
    assert!(
        refreshed.diff.iter().all(|hunk| hunk.path != "notes.md"),
        "refresh should not reintroduce notes.md after removing it from @"
    );
}

#[test]
fn diffedit_remove_selected_lines_updates_working_copy_on_disk() {
    if !jj_is_available() {
        eprintln!("skipping real jj repo test because `jj` is not installed");
        return;
    }

    let temp_dir = init_real_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    fs::write(
        repo_path.join("notes.md"),
        "first line\nremove me\nlast line\n",
    )
    .expect("write new file");
    repo.refresh_working_copy()
        .expect("snapshot working copy changes");

    let selection = selection_for_lines(&repo, "@", "notes.md", &[(2, 2)]);
    repo.apply_diff_selection(
        "@",
        DiffEditDestination::RemoveFromSource,
        &[selection],
        "",
        false,
    )
    .expect("remove selected lines from working copy");

    let expected = "first line\nlast line\n";
    let current = repo.show("@").expect("show updated working copy");
    let notes = current
        .diff
        .iter()
        .find(|hunk| hunk.path == "notes.md")
        .expect("notes.md remains in working copy");
    assert_eq!(notes.new_content.as_deref(), Some(expected));
    assert_eq!(
        fs::read_to_string(repo_path.join("notes.md")).expect("read updated working copy file"),
        expected
    );

    repo.refresh_working_copy()
        .expect("refresh updated working copy");
    let refreshed = repo.show("@").expect("show refreshed working copy");
    let refreshed_notes = refreshed
        .diff
        .iter()
        .find(|hunk| hunk.path == "notes.md")
        .expect("notes.md remains after refresh");
    assert_eq!(refreshed_notes.new_content.as_deref(), Some(expected));
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
        source_detail
            .diff
            .iter()
            .all(|hunk| hunk.path != "notes.md"),
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
        source_detail
            .diff
            .iter()
            .all(|hunk| hunk.path != "notes.md"),
        "rewritten source should no longer contain notes.md"
    );

    let parallel_detail = repo
        .show(&parallel.commit_id)
        .expect("show selected parallel");
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

#[test]
fn test_default_revset_not_empty() {
    assert!(!DEFAULT_REVSET.is_empty());
    assert!(
        DEFAULT_REVSET.contains("@"),
        "default revset should contain '@'"
    );
}
