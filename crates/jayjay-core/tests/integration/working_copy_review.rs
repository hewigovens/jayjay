use std::fs;
use std::path::{Path, PathBuf};

use jayjay_core::{WorkingCopyReview, WorkingCopyReviewMark, WorkingCopyReviewNote};
use jayjay_primitives::{
    JayResult, NoteSide, NoteStatus, ReviewFileRollup, ReviewGroupState, ReviewMarkSource,
};
use jayjay_review::ReviewStore;
use jj_test::{add_two_group_edit, init_jj_repo, run_jj_in};
use tempfile::TempDir;

fn two_group_edit() -> (TempDir, PathBuf, &'static str) {
    let temp = init_jj_repo();
    let root = temp.path().join("repo");
    let path = add_two_group_edit(&root);
    run_jj_in(&root, &["status"]);
    (temp, root, path)
}

fn mark(
    review: &WorkingCopyReview,
    path: &str,
    display_group: Option<u32>,
) -> WorkingCopyReviewMark {
    WorkingCopyReviewMark {
        expected_commit: review.commit_id().into(),
        path: path.into(),
        display_group,
        ignore_whitespace: false,
        reviewed: true,
    }
}

fn error<T>(result: JayResult<T>) -> String {
    result.err().expect("expected an error").to_string()
}

fn heads(root: &Path) -> Vec<String> {
    let mut heads: Vec<_> = fs::read_dir(root.join(".jj/repo/op_heads/heads"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    heads.sort();
    heads
}

#[test]
fn person_marks_round_trip_with_native_group_actions_without_snapshotting() {
    let (temp, root, path) = two_group_edit();
    let before = heads(&root);
    fs::write(root.join(path), "unsnapshotted agent edits\n").unwrap();
    let review = WorkingCopyReview::open(&root).unwrap();
    let mut store = ReviewStore::try_load_from(temp.path().join("review_store.json")).unwrap();
    let initial = review.status(&store, false).unwrap();
    assert_eq!(
        initial.files[0].group_states,
        [ReviewGroupState::Unreviewed; 2]
    );
    assert!(
        initial.files[0]
            .diff
            .new
            .content
            .as_ref()
            .unwrap()
            .contains("changed seven")
    );
    store
        .mark_groups_reviewed_as(
            review.change_id(),
            path,
            &initial.files[0].diff.review_identity,
            &initial.files[0].snapshot,
            &[1],
            ReviewMarkSource::Agent,
        )
        .unwrap();
    let mut mark = mark(&review, path, Some(0));
    review.set_reviewed(&mut store, &mark).unwrap();
    review.set_reviewed(&mut store, &mark).unwrap();
    assert_eq!(
        store.paths_owned_by(review.change_id(), ReviewMarkSource::User),
        [path]
    );
    assert_eq!(
        store.paths_owned_by(review.change_id(), ReviewMarkSource::Agent),
        [path]
    );
    mark.reviewed = false;
    review.set_reviewed(&mut store, &mark).unwrap();
    assert_eq!(
        review.status(&store, false).unwrap().files[0].group_states,
        [ReviewGroupState::Unreviewed, ReviewGroupState::Reviewed]
    );
    mark.display_group = None;
    mark.reviewed = true;
    review.set_reviewed(&mut store, &mark).unwrap();
    assert!(
        store
            .paths_owned_by(review.change_id(), ReviewMarkSource::Agent)
            .is_empty()
    );
    let file = &review.status(&store, false).unwrap().files[0];
    assert_eq!(file.rollup, ReviewFileRollup::Reviewed);
    assert_eq!(file.source, ReviewMarkSource::User);
    let mut app_store = ReviewStore::load_from(temp.path().join("review_store.json"));
    app_store.toggle_display_group_snapshot(
        review.change_id(),
        path,
        &file.diff.review_identity,
        &file.snapshot,
        &file.display_groups,
        1,
    );
    let refreshed = ReviewStore::try_load_from(temp.path().join("review_store.json")).unwrap();
    assert_eq!(
        WorkingCopyReview::open(&root)
            .unwrap()
            .status(&refreshed, false)
            .unwrap()
            .files[0]
            .rollup,
        ReviewFileRollup::Partial
    );
    assert_eq!(heads(&root), before);
}

#[test]
fn stale_commits_invalid_groups_and_failed_saves_do_not_change_marks() {
    let (temp, root, path) = two_group_edit();
    let review = WorkingCopyReview::open(&root).unwrap();
    let store_path = temp.path().join("review_store.json");
    let mut store = ReviewStore::try_load_from(store_path.clone()).unwrap();
    let mut mark = mark(&review, path, Some(99));
    assert!(review.set_reviewed(&mut store, &mark).is_err());
    assert!(!store_path.exists());
    mark.display_group = None;
    review.set_reviewed(&mut store, &mark).unwrap();
    let saved = fs::read(&store_path).unwrap();
    fs::remove_file(&store_path).unwrap();
    fs::create_dir(&store_path).unwrap();
    mark.reviewed = false;
    assert!(review.set_reviewed(&mut store, &mark).is_err());
    fs::remove_dir(&store_path).unwrap();
    fs::write(&store_path, &saved).unwrap();
    run_jj_in(
        &root,
        &["--ignore-working-copy", "describe", "-m", "new metadata"],
    );
    let before = heads(&root);
    assert!(error(review.set_reviewed(&mut store, &mark)).contains("changed since inspection"));
    assert!(
        WorkingCopyReview::open(&root)
            .unwrap()
            .set_reviewed(&mut store, &mark)
            .is_err()
    );
    assert_eq!(fs::read(&store_path).unwrap(), saved);
    assert_eq!(heads(&root), before);
}

#[test]
fn notes_anchor_to_the_displayed_line_and_reject_stale_commits() {
    let (temp, root, path) = two_group_edit();
    let review = WorkingCopyReview::open(&root).unwrap();
    let store_path = temp.path().join("review_store.json");
    let mut store = ReviewStore::try_load_from(store_path.clone()).unwrap();
    let mut note = WorkingCopyReviewNote {
        expected_commit: review.commit_id().into(),
        path: path.into(),
        side: NoteSide::New,
        line: 7,
        body: "  Why seven?  ".into(),
        ignore_whitespace: false,
    };
    let saved = review.add_note(&mut store, &note).unwrap();
    assert_eq!(
        (saved.body.as_str(), saved.anchor_excerpt.as_str()),
        ("Why seven?", "changed seven")
    );
    let notes = review
        .notes(&ReviewStore::try_load_from(store_path.clone()).unwrap())
        .unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].status, NoteStatus::Current);

    note.line = 2;
    assert!(
        review.add_note(&mut store, &note).is_err(),
        "line 2 is unchanged context"
    );

    review.resolve_note(&mut store, &saved.id).unwrap();
    assert!(review.notes(&store).unwrap()[0].note.resolved);
    assert!(review.resolve_note(&mut store, "no-such-note").is_err());

    run_jj_in(
        &root,
        &["--ignore-working-copy", "describe", "-m", "new metadata"],
    );
    note.line = 7;
    assert!(error(review.add_note(&mut store, &note)).contains("changed since inspection"));
}

#[test]
fn an_unpushed_ancestor_is_reviewed_under_its_own_change_until_rewritten() {
    let temp = init_jj_repo();
    let root = temp.path().join("repo");
    let path = add_two_group_edit(&root);
    run_jj_in(&root, &["new", "-m", "on top"]);
    let log = run_jj_in(&root, &["log", "-r", "@-", "--no-graph", "-T", "change_id"]);
    let fixture = String::from_utf8(log.stdout).unwrap().trim().to_owned();
    run_jj_in(&root, &["bookmark", "create", "kkkk", "-r", "@-"]);
    assert!(
        WorkingCopyReview::open_change(&root, "kkkk").is_err(),
        "a bookmark name is not a change id"
    );
    let review = WorkingCopyReview::open_change(&root, &fixture[..8]).unwrap();
    assert_eq!(review.change_id(), fixture);
    assert_eq!(review.files()[0].path, path);
    let store_path = temp.path().join("review_store.json");
    let mut store = ReviewStore::try_load_from(store_path).unwrap();
    let mark = mark(&review, path, Some(1));
    review.set_reviewed(&mut store, &mark).unwrap();
    assert_eq!(
        review.status(&store, false).unwrap().files[0].group_states,
        [ReviewGroupState::Unreviewed, ReviewGroupState::Reviewed]
    );
    run_jj_in(&root, &["describe", "-r", &fixture, "-m", "reworded"]);
    assert!(
        error(review.set_reviewed(&mut store, &mark))
            .contains("The change changed since inspection")
    );
    assert!(
        WorkingCopyReview::open_change(&root, "zzzzzzzz").is_err(),
        "the root change is immutable"
    );
    assert!(
        WorkingCopyReview::open_change(&root, "@-").is_err(),
        "only change ids, not revsets"
    );
}

#[test]
fn concurrent_operations_are_rejected_without_reconciling_them() {
    let temp = init_jj_repo();
    let root = temp.path().join("repo");
    let base = heads(&root).pop().unwrap();
    for description in ["writer one", "writer two"] {
        run_jj_in(
            &root,
            &[
                "--ignore-working-copy",
                "--at-op",
                &base,
                "describe",
                "-m",
                description,
            ],
        );
    }
    let before = heads(&root);
    assert_eq!(before.len(), 2);
    assert!(error(WorkingCopyReview::open(&root)).contains("Concurrent repository operations"));
    assert_eq!(heads(&root), before);
}

#[test]
fn inherited_conflicts_keep_the_rest_of_the_change_reviewable() {
    let temp = init_jj_repo();
    let root = temp.path().join("repo");
    for (message, content, next) in [
        ("base", "base\n", &["new", "@"][..]),
        ("left", "left\n", &["new", "@-"][..]),
        ("right", "right\n", &["new", "left", "@"][..]),
    ] {
        fs::write(root.join("conflict.txt"), content).unwrap();
        run_jj_in(&root, &["describe", "-m", message]);
        if message == "left" {
            run_jj_in(&root, &["bookmark", "create", "left", "-r", "@"]);
        }
        run_jj_in(&root, next);
    }
    run_jj_in(&root, &["new"]);
    fs::write(root.join("notes.txt"), "reviewable\n").unwrap();
    run_jj_in(&root, &["status"]);

    let review = WorkingCopyReview::open(&root).unwrap();
    let mut store = ReviewStore::try_load_from(temp.path().join("review_store.json")).unwrap();
    let status = review.status(&store, false).unwrap();
    let conflict = status
        .files
        .iter()
        .find(|file| file.diff.path == "conflict.txt")
        .unwrap();
    assert!(conflict.diff.is_conflict_only_placeholder());
    assert!(conflict.display_groups.is_empty());
    assert!(
        status
            .files
            .iter()
            .any(|file| file.diff.path == "notes.txt")
    );
    review
        .set_reviewed(&mut store, &mark(&review, "notes.txt", None))
        .unwrap();
    assert!(
        error(review.set_reviewed(&mut store, &mark(&review, "conflict.txt", None)))
            .contains("cannot carry review marks")
    );
}
