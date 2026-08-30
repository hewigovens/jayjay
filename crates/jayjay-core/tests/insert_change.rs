use std::fs;

use jayjay_core::{InsertPosition, Repo};
use jj_test::{change_by_description, current_op_id, init_jj_repo, run_git, run_jj, run_jj_in};

#[test]
fn new_change_before_moves_the_working_copy_under_the_target_and_keeps_its_content() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");
    fs::write(repo_path.join("work.txt"), "work in progress\n").expect("write work.txt");
    repo.refresh_working_copy().expect("snapshot work.txt");
    repo.describe("@", "target").expect("describe target");
    let ops_before = repo.op_log().expect("op log").len();

    repo.new_change_inserted("@", InsertPosition::Before, "prerequisite")
        .expect("insert before the working copy");

    let changes = repo.log("all()").expect("log all");
    let inserted = change_by_description(&changes, "prerequisite");
    let target = change_by_description(&changes, "target");
    assert!(inserted.is_working_copy && inserted.is_empty);
    assert_eq!(inserted.parents, vec!["0".repeat(40)]);
    assert_eq!(target.parents, vec![inserted.commit_id.id.clone()]);
    let target_diff = repo.show(&target.commit_id).expect("show target").diff;
    assert!(
        target_diff.iter().any(|hunk| hunk.path == "work.txt"),
        "the displaced change must keep its content"
    );
    assert!(
        !repo_path.join("work.txt").exists(),
        "the checkout must follow the working copy below the target"
    );
    assert_eq!(repo.op_log().expect("op log").len(), ops_before + 1);
}

#[test]
fn insert_after_keeps_a_merge_childs_unrelated_parent() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    run_jj_in(&repo_path, &["describe", "-m", "target"]);
    run_jj_in(&repo_path, &["new", "-m", "other", "root()"]);
    run_jj_in(
        &repo_path,
        &[
            "new",
            "-m",
            "merge",
            "subject(\"target\")",
            "subject(\"other\")",
        ],
    );
    let repo = Repo::open(&repo_path).expect("open repo");

    repo.new_change_inserted("subject(\"target\")", InsertPosition::After, "inserted")
        .expect("insert after the merge child's first parent");

    let changes = repo.log("all()").expect("log all");
    let inserted = change_by_description(&changes, "inserted");
    let target = change_by_description(&changes, "target");
    let other = change_by_description(&changes, "other");
    assert_eq!(inserted.parents, vec![target.commit_id.id.clone()]);
    assert_eq!(
        change_by_description(&changes, "merge").parents,
        vec![inserted.commit_id.id.clone(), other.commit_id.id.clone()],
        "the merge child must keep its unrelated parent"
    );
}

#[test]
fn insert_follows_a_working_copy_rewritten_by_the_snapshot() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo_str = repo_path.to_str().expect("repo path utf-8");
    let base_op = current_op_id(&repo_path);
    fs::write(repo_path.join("hello.txt"), "left\n").expect("write left");
    run_jj(&["-R", repo_str, "describe", "-m", "left"]);
    fs::write(repo_path.join("hello.txt"), "right\n").expect("write right");
    run_jj(&[
        "-R", repo_str, "--at-op", &base_op, "describe", "-m", "right",
    ]);

    let repo = Repo::open(&repo_path).expect("open repo");
    let stale = repo
        .log("all()")
        .expect("log")
        .into_iter()
        .find(|change| change.is_working_copy)
        .expect("working copy in log");
    assert!(stale.is_divergent, "fixture working copy must be divergent");
    fs::write(repo_path.join("work.txt"), "fresh edit\n").expect("write unsnapshotted edit");

    repo.new_change_inserted(&stale.commit_id.id, InsertPosition::Before, "prereq")
        .expect("insert before the stale working-copy commit id");

    let changes = repo.log("all()").expect("log after insert");
    let siblings: Vec<_> = changes
        .iter()
        .filter(|change| change.change_id.id == stale.change_id.id)
        .collect();
    assert_eq!(
        siblings.len(),
        2,
        "the hidden pre-snapshot commit must not be resurrected"
    );
    let inserted = change_by_description(&changes, "prereq");
    assert!(inserted.is_working_copy);
    let moved = siblings
        .iter()
        .find(|change| change.parents == vec![inserted.commit_id.id.clone()])
        .expect("the displaced sibling sits on the inserted change");
    let diff = repo.show(&moved.commit_id).expect("show displaced").diff;
    assert!(
        diff.iter().any(|hunk| hunk.path == "work.txt"),
        "the freshly snapshotted edit must stay in the displaced change"
    );
}

#[test]
fn on_top_hides_only_for_a_discardable_working_copy() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    run_jj_in(&repo_path, &["new"]);
    let repo = Repo::open(&repo_path).expect("open repo");

    let wc = |changes: &[jayjay_core::ChangeInfo]| {
        changes
            .iter()
            .find(|change| change.is_working_copy)
            .expect("working copy in log")
            .clone()
    };
    assert!(!wc(&repo.log("all()").expect("log")).new_change.on_top);
    assert!(
        change_by_description(&repo.log("all()").expect("log"), "initial change")
            .new_change
            .on_top,
        "a described change keeps new change on top"
    );

    run_jj_in(&repo_path, &["bookmark", "create", "wip", "-r", "@"]);
    let repo = Repo::open(&repo_path).expect("reopen repo");
    let referenced = wc(&repo.log("all()").expect("log"));
    assert!(
        referenced.new_change.on_top,
        "a referenced working copy is not discardable"
    );

    run_jj_in(&repo_path, &["bookmark", "delete", "wip"]);
    run_git(
        &repo_path,
        &[
            "update-ref",
            "refs/remotes/origin/keeper",
            &referenced.commit_id.id,
        ],
    );
    run_jj_in(&repo_path, &["st"]);
    run_jj_in(&repo_path, &["bookmark", "track", "keeper@origin"]);
    run_jj_in(&repo_path, &["bookmark", "delete", "keeper"]);
    run_jj_in(&repo_path, &["edit", &referenced.commit_id.id]);
    let repo = Repo::open(&repo_path).expect("reopen repo after remote ref");
    let retained = wc(&repo.log("all()").expect("log"));
    assert_eq!(retained.commit_id.id, referenced.commit_id.id);
    assert!(
        retained.new_change.on_top,
        "a tracked remote bookmark retains the working copy"
    );
}

#[test]
fn insert_after_gate_counts_children_hidden_from_the_revset() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    run_jj_in(&repo_path, &["describe", "-m", "protected"]);
    run_jj_in(&repo_path, &["new", "-m", "child"]);
    run_jj_in(&repo_path, &["new", "-m", "grandchild"]);
    run_git(&repo_path, &["tag", "release"]);
    run_jj_in(&repo_path, &["st"]);
    let repo = Repo::open(&repo_path).expect("open repo");

    let protected = repo
        .log("subject(\"protected\")")
        .expect("log protected only");
    assert!(
        !protected[0].new_change.after,
        "the immutable child stays counted while the revset hides it"
    );
    let changes = repo.log("all()").expect("log all");
    let child = change_by_description(&changes, "child");
    assert!(child.is_immutable && child.new_change.after);
    let grandchild = change_by_description(&changes, "grandchild");
    assert!(
        !grandchild.new_change.after,
        "a head has nothing to insert after"
    );
}

#[test]
fn new_change_after_reparents_every_child_onto_the_inserted_change() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    run_jj_in(&repo_path, &["describe", "-m", "base"]);
    run_jj_in(&repo_path, &["new", "-m", "left"]);
    run_jj_in(&repo_path, &["new", "-m", "right", "@-"]);
    let repo = Repo::open(&repo_path).expect("open repo");

    repo.new_change_inserted("@-", InsertPosition::After, "inserted")
        .expect("insert after base");

    let changes = repo.log("all()").expect("log all");
    let base = change_by_description(&changes, "base");
    let inserted = change_by_description(&changes, "inserted");
    assert!(inserted.is_working_copy);
    assert_eq!(inserted.parents, vec![base.commit_id.id.clone()]);
    for child in ["left", "right"] {
        assert_eq!(
            change_by_description(&changes, child).parents,
            vec![inserted.commit_id.id.clone()],
            "{child} must now descend from the inserted change"
        );
    }
}
