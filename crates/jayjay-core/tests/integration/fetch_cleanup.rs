use std::fs;

use jayjay_core::{FetchResult, RemoteSyncStatus, Repo};
use jj_test::{
    configure_test_user, find_bookmark, first_change, init_colocated, run_git, run_jj, run_jj_in,
};

#[test]
fn fetch_cleans_up_a_merged_bookmark_after_its_remote_is_deleted() {
    for pull_bookmark in [false, true] {
        let fixture = DeletedFeatureFixture::merged();
        if pull_bookmark {
            run_jj_in(&fixture.bob, &["git", "fetch", "-b", "main"]);
        }
        let repo = fixture.open();
        let feature = first_change(&repo, "bookmarks(exact:feature)");
        let working_copy = first_change(&repo, "@");
        assert!(feature.is_empty && find_bookmark(&repo, "feature").is_tracking_remote);

        let result = if pull_bookmark {
            repo.git_pull_bookmark("feature", &repo.sync_token())
                .unwrap()
        } else {
            fetch(&repo)
        };

        assert_eq!(result.abandoned_bookmarks, ["feature"]);
        assert!(result.suggest_abandon_bookmarks.is_empty());
        assert!(
            repo.list_bookmarks()
                .unwrap()
                .iter()
                .all(|b| b.name != "feature")
        );
        assert!(
            repo.log("all()")
                .unwrap()
                .iter()
                .all(|c| c.change_id.id != feature.change_id.id)
        );
        let rebased = repo
            .log("@ & subject(\"squash merged feature\")::")
            .unwrap();
        assert_eq!(rebased.len(), 1);
        assert_eq!(rebased[0].change_id.id, working_copy.change_id.id);
        assert!(repo.log("(trunk()..@-) & empty()").unwrap().is_empty());
    }
}

#[test]
fn fetch_preserves_unmerged_work_and_suggests_conflicted_bookmarks() {
    for (contents, conflict, empty) in [
        ("local work\n", true, false),
        ("feature\n", true, true),
        ("feature\n", false, false),
    ] {
        let merged = if conflict {
            "merged feature\n"
        } else {
            "feature\n"
        };
        let fixture = DeletedFeatureFixture::build(contents, merged, !conflict);
        let repo = fixture.open();
        let result = fetch(&repo);
        assert!(result.abandoned_bookmarks.is_empty());
        assert_eq!(
            result.suggest_abandon_bookmarks,
            if conflict { vec!["feature"] } else { vec![] }
        );
        let bookmark = find_bookmark(&repo, "feature");
        assert!(bookmark.is_tracking_remote);
        assert_eq!(bookmark.remote_targets[0].status, RemoteSyncStatus::Deleted);
        let feature = first_change(&repo, "bookmarks(exact:feature)");
        assert_eq!(feature.is_empty, empty);
        assert_eq!(feature.has_conflict, conflict);

        let second = fetch(&repo);
        assert!(second.abandoned_bookmarks.is_empty());
        assert!(second.suggest_abandon_bookmarks.is_empty());
    }
}

#[test]
fn fetch_cleanup_only_counts_tracked_remotes_as_surviving() {
    for tracked in [true, false] {
        let fixture = DeletedFeatureFixture::merged();
        fixture.add_remote_ref("upstream", "feature");
        if tracked {
            run_jj_in(&fixture.bob, &["bookmark", "track", "feature@upstream"]);
        }
        let repo = fixture.open();
        let result = fetch(&repo);
        if !tracked {
            assert_eq!(result.abandoned_bookmarks, ["feature"]);
            assert!(repo.log("bookmarks(exact:feature)").unwrap().is_empty());
            continue;
        }
        assert!(result.abandoned_bookmarks.is_empty());
        assert!(result.suggest_abandon_bookmarks.is_empty());
        assert!(first_change(&repo, "bookmarks(exact:feature)").is_empty);
        let bookmark = find_bookmark(&repo, "feature");
        assert_eq!(bookmark.tracked_remotes, ["origin", "upstream"]);
        assert_eq!(bookmark.remote_targets[0].status, RemoteSyncStatus::Deleted);
        assert_ne!(bookmark.remote_targets[1].status, RemoteSyncStatus::Deleted);
    }
}

#[test]
fn fetch_cleanup_is_one_undoable_operation_for_multiple_bookmarks() {
    let fixture = DeletedFeatureFixture::merged();
    fixture.add_remote_ref("origin", "second");
    run_jj_in(&fixture.bob, &["bookmark", "track", "second@origin"]);
    run_jj_in(&fixture.bob, &["bookmark", "set", "second", "-r", "@"]);
    run_jj_in(&fixture.bob, &["new", "-m", "keep working"]);
    let repo = fixture.open();
    let before = ["feature", "second"].map(|name| {
        (
            name,
            first_change(&repo, &format!("bookmarks(exact:{name})"))
                .change_id
                .id,
        )
    });
    assert_ne!(before[0].1, before[1].1);
    let mut abandoned = fetch(&repo).abandoned_bookmarks;
    abandoned.sort();
    assert_eq!(abandoned, ["feature", "second"]);
    let ops = repo.op_log().unwrap();
    let cleanup_ops = ops
        .iter()
        .take_while(|op| !op.description.starts_with("rebase"))
        .count();
    assert_eq!(cleanup_ops, 1, "{ops:?}");
    run_jj_in(&fixture.bob, &["undo"]);
    let repo = fixture.open();
    for (name, change_id) in before {
        let restored = repo.log(&format!("bookmarks(exact:{name})")).unwrap();
        assert_eq!(restored.len(), 1);
        assert_eq!(restored[0].change_id.id, change_id);
    }
    assert_eq!(repo.log("(trunk()..@-) & empty()").unwrap().len(), 3);
}

#[test]
fn fetch_cleans_up_a_bookmarked_change_that_becomes_empty() {
    let fixture = DeletedFeatureFixture::build("feature\n", "feature\n", true);
    fs::write(fixture.alice.join("local.txt"), "keep me\n").unwrap();
    run_jj_in(&fixture.alice, &["describe", "-m", "merged local work"]);
    run_jj_in(&fixture.alice, &["bookmark", "set", "main"]);
    run_jj_in(&fixture.alice, &["git", "push", "-b", "main"]);
    let repo = fixture.open();
    assert!(!first_change(&repo, "feature").is_empty);
    assert_eq!(fetch(&repo).abandoned_bookmarks, ["feature"]);
    assert!(repo.log("(trunk()..@-) & empty()").unwrap().is_empty());
}

#[test]
fn fetch_cleans_up_a_bookmark_whose_remote_commit_was_split_locally() {
    let fixture = DeletedFeatureFixture::merged();
    run_jj_in(
        &fixture.bob,
        &[
            "split",
            "-r",
            "feature@origin",
            "-m",
            "first half",
            "root:\"feature.txt\"",
        ],
    );
    let result = fetch(&fixture.open());
    assert_eq!(result.abandoned_bookmarks, ["feature"]);
    assert!(!result.message.contains("Post-fetch cleanup failed"));
}

#[test]
fn fetch_does_not_clean_up_a_bookmark_untracked_after_opening_the_repo() {
    let fixture = DeletedFeatureFixture::merged();
    let repo = fixture.open();
    let feature = first_change(&repo, "feature").change_id.id;
    run_jj_in(&fixture.bob, &["bookmark", "untrack", "feature@origin"]);
    let result = fetch(&repo);
    assert!(result.abandoned_bookmarks.is_empty());
    assert!(result.suggest_abandon_bookmarks.is_empty());
    assert_eq!(first_change(&repo, "feature").change_id.id, feature);
}

#[test]
fn fetch_cleanup_keeps_other_workspaces_working_copies() {
    for target in ["feature", "feature@origin"] {
        let fixture = DeletedFeatureFixture::merged();
        let sibling = fixture.dir.path().join("sibling");
        run_jj_in(
            &fixture.bob,
            &[
                "workspace",
                "add",
                "--name",
                "sibling",
                sibling.to_str().unwrap(),
            ],
        );
        run_jj_in(&sibling, &["edit", "--ignore-immutable", target]);
        let before = first_change(&Repo::open(&sibling).unwrap(), "@")
            .change_id
            .id;
        let repo = fixture.open();
        let result = fetch(&repo);
        assert!(
            !result.message.contains("cleanup failed"),
            "{}",
            result.message
        );
        let copies = repo.log("working_copies()").unwrap();
        assert!(copies.iter().any(|c| c.change_id.id == before));
        if target == "feature" {
            assert!(result.abandoned_bookmarks.is_empty());
            assert_eq!(
                first_change(&repo, "bookmarks(exact:feature)").change_id.id,
                before
            );
        }
    }
}

fn fetch(repo: &Repo) -> FetchResult {
    repo.git_fetch("origin", &repo.sync_token()).unwrap()
}

/// Alice pushed `main` and `feature`; Bob tracked `feature@origin` and moved his `feature` onto local work; Alice then squash-merged, deleted `feature`, and pushed.
struct DeletedFeatureFixture {
    dir: tempfile::TempDir,
    bob: std::path::PathBuf,
    alice: std::path::PathBuf,
}

impl DeletedFeatureFixture {
    fn merged() -> Self {
        Self::build("feature\n", "feature\n", false)
    }

    fn build(local_contents: &str, merged_contents: &str, extra_work: bool) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let origin = dir.path().join("origin.git");
        let alice = dir.path().join("alice");
        let bob = dir.path().join("bob");
        run_git(dir.path(), &["init", "--bare", origin.to_str().unwrap()]);
        init_colocated(&alice);
        configure_test_user(&alice);
        fs::write(alice.join("base.txt"), "base\n").unwrap();
        run_jj_in(&alice, &["describe", "-m", "base"]);
        run_jj_in(&alice, &["bookmark", "create", "main"]);
        run_jj_in(&alice, &["new", "-m", "feature"]);
        fs::write(alice.join("feature.txt"), "feature\n").unwrap();
        run_jj_in(&alice, &["bookmark", "create", "feature"]);
        run_git(
            &alice,
            &["remote", "add", "origin", origin.to_str().unwrap()],
        );
        run_jj_in(&alice, &["git", "push", "-b", "main", "-b", "feature"]);
        run_git(&origin, &["symbolic-ref", "HEAD", "refs/heads/main"]);
        run_jj(&[
            "git",
            "clone",
            "--colocate",
            origin.to_str().unwrap(),
            bob.to_str().unwrap(),
        ]);
        configure_test_user(&bob);
        run_jj_in(&bob, &["bookmark", "track", "feature@origin"]);
        run_jj_in(&bob, &["new", "feature", "-m", "local feature"]);
        fs::write(bob.join("feature.txt"), local_contents).unwrap();
        if extra_work {
            fs::write(bob.join("local.txt"), "keep me\n").unwrap();
        }
        run_jj_in(&bob, &["bookmark", "set", "feature"]);
        run_jj_in(&bob, &["new", "-m", "ongoing work"]);
        run_jj_in(&alice, &["new", "main", "-m", "squash merged feature"]);
        fs::write(alice.join("feature.txt"), merged_contents).unwrap();
        run_jj_in(&alice, &["bookmark", "set", "main"]);
        run_jj_in(&alice, &["bookmark", "delete", "feature"]);
        run_jj_in(&alice, &["git", "push", "-b", "main", "-b", "feature"]);
        Self { dir, bob, alice }
    }

    fn open(&self) -> Repo {
        Repo::open(&self.bob).unwrap()
    }

    /// Points `refs/remotes/<remote>/<name>` in Bob's clone at the commit `feature@origin` had before the deletion.
    fn add_remote_ref(&self, remote: &str, name: &str) {
        let target = run_jj_in(
            &self.bob,
            &[
                "log",
                "-r",
                "feature@origin",
                "--no-graph",
                "-T",
                "commit_id",
            ],
        );
        if remote != "origin" {
            let url = format!("https://example.invalid/{remote}.git");
            run_git(&self.bob, &["remote", "add", remote, &url]);
        }
        let git_ref = format!("refs/remotes/{remote}/{name}");
        run_git(
            &self.bob,
            &[
                "update-ref",
                &git_ref,
                String::from_utf8(target.stdout).unwrap().trim(),
            ],
        );
    }
}
