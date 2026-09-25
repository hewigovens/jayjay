use std::fs;

use jayjay_core::Repo;
use jayjay_core::diff::{ConflictLineKind, compute_file_diff_full};
use jj_test::{change_by_description, current_op_id, init_jj_repo, run_git, run_jj, run_jj_in};

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
fn show_summary_reports_immutable_change() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");

    fs::write(repo_path.join("protected.txt"), "protected\n").expect("write protected file");
    run_jj_in(&repo_path, &["describe", "-m", "protected"]);
    run_jj_in(&repo_path, &["new", "-m", "child"]);
    run_git(&repo_path, &["tag", "release"]);
    run_jj_in(&repo_path, &["st"]);

    let repo = Repo::open(&repo_path).expect("open repo");
    let protected = repo
        .log("all()")
        .expect("load graph")
        .into_iter()
        .find(|change| change.description.trim() == "protected")
        .expect("protected change present");
    assert!(protected.is_immutable, "fixture change must be immutable");

    let detail = repo
        .show_summary(&protected.commit_id)
        .expect("show protected summary");
    assert!(
        detail.info.is_immutable,
        "detail metadata must preserve the graph's immutability policy"
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
    let new_content = hunk.new.content.as_deref().expect("new content");

    assert!(
        !new_content.contains("<conflicted file>"),
        "conflict placeholder should not reach the diff view"
    );
    assert!(new_content.contains("<<<<<<< conflict 1 of 1"));
    assert!(new_content.contains("%%%%%%%"));
    assert!(new_content.contains(">>>>>>> conflict 1 of 1 ends"));
    assert_eq!(
        repo.file_content("@", "hello.txt").expect("file content"),
        new_content,
        "file content must carry the same materialized markers as the diff"
    );
    assert_eq!(repo.file_content("@", "missing.txt").expect("absent"), "");

    let diff = compute_file_diff_full(
        "hello.txt",
        hunk.old.content.as_deref().unwrap_or_default(),
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
    assert!(
        (1..=current.info.change_id.len() as u32).contains(&current.info.change_id.short_len),
        "change_id_short_len out of range: {}",
        current.info.change_id.short_len
    );
    assert_eq!(current.diff.len(), 1, "expected initial file diff");
    assert_eq!(current.diff[0].path, "hello.txt");
    assert_eq!(current.diff[0].hunk_type, jayjay_core::HunkType::Added);
    assert_eq!(
        current.diff[0].new.content.as_deref(),
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
        bookmark.name == "test-bookmark" && bookmark.change_id == child.info.change_id.id
    }));
}
#[test]
fn change_info_surfaces_git_tags() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo_str = repo_path.to_str().expect("repo path utf-8");

    // Move off the initial change so it gains a git commit that HEAD points to,
    // then tag it. A jj command imports the git tag into jj's view.
    run_jj(&["-R", repo_str, "new", "-m", "child change"]);
    run_git(&repo_path, &["tag", "v1.0", "HEAD"]);
    run_git(&repo_path, &["tag", "shipped", "HEAD"]);
    run_jj(&["-R", repo_str, "new", "-m", "grandchild change"]);
    run_git(&repo_path, &["tag", "v1.1", "HEAD"]);
    run_jj(&["-R", repo_str, "status"]);

    let repo = Repo::open(&repo_path).expect("open repo");
    let changes = repo.log("all()").expect("load log");
    assert_eq!(
        change_by_description(&changes, "initial change").tags,
        ["shipped", "v1.0"]
    );
    assert_eq!(
        change_by_description(&changes, "child change").tags,
        ["v1.1"]
    );
    assert!(
        change_by_description(&changes, "grandchild change")
            .tags
            .is_empty(),
        "the untagged working copy must not pick up a tag"
    );

    let parent = repo.show("@--").expect("show tagged change");
    assert_eq!(parent.info.tags, ["shipped", "v1.0"]);
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
        hello.new.content.as_deref(),
        Some("hello from jayjay\nupdated in working copy\n")
    );

    let notes = current
        .diff
        .iter()
        .find(|hunk| hunk.path == "notes.md")
        .expect("notes.md diff");
    assert_eq!(notes.hunk_type, jayjay_core::HunkType::Added);
    assert_eq!(
        notes.new.content.as_deref(),
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
        icon.old.preview.is_none(),
        "added file has no old side preview"
    );

    let Some(jayjay_core::DiffPreview::Image { path: cache_path }) = &icon.new.preview else {
        panic!(
            "expected DiffPreview::Image on new side, got {:?}",
            icon.new.preview
        );
    };

    let cached_bytes =
        fs::read(cache_path).unwrap_or_else(|err| panic!("read cache file {cache_path}: {err}"));
    assert_eq!(
        cached_bytes, png_bytes,
        "cache file contents must match the original bytes"
    );

    let new_content = icon.new.content.as_deref().unwrap_or("");
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
    assert_eq!(reverted.parents, vec![current.info.commit_id.id.clone()]);
    let reverted = repo.show(&reverted.change_id.id).expect("show revert");
    assert_eq!(
        reverted
            .diff
            .iter()
            .map(|hunk| hunk.path.as_str())
            .collect::<Vec<_>>(),
        ["hello.txt"],
        "the revert must take back the fixture's initial file"
    );
    assert!(
        reverted.diff[0].new.content.is_none(),
        "the file is removed again"
    );
}

#[test]
fn split_parallel_makes_a_sibling_and_moves_the_working_copy_to_the_remainder() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");
    run_jj_in(&repo_path, &["describe", "-m", "both files"]);
    run_jj_in(&repo_path, &["bookmark", "create", "topic", "-r", "@"]);
    fs::write(repo_path.join("other.txt"), "other\n").expect("write other");
    repo.refresh_working_copy().expect("snapshot");
    let original = repo.log("@").expect("log")[0].clone();

    repo.split("@", &["other.txt".to_owned()], "just other", true)
        .expect("parallel split");

    let changes = repo.log("all()").expect("log");
    let first = change_by_description(&changes, "just other");
    let remainder = change_by_description(&changes, "both files");
    assert_eq!(
        first.change_id.id, original.change_id.id,
        "the split-out part keeps the change id"
    );
    assert_ne!(remainder.change_id.id, original.change_id.id);
    assert_eq!(
        first.parents, remainder.parents,
        "parallel split makes siblings"
    );
    assert!(
        remainder.is_working_copy,
        "the working copy follows the remainder"
    );
    assert_eq!(
        remainder.bookmarks,
        ["topic"],
        "bookmarks follow the remainder"
    );
    let first_paths: Vec<_> = repo
        .show(&first.commit_id.id)
        .expect("show")
        .diff
        .into_iter()
        .map(|h| h.path)
        .collect();
    let remainder_paths: Vec<_> = repo
        .show("@")
        .expect("show")
        .diff
        .into_iter()
        .map(|h| h.path)
        .collect();
    assert_eq!(first_paths, ["other.txt"]);
    assert_eq!(remainder_paths, ["hello.txt"]);
}

#[test]
fn split_refuses_a_selection_that_no_longer_differs() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");
    let before = repo.op_log().expect("op log").len();

    let err = repo
        .split("@", &["missing.txt".to_owned()], "nothing", false)
        .expect_err("a selection without changes must not split");

    assert!(err.to_string().contains("differ from the parent"), "{err}");
    assert_eq!(repo.op_log().expect("op log").len(), before);
}

#[test]
fn split_without_legacy_bookmark_behavior_keeps_bookmarks_on_the_split_out_part() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    run_jj_in(
        &repo_path,
        &[
            "config",
            "set",
            "--repo",
            "split.legacy-bookmark-behavior",
            "false",
        ],
    );
    run_jj_in(&repo_path, &["describe", "-m", "both files"]);
    run_jj_in(&repo_path, &["bookmark", "create", "topic", "-r", "@"]);
    fs::write(repo_path.join("other.txt"), "other\n").expect("write other");
    let repo = Repo::open(&repo_path).expect("open repo");

    repo.split("@", &["other.txt".to_owned()], "just other", false)
        .expect("split");

    let changes = repo.log("all()").expect("log");
    assert_eq!(
        change_by_description(&changes, "just other").bookmarks,
        ["topic"]
    );
    assert!(
        change_by_description(&changes, "both files")
            .bookmarks
            .is_empty()
    );
}

#[test]
fn move_to_working_copy_refuses_a_selection_that_does_not_differ() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");
    repo.new_change("@", "on top").expect("new change");
    let before = repo.op_log().expect("op log").len();

    let err = repo
        .move_to_working_copy("@-", &["missing.txt".to_owned()])
        .expect_err("nothing to move");

    assert!(err.to_string().contains("differ from the parent"), "{err}");
    assert_eq!(repo.op_log().expect("op log").len(), before);

    repo.new_change("@", "empty source").expect("empty change");
    repo.new_change("@", "on top of empty").expect("new change");
    let before = repo.op_log().expect("op log").len();
    let err = repo
        .move_to_working_copy("@-", &["hello.txt".to_owned()])
        .expect_err("an already-empty source must not be abandoned into @");
    assert!(err.to_string().contains("differ from the parent"), "{err}");
    assert_eq!(repo.op_log().expect("op log").len(), before);
    assert!(
        repo.log("all()")
            .expect("log")
            .iter()
            .any(|change| change.description.trim() == "empty source"),
        "the empty source must survive"
    );
    assert_eq!(
        repo.log("@").expect("log")[0].description.trim(),
        "on top of empty"
    );
}

#[test]
fn move_to_working_copy_abandons_an_emptied_source_and_keeps_its_message() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");
    repo.new_change("@", "on top").expect("new change");

    repo.move_to_working_copy("@-", &["hello.txt".to_owned()])
        .expect("move the only file");

    let changes = repo.log("all()").expect("log");
    assert!(
        changes
            .iter()
            .all(|change| change.description.trim() != "initial change"),
        "the emptied source is abandoned"
    );
    let head = &repo.log("@").expect("log")[0];
    assert_eq!(head.description.trim(), "on top\ninitial change");
    assert!(!head.is_empty);
}

#[test]
fn merge_rejects_changes_from_the_same_line_of_history() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    repo.describe("@", "parent").expect("describe parent");
    repo.new_change("@", "child").expect("create child");

    let err = repo
        .merge(&[
            "description(\"parent\")".to_owned(),
            "description(\"child\")".to_owned(),
        ])
        .expect_err("linear changes must not create a redundant merge");

    assert!(
        err.to_string().contains("independent heads"),
        "unexpected error: {err}"
    );
    assert_eq!(
        repo.show("@")
            .expect("show unchanged working copy")
            .info
            .description
            .trim(),
        "child"
    );
}

/// Look up a revision in the log by its description (trimmed).
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
    assert_eq!(b_file.new.content.as_deref(), Some("from B\n"));
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
    assert_eq!(c_file.new.content.as_deref(), Some("from C\n"));

    // The intermediate parent retains its own identity and gains no content.
    let parent = change_by_description(&changes, "parent");
    let parent_detail = repo.show(&parent.commit_id).expect("show parent");
    assert!(
        !parent_detail.diff.iter().any(|hunk| hunk.path == "c.txt"),
        "content must not pass through the skipped parent"
    );
}

#[test]
fn open_reports_conflict_markers_in_global_gitconfig() {
    if let Ok(repo_path) = std::env::var("JAYJAY_BROKEN_GITCONFIG_REPO") {
        let error = match Repo::open(std::path::Path::new(&repo_path)) {
            Ok(_) => panic!("conflicted global gitconfig should fail to open"),
            Err(error) => error,
        };
        let message = error.to_string();
        assert!(
            message.contains("unexpected token") && message.contains("<<<<<<<"),
            "open should surface the gix parse error, got: {message}"
        );
        return;
    }

    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let broken = temp_dir.path().join("broken.gitconfig");
    fs::write(
        &broken,
        "[user]\n\tname = Test\n<<<<<<< Conflict 1 of 1\n\temail = a@b.com\n=======\n\temail = c@d.com\n>>>>>>> Conflict 1 of 1 ends\n",
    )
    .expect("write conflicted gitconfig");

    // Keep GIT_CONFIG_GLOBAL out of this process so parallel jj fixtures stay valid.
    let output = std::process::Command::new(std::env::current_exe().expect("test executable"))
        .arg("repo::open_reports_conflict_markers_in_global_gitconfig")
        .arg("--exact")
        .env("JAYJAY_BROKEN_GITCONFIG_REPO", &repo_path)
        .env("GIT_CONFIG_GLOBAL", &broken)
        .output()
        .expect("run isolated child");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success() && stdout.contains("1 passed"),
        "child must run exactly this test and pass\nstdout:\n{stdout}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
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
        err.to_string().contains("immutable"),
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
    assert_eq!(notes.new.content.as_deref(), Some("edit in progress\n"));
}

#[test]
fn loaded_repo_knows_when_an_operation_landed_elsewhere() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");
    assert!(repo.is_at_operation_head().expect("read heads"));

    run_jj_in(&repo_path, &["describe", "-m", "elsewhere"]);
    assert!(!repo.is_at_operation_head().expect("read heads"));

    repo.refresh_working_copy()
        .expect("snapshot adopts the head");
    assert!(repo.is_at_operation_head().expect("read heads"));
}

#[test]
fn log_reports_immutability_from_the_current_operation() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    run_jj_in(&repo_path, &["describe", "-m", "protected"]);
    run_jj_in(&repo_path, &["new", "-m", "child"]);

    let repo = Repo::open(&repo_path).expect("open repo");
    let before = repo.log("all()").expect("log all");
    assert!(!change_by_description(&before, "protected").is_immutable);

    run_git(&repo_path, &["tag", "release"]);
    run_jj_in(&repo_path, &["st"]);
    repo.refresh_working_copy()
        .expect("adopt the operation that imported the tag");

    let after = repo.log("all()").expect("log all again");
    assert!(
        change_by_description(&after, "protected").is_immutable,
        "a tag imported by a later operation must make its ancestors immutable"
    );
}

#[test]
fn log_reloads_immutability_config_without_a_new_operation() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    run_jj_in(&repo_path, &["describe", "-m", "protected"]);
    run_jj_in(&repo_path, &["new", "-m", "child"]);

    let repo = Repo::open(&repo_path).expect("open repo");
    let before = repo.log("all()").expect("load log");
    assert!(!change_by_description(&before, "protected").is_immutable);
    let operation = current_op_id(&repo_path);

    for (heads, immutable) in [("@-", true), ("none()", false)] {
        run_jj_in(
            &repo_path,
            &[
                "config",
                "set",
                "--repo",
                "revset-aliases.\"immutable_heads()\"",
                heads,
            ],
        );
        repo.refresh_working_copy().expect("reload settings");
        assert_eq!(current_op_id(&repo_path), operation);
        let changes = repo.log("all()").expect("reload log");
        assert_eq!(
            change_by_description(&changes, "protected").is_immutable,
            immutable,
            "immutable_heads() = {heads}"
        );
    }
}

#[test]
fn jj_commit_describes_the_working_copy_and_starts_an_empty_child() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");
    fs::write(repo_path.join("hello.txt"), "committed\n").expect("edit hello");

    repo.jj_commit("finish hello").expect("commit");

    let changes = repo.log("all()").expect("log");
    let committed = change_by_description(&changes, "finish hello");
    let head = changes
        .iter()
        .find(|change| change.is_working_copy)
        .expect("working copy");
    assert!(!committed.is_empty && !committed.is_working_copy);
    assert_eq!(head.description, "");
    assert_eq!(head.parents, vec![committed.commit_id.id.clone()]);
    assert!(head.is_empty, "the new working copy must start empty");
    assert_eq!(
        fs::read_to_string(repo_path.join("hello.txt")).expect("read hello"),
        "committed\n",
        "the committed edit must stay on disk"
    );
    assert_eq!(
        run_git(&repo_path, &["rev-parse", "HEAD"]).stdout,
        format!("{}\n", committed.commit_id.id).into_bytes(),
        "the colocated Git HEAD must follow @'s parent"
    );
}

#[test]
fn native_bookmark_moves_reach_the_colocated_git_refs() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    repo.create_bookmark("topic", "@").expect("create bookmark");

    assert_eq!(
        run_git(&repo_path, &["rev-parse", "refs/heads/topic"]).stdout,
        format!("{}\n", repo.log("@").expect("log")[0].commit_id.id).into_bytes()
    );
}

#[test]
fn op_log_snapshots_pending_edits_before_listing() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");
    fs::write(repo_path.join("hello.txt"), "pending\n").expect("edit hello");

    let entries = repo.op_log().expect("op log");

    assert!(entries[0].is_current);
    assert_eq!(entries[0].description, "snapshot working copy");
}

#[test]
fn op_log_matches_the_cli_listing() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");
    repo.describe("@", "described natively").expect("describe");

    let entries = repo.op_log().expect("op log");
    let cli = run_jj_in(
        &repo_path,
        &[
            "op",
            "log",
            "--no-graph",
            "--limit",
            "20",
            "-T",
            r#"self.id() ++ "\t" ++ self.description() ++ "\t" ++ self.time().start().format("%s") ++ "\n""#,
        ],
    );
    let cli: Vec<Vec<&str>> = std::str::from_utf8(&cli.stdout)
        .expect("utf8")
        .lines()
        .map(|line| line.split('\t').collect())
        .collect();

    assert_eq!(entries.len(), cli.len());
    for (entry, line) in entries.iter().zip(&cli) {
        assert_eq!(entry.id.id, line[0]);
        assert_eq!(
            entry.timestamp_millis / 1000,
            line[2].parse::<i64>().expect("epoch seconds")
        );
    }
    assert!(entries[0].is_current);
    assert!(entries[1..].iter().all(|entry| !entry.is_current));
    assert_eq!(entries[0].description, "describe");
    let fixture_describe = cli[1][1]
        .strip_prefix("describe commit ")
        .expect("the fixture's CLI describe carries a full commit id");
    assert_eq!(fixture_describe.len(), 40);
    assert_eq!(
        entries[1].description,
        format!("describe commit {}", &fixture_describe[..12])
    );
}

#[test]
fn op_restore_returns_the_view_to_the_target_operation() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");
    let before = repo.op_log().expect("op log")[0].id.id.clone();
    repo.describe("@", "rewritten").expect("describe");
    assert_eq!(
        repo.log("@").expect("log")[0].description.trim(),
        "rewritten"
    );

    repo.op_restore(&before).expect("restore");

    assert_eq!(
        repo.log("@").expect("log")[0].description.trim(),
        "initial change"
    );
    let entries = repo.op_log().expect("op log");
    assert!(entries[0].is_current);
    assert_eq!(
        entries[0].description,
        format!("restore to operation {}", &before[..12])
    );
    assert!(repo.op_restore("0000000000000000").is_err());
}

#[test]
fn op_restore_refuses_an_operation_that_predates_this_workspace() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");
    let before = repo.op_log().expect("op log")[0].id.id.clone();
    let dest = temp_dir.path().join("repo-feature");
    repo.workspace_add(dest.to_str().expect("utf8 dest"), "feature", "")
        .expect("workspace add");
    let feature = Repo::open(&dest).expect("open feature workspace");

    let err = feature
        .op_restore(&before)
        .expect_err("restore must be refused");

    assert!(
        err.to_string().contains("predates workspace 'feature'"),
        "{err}"
    );
    assert!(
        feature
            .workspace_list()
            .expect("workspaces")
            .iter()
            .any(|ws| ws.name == "feature")
    );
}
