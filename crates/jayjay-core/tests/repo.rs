use std::fs;

use jayjay_core::Repo;
use jayjay_core::diff::{ConflictLineKind, compute_file_diff_full};
use jj_test::{current_op_id, init_jj_repo, run_jj};

#[test]
fn show_summary_marks_divergent_revision_loaded_by_commit_id() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo_str = repo_path.to_str().expect("repo path utf-8");
    let base_op = current_op_id(&repo_path);

    fs::write(repo_path.join("hello.txt"), "left\n").expect("write left version");
    run_jj(&["-R", repo_str, "describe", "-m", "left version"]);

    fs::write(repo_path.join("hello.txt"), "right\n").expect("write right version");
    run_jj(&[
        "-R",
        repo_str,
        "--at-op",
        &base_op,
        "describe",
        "-m",
        "right version",
    ]);

    let repo = Repo::open(&repo_path).expect("open repo");
    let changes = repo.log("all()").expect("load log");
    let divergent = changes
        .iter()
        .find(|change| change.is_divergent)
        .unwrap_or_else(|| {
            panic!(
                "missing divergent changes: {:?}",
                changes
                    .iter()
                    .map(|change| (
                        change.change_id.as_str(),
                        change.commit_id.as_str(),
                        change.description.as_str(),
                        change.is_divergent,
                    ))
                    .collect::<Vec<_>>()
            )
        });
    assert!(
        divergent.is_divergent,
        "log entries should mark duplicate change ids as divergent"
    );

    let detail = repo
        .show_summary(&divergent.commit_id)
        .expect("show divergent summary by commit id");
    assert!(
        detail.info.is_divergent,
        "detail loaded by commit id should preserve divergent status"
    );
}
#[test]
fn mutation_rejects_revset_matching_multiple_commits() {
    // resolve_commit takes the first stream entry, so an ambiguous revset must
    // be rejected like the jj CLI rather than silently rewriting one match.
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    repo.new_change("@", "child").expect("create child change");

    let err = repo
        .describe("@ | @-", "should not land")
        .expect_err("a multi-commit revset must fail, not pick one match");
    assert!(
        err.to_string()
            .contains("resolved to more than one revision"),
        "unexpected error: {err}"
    );
}
#[test]
fn show_file_materializes_conflicted_file_content() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo_str = repo_path.to_str().expect("repo path utf-8");

    fs::write(repo_path.join("hello.txt"), "line1\nline2\nline3\n").expect("write base");
    run_jj(&["-R", repo_str, "describe", "-m", "base"]);
    run_jj(&["-R", repo_str, "bookmark", "create", "main", "-r", "@"]);

    run_jj(&["-R", repo_str, "new", "-m", "main side"]);
    fs::write(repo_path.join("hello.txt"), "line1\nline2 MAIN\nline3\n").expect("write main");
    run_jj(&["-R", repo_str, "bookmark", "set", "main", "-r", "@"]);

    run_jj(&["-R", repo_str, "new", "-r", "main-", "-m", "feature side"]);
    fs::write(repo_path.join("hello.txt"), "line1\nline2 FEATURE\nline3\n").expect("write feature");
    run_jj(&["-R", repo_str, "rebase", "-r", "@", "-d", "main"]);

    let repo = Repo::open(&repo_path).expect("open repo");
    let hunk = repo
        .show_file("@", "hello.txt")
        .expect("show conflicted file");
    let new_content = hunk.new_content.as_deref().expect("new content");

    assert!(
        !new_content.contains("<conflicted file>"),
        "conflict placeholder should not reach the diff view"
    );
    assert!(new_content.contains("<<<<<<< conflict 1 of 1"));
    assert!(new_content.contains("%%%%%%%"));
    assert!(new_content.contains(">>>>>>> conflict 1 of 1 ends"));

    let diff = compute_file_diff_full(
        "hello.txt",
        hunk.old_content.as_deref().unwrap_or_default(),
        new_content,
        false,
    );
    assert!(
        diff.lines
            .iter()
            .any(|line| line.conflict_kind == ConflictLineKind::Start)
    );
    assert!(
        diff.lines
            .iter()
            .any(|line| line.conflict_kind == ConflictLineKind::End)
    );
}
#[test]
fn repo_operations_work_against_jj_fixture() {
    let temp_dir = init_jj_repo();
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
    let temp_dir = init_jj_repo();
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
    let temp_dir = init_jj_repo();
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
    let temp_dir = init_jj_repo();
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
/// Look up a revision in the log by its description (trimmed).
fn change_by_description<'a>(
    changes: &'a [jayjay_core::ChangeInfo],
    description: &str,
) -> &'a jayjay_core::ChangeInfo {
    changes
        .iter()
        .find(|change| change.description.trim() == description)
        .unwrap_or_else(|| panic!("missing change with description {description:?}"))
}

#[test]
fn squash_merges_descriptions_and_moves_content_into_parent() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    // Parent A with its own file, child B (working copy) with a distinct file.
    fs::write(repo_path.join("a.txt"), "from A\n").expect("write a.txt");
    repo.refresh_working_copy().expect("snapshot A");
    repo.describe("@", "base msg").expect("describe A");
    repo.new_change("@", "child msg").expect("create child B");
    fs::write(repo_path.join("b.txt"), "from B\n").expect("write b.txt");
    repo.refresh_working_copy().expect("snapshot B");

    repo.squash("@", None).expect("squash B into A");

    let parent = repo.show("@-").expect("show squashed parent");
    assert_eq!(
        parent.info.description.trim(),
        "base msg\nchild msg",
        "dest description must come first, then source"
    );

    let added: Vec<&str> = parent
        .diff
        .iter()
        .filter(|hunk| hunk.hunk_type == jayjay_core::HunkType::Added)
        .map(|hunk| hunk.path.as_str())
        .collect();
    assert!(
        added.contains(&"a.txt") && added.contains(&"b.txt"),
        "squashed parent must hold both files, got {added:?}"
    );
    let b_file = parent
        .diff
        .iter()
        .find(|hunk| hunk.path == "b.txt")
        .expect("b.txt landed in parent");
    assert_eq!(b_file.new_content.as_deref(), Some("from B\n"));
}

#[test]
fn squash_with_empty_source_description_leaves_dest_unchanged() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    repo.describe("@", "kept description").expect("describe A");
    // Child B has no description but does carry a file edit to squash.
    repo.new_change("@", "").expect("create empty-desc child B");
    fs::write(repo_path.join("b.txt"), "from B\n").expect("write b.txt");
    repo.refresh_working_copy().expect("snapshot B");

    repo.squash("@", None).expect("squash empty-desc child");

    let parent = repo.show("@-").expect("show squashed parent");
    assert_eq!(
        parent.info.description.trim(),
        "kept description",
        "empty source description must not alter the destination"
    );
    assert!(
        parent.diff.iter().any(|hunk| hunk.path == "b.txt"),
        "child content must still land in the destination"
    );
}

#[test]
fn squash_into_explicit_grandparent_target() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    // Grandparent G -> parent P -> child C (working copy).
    repo.describe("@", "grandparent").expect("describe G");
    repo.new_change("@", "parent").expect("create P");
    repo.new_change("@", "child").expect("create C");
    fs::write(repo_path.join("c.txt"), "from C\n").expect("write c.txt");
    repo.refresh_working_copy().expect("snapshot C");

    // Squash C directly into the grandparent, skipping the parent.
    repo.squash("@", Some("@--"))
        .expect("squash into grandparent");

    let changes = repo.log("all()").expect("log changes");
    let grandparent = change_by_description(&changes, "grandparent\nchild");
    let detail = repo.show(&grandparent.commit_id).expect("show grandparent");
    let c_file = detail
        .diff
        .iter()
        .find(|hunk| hunk.path == "c.txt")
        .expect("child content must land in the grandparent");
    assert_eq!(c_file.new_content.as_deref(), Some("from C\n"));

    // The intermediate parent retains its own identity and gains no content.
    let parent = change_by_description(&changes, "parent");
    let parent_detail = repo.show(&parent.commit_id).expect("show parent");
    assert!(
        !parent_detail.diff.iter().any(|hunk| hunk.path == "c.txt"),
        "content must not pass through the skipped parent"
    );
}

#[test]
fn squash_root_commit_is_rejected() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    let err = repo
        .squash("root()", None)
        .expect_err("squashing the root commit must fail");
    assert!(
        err.to_string().contains("cannot squash root commit"),
        "unexpected error: {err}"
    );
}

#[test]
fn squash_snapshots_unsnapshotted_working_copy_edit() {
    // Regression: mutations must snapshot the working copy first, or the
    // post-transaction checkout discards on-disk edits that were never snapshotted.
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    repo.new_change("@", "child").expect("new child change");
    fs::write(repo_path.join("notes.md"), "edit in progress\n").expect("disk edit");

    repo.squash("@", None).expect("squash @ into parent");

    let parent = repo.show("@-").expect("show parent");
    let notes = parent
        .diff
        .iter()
        .find(|hunk| hunk.path == "notes.md")
        .expect("squash must capture the un-snapshotted disk edit");
    assert_eq!(notes.new_content.as_deref(), Some("edit in progress\n"));
}
