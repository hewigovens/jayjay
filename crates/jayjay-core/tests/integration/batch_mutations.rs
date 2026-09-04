use std::fs;

use jayjay_core::{ChangeInfo, Repo};
use jj_test::{current_op_id, init_jj_repo, run_jj, run_jj_in};

fn change_by_description(repo: &Repo, description: &str) -> ChangeInfo {
    repo.log("all()")
        .expect("load changes")
        .into_iter()
        .find(|change| change.description.trim() == description)
        .unwrap_or_else(|| panic!("missing change {description:?}"))
}

#[test]
fn abandon_many_removes_selection_and_reparents_descendants() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    run_jj_in(&repo_path, &["describe", "-m", "base"]);
    run_jj_in(&repo_path, &["new", "-m", "first"]);
    run_jj_in(&repo_path, &["new", "-m", "second"]);
    run_jj_in(&repo_path, &["new", "-m", "tip"]);
    let repo = Repo::open(&repo_path).expect("open repo");

    repo.abandon_many(&[
        change_by_description(&repo, "first").change_id.id,
        change_by_description(&repo, "second").change_id.id,
    ])
    .expect("abandon selected changes");

    let changes = repo.log("all()").expect("load rewritten changes");
    assert!(
        changes
            .iter()
            .all(|change| !matches!(change.description.trim(), "first" | "second"))
    );
    let base = changes
        .iter()
        .find(|change| change.description.trim() == "base")
        .expect("base remains");
    let tip = changes
        .iter()
        .find(|change| change.description.trim() == "tip")
        .expect("tip remains");
    assert_eq!(tip.parents, vec![base.commit_id.id.clone()]);
}

#[test]
fn abandon_many_targets_a_divergent_working_copy_after_cli_snapshot() {
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
    let changes = repo.log("all()").expect("load divergent changes");
    let working_copy = changes
        .iter()
        .find(|change| change.is_working_copy)
        .expect("working copy");
    assert!(
        working_copy.is_divergent,
        "fixture working copy must be divergent"
    );
    let sibling = changes
        .iter()
        .find(|change| {
            change.change_id == working_copy.change_id && change.commit_id != working_copy.commit_id
        })
        .expect("divergent sibling");
    fs::write(repo_path.join("fresh.txt"), "unsnapshotted\n").expect("write fresh edit");

    repo.abandon_many(&[
        working_copy.commit_id.id.clone(),
        sibling.commit_id.id.clone(),
    ])
    .expect("abandon both divergent versions");

    let remaining = repo.log("all()").expect("load remaining changes");
    assert!(
        remaining
            .iter()
            .all(|change| change.change_id != working_copy.change_id),
        "the freshly snapshotted working-copy version must be targeted instead of its predecessor"
    );
    assert!(
        !repo_path.join("fresh.txt").exists(),
        "abandoning the selected working copy must remove its freshly snapshotted content"
    );
}

#[test]
fn abandon_many_targets_the_selection_not_a_moved_working_copy() {
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
    let changes = repo.log("all()").expect("load divergent changes");
    let working_copy = changes
        .iter()
        .find(|change| change.is_working_copy)
        .expect("working copy");
    let sibling = changes
        .iter()
        .find(|change| {
            change.change_id == working_copy.change_id && change.commit_id != working_copy.commit_id
        })
        .expect("divergent sibling");

    // A concurrent actor moves @ onto a fresh child after the user made their selection.
    run_jj(&["-R", repo_str, "new", "-m", "external"]);
    fs::write(repo_path.join("external.txt"), "keep me\n").expect("write external edit");

    repo.abandon_many(&[
        working_copy.commit_id.id.clone(),
        sibling.commit_id.id.clone(),
    ])
    .expect("abandon the selected divergent versions");

    let remaining = repo.log("all()").expect("load remaining changes");
    assert!(
        remaining
            .iter()
            .all(|change| change.change_id != working_copy.change_id),
        "both selected versions must be abandoned"
    );
    assert!(
        remaining
            .iter()
            .any(|change| change.description.trim() == "external"),
        "the concurrently created working copy must survive"
    );
    assert!(
        repo_path.join("external.txt").exists(),
        "the concurrent actor's edit must not be abandoned with the selection"
    );
}

#[test]
fn rebase_many_preserves_dependencies_within_the_selection() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    run_jj_in(&repo_path, &["describe", "-m", "base"]);
    run_jj_in(&repo_path, &["bookmark", "create", "base", "-r", "@"]);
    run_jj_in(&repo_path, &["new", "-m", "selected root"]);
    run_jj_in(&repo_path, &["new", "-m", "selected tip"]);
    run_jj_in(&repo_path, &["new", "base", "-m", "destination"]);
    let repo = Repo::open(&repo_path).expect("open repo");

    repo.rebase_many(
        &[
            change_by_description(&repo, "selected tip").change_id.id,
            change_by_description(&repo, "selected root").change_id.id,
        ],
        &change_by_description(&repo, "destination").change_id.id,
    )
    .expect("rebase selected changes");

    let destination = change_by_description(&repo, "destination");
    let selected_root = change_by_description(&repo, "selected root");
    let selected_tip = change_by_description(&repo, "selected tip");
    assert_eq!(selected_root.parents, vec![destination.commit_id.id]);
    assert_eq!(selected_tip.parents, vec![selected_root.commit_id.id]);
}

#[test]
fn squash_many_combines_a_consecutive_linear_range_into_its_oldest_change() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    fs::write(repo_path.join("oldest.txt"), "oldest\n").expect("write oldest");
    run_jj_in(&repo_path, &["describe", "-m", "oldest"]);
    run_jj_in(&repo_path, &["new", "-m", "middle"]);
    fs::write(repo_path.join("middle.txt"), "middle\n").expect("write middle");
    run_jj_in(&repo_path, &["new", "-m", "newest"]);
    fs::write(repo_path.join("newest.txt"), "newest\n").expect("write newest");
    run_jj_in(&repo_path, &["st"]);
    let repo = Repo::open(&repo_path).expect("open repo");
    let oldest = change_by_description(&repo, "oldest");
    let middle = change_by_description(&repo, "middle");
    let newest = change_by_description(&repo, "newest");

    let err = repo
        .squash_many(&[newest.change_id.id.clone(), oldest.change_id.id.clone()])
        .expect_err("a selection with a gap must be rejected");
    assert!(
        err.to_string().contains("consecutive linear range"),
        "{err}"
    );

    let destination = repo
        .squash_many(&[
            newest.change_id.id,
            middle.change_id.id,
            oldest.change_id.id.clone(),
        ])
        .expect("squash selected range");

    let squashed = repo
        .show(&oldest.change_id.id)
        .expect("show squashed change");
    assert_eq!(squashed.info.commit_id.id, destination);
    assert_eq!(squashed.info.description.trim(), "oldest\nmiddle\nnewest");
    let paths = squashed
        .diff
        .iter()
        .map(|hunk| hunk.path.as_str())
        .collect::<Vec<_>>();
    for path in ["middle.txt", "newest.txt", "oldest.txt"] {
        assert!(paths.contains(&path), "missing {path}: {paths:?}");
    }
}
