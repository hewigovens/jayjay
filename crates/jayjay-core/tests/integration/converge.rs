use std::fs;
use std::path::PathBuf;

use jayjay_core::{
    ConvergeAuthorAttribute, ConvergeChoices, ConvergeDescriptionAttribute,
    ConvergeParentsAttribute, DescriptionChoice, JayError, Repo,
};
use jj_test::{current_op_id, init_jj_repo, run_jj_in};
use tempfile::TempDir;

#[test]
fn converge_rewrites_versions_refs_descendants_and_undoes_once() {
    let (_temp, path) = divergent_repo();
    let repo = Repo::open(&path).unwrap();
    let plan = repo.converge_preview("subject(left)").unwrap();
    let left = plan
        .versions
        .iter()
        .find(|version| version.description.trim() == "left")
        .unwrap()
        .commit_id
        .clone();
    let right = plan
        .versions
        .iter()
        .find(|version| version.description.trim() == "right")
        .unwrap()
        .commit_id
        .clone();
    run_jj_in(&path, &["new", &left, "-m", "child"]);
    run_jj_in(&path, &["bookmark", "create", "kept", "-r", &right]);
    run_jj_in(&path, &["edit", &right]);
    let repo = Repo::open(&path).unwrap();
    let plan = repo.converge_preview(&plan.change_id).unwrap();
    assert_eq!(plan.versions.len(), 2);
    assert!(matches!(
        plan.author,
        ConvergeAuthorAttribute::Solved { .. }
    ));
    assert!(matches!(
        plan.parents,
        ConvergeParentsAttribute::Solved { .. }
    ));
    assert!(matches!(
        plan.description,
        ConvergeDescriptionAttribute::Unsolved
    ));
    assert_eq!(plan.effects.descendants.len(), 1);
    assert_eq!(plan.effects.descendants[0].description, "child");
    assert_eq!(plan.effects.bookmarks, ["kept"]);
    assert_eq!(plan.effects.workspaces, ["default"]);
    let before = repo.op_log().unwrap();
    let outcome = repo
        .converge_apply(
            &plan,
            &ConvergeChoices {
                description: Some(DescriptionChoice::FromVersion(right.clone())),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(outcome.conflicted_paths.is_empty());
    let versions = repo.log(&format!("change_id({})", plan.change_id)).unwrap();
    assert_eq!(versions.len(), 1);
    assert_eq!(versions[0].commit_id.id, outcome.successor_commit_id);
    assert_eq!(versions[0].description, "right\n");
    assert_eq!(
        repo.log("kept").unwrap()[0].commit_id.id,
        outcome.successor_commit_id
    );
    assert_eq!(
        repo.log("@").unwrap()[0].commit_id.id,
        outcome.successor_commit_id
    );
    assert_eq!(
        repo.log("subject(child)").unwrap()[0].parents,
        std::slice::from_ref(&outcome.successor_commit_id)
    );
    let evolog = run_jj_in(
        &path,
        &[
            "evolog",
            "--no-graph",
            "-r",
            &outcome.successor_commit_id,
            "-T",
            "commit.commit_id() ++ '\n'",
        ],
    );
    let evolution = String::from_utf8(evolog.stdout).unwrap();
    assert!(evolution.lines().any(|line| line == left));
    assert!(evolution.lines().any(|line| line == right));
    let after = repo.op_log().unwrap();
    assert_eq!(after.len(), before.len() + 1);
    assert!(
        after[0]
            .description
            .starts_with("converge divergent change ")
    );
    run_jj_in(&path, &["undo"]);
    let restored = Repo::open(&path).unwrap();
    let mut ids: Vec<_> = restored
        .log(&format!("change_id({})", plan.change_id))
        .unwrap()
        .into_iter()
        .map(|change| change.commit_id.id)
        .collect();
    ids.sort();
    assert_eq!(ids, plan.basis);
}

#[test]
fn converge_rejects_stale_versions_and_invalid_choices_without_an_operation() {
    let (_temp, path) = divergent_repo();
    let repo = Repo::open(&path).unwrap();
    let plan = repo.converge_preview("subject(left)").unwrap();
    let valid = ConvergeChoices {
        description: Some(DescriptionChoice::Custom("merged".into())),
        ..Default::default()
    };
    for (choices, message) in [
        (ConvergeChoices::default(), "description choice is missing"),
        (
            ConvergeChoices {
                description: Some(DescriptionChoice::FromVersion("root()".into())),
                ..Default::default()
            },
            "root() is not a version",
        ),
        (
            ConvergeChoices {
                contents: Some("root()".into()),
                ..valid.clone()
            },
            "root() is not a version",
        ),
    ] {
        let before = current_op_id(&path);
        assert!(matches!(repo.converge_apply(&plan, &choices),
            Err(JayError::Internal { message: actual }) if actual.contains(message)));
        assert_eq!(current_op_id(&path), before);
    }
    let mut changed_attributes = plan.clone();
    changed_attributes.description = ConvergeDescriptionAttribute::Solved {
        value: "stale description".into(),
    };
    let before = current_op_id(&path);
    assert!(matches!(repo.converge_apply(&changed_attributes, &valid),
        Err(JayError::ConvergePlanStale { change_id }) if change_id == plan.change_id));
    assert_eq!(current_op_id(&path), before);
    run_jj_in(
        &path,
        &["new", "--no-edit", &plan.basis[0], "-m", "late child"],
    );
    let before = current_op_id(&path);
    assert!(matches!(repo.converge_apply(&plan, &valid),
        Err(JayError::ConvergePlanStale { change_id }) if change_id == plan.change_id));
    assert_eq!(current_op_id(&path), before);
    run_jj_in(
        &path,
        &["describe", "-r", &plan.basis[0], "-m", "changed again"],
    );
    let before = current_op_id(&path);
    assert!(matches!(repo.converge_apply(&plan, &valid),
        Err(JayError::ConvergePlanStale { change_id }) if change_id == plan.change_id));
    assert_eq!(current_op_id(&path), before);
    let refreshed = repo.converge_preview(&plan.change_id).unwrap();
    repo.converge_apply(&refreshed, &valid).unwrap();
    let before = current_op_id(&path);
    assert!(matches!(repo.converge_apply(&refreshed, &valid),
        Err(JayError::ConvergePlanStale { change_id }) if change_id == plan.change_id));
    assert_eq!(current_op_id(&path), before);
}

#[test]
fn converge_blocks_immutable_versions_and_newly_immutable_descendants() {
    for descendant in [false, true] {
        let (_temp, path) = divergent_repo();
        run_jj_in(&path, &["new", "subject(left)", "-m", "child"]);
        run_jj_in(&path, &["new", "root()", "-m", "unrelated"]);
        let repo = Repo::open(&path).unwrap();
        let reviewed = repo.converge_preview("subject(left)").unwrap();
        let target = if descendant {
            repo.log("subject(child)").unwrap()[0].commit_id.id.clone()
        } else {
            reviewed.basis[0].clone()
        };
        run_jj_in(
            &path,
            &[
                "config",
                "set",
                "--repo",
                "revset-aliases.\"immutable_heads()\"",
                &target,
            ],
        );
        let fresh = Repo::open(&path).unwrap();
        let blocked = fresh.converge_preview(&reviewed.change_id).unwrap();
        assert!(
            blocked
                .effects
                .blocked_reason
                .as_ref()
                .unwrap()
                .contains(&target)
        );
        let before = current_op_id(&path);
        let choices = ConvergeChoices {
            description: Some(DescriptionChoice::Custom("merged".into())),
            ..Default::default()
        };
        for plan in [&reviewed, &blocked] {
            assert!(matches!(repo.converge_apply(plan, &choices),
                Err(JayError::ConvergeBlocked { change_id, reason })
                    if change_id == plan.change_id && reason.contains(&target)));
            assert_eq!(current_op_id(&path), before);
        }
    }
}

#[test]
fn converge_chosen_parents_recompute_the_tree_and_preserve_conflicts() {
    let temp = init_jj_repo();
    let path = temp.path().join("repo");
    run_jj_in(&path, &["new", "-m", "parent-left"]);
    fs::write(path.join("left.txt"), "left parent\n").unwrap();
    run_jj_in(&path, &["new", "root()", "-m", "parent-right"]);
    fs::write(path.join("right.txt"), "right parent\n").unwrap();
    run_jj_in(
        &path,
        &["new", "subject(\"initial change\")", "-m", "target"],
    );
    let base_op = current_op_id(&path);
    fs::write(path.join("hello.txt"), "left version\n").unwrap();
    run_jj_in(&path, &["rebase", "-r", "@", "-d", "subject(parent-left)"]);
    run_jj_in(
        &path,
        &[
            "--at-op",
            &base_op,
            "rebase",
            "-r",
            "@",
            "-d",
            "subject(parent-right)",
        ],
    );
    fs::write(path.join("hello.txt"), "right version\n").unwrap();
    run_jj_in(&path, &["describe", "-m", "target"]);
    run_jj_in(&path, &["status"]);
    let repo = Repo::open(&path).unwrap();
    let plan = repo.converge_preview("subject(target)").unwrap();
    assert_eq!(plan.conflicted_paths, None);
    assert!(matches!(plan.parents, ConvergeParentsAttribute::Unsolved));
    let parent = repo.log("subject(parent-right)").unwrap()[0]
        .commit_id
        .id
        .clone();
    let choice = plan
        .versions
        .iter()
        .find(|version| version.parents == std::slice::from_ref(&parent))
        .unwrap()
        .commit_id
        .clone();
    let outcome = repo
        .converge_apply(
            &plan,
            &ConvergeChoices {
                parents: Some(choice.clone()),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(
        repo.log(&outcome.successor_commit_id).unwrap()[0].parents,
        [parent]
    );
    assert_eq!(
        repo.file_content(&outcome.successor_commit_id, "right.txt")
            .unwrap(),
        "right parent\n"
    );
    assert_eq!(
        repo.file_content(&outcome.successor_commit_id, "left.txt")
            .unwrap(),
        ""
    );
    assert_eq!(outcome.conflicted_paths, ["hello.txt"]);
    assert_eq!(
        repo.resolve_list(&outcome.successor_commit_id).unwrap(),
        outcome.conflicted_paths
    );
    run_jj_in(&path, &["undo"]);
    let repo = Repo::open(&path).unwrap();
    let plan = repo.converge_preview("subject(target)").unwrap();
    let left = plan
        .versions
        .iter()
        .find(|version| version.commit_id != choice)
        .unwrap()
        .commit_id
        .clone();
    let outcome = repo
        .converge_apply(
            &plan,
            &ConvergeChoices {
                parents: Some(choice),
                contents: Some(left),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(outcome.conflicted_paths, ["hello.txt"]);
    for (file, content) in [("right.txt", "right parent\n"), ("left.txt", "")] {
        assert_eq!(
            repo.file_content(&outcome.successor_commit_id, file)
                .unwrap(),
            content
        );
    }
}

#[test]
fn converge_rejects_nondivergence_but_accepts_unrelated_operations() {
    let temp = init_jj_repo();
    let path = temp.path().join("repo");
    let repo = Repo::open(&path).unwrap();
    assert!(matches!(
        repo.converge_preview("@"),
        Err(JayError::ConvergeNotDivergent { .. })
    ));
    let base_op = current_op_id(&path);
    run_jj_in(&path, &["describe", "-m", "left"]);
    run_jj_in(&path, &["--at-op", &base_op, "describe", "-m", "right"]);
    run_jj_in(&path, &["status"]);
    let repo = Repo::open(&path).unwrap();
    let plan = repo.converge_preview("subject(left)").unwrap();
    run_jj_in(&path, &["new", "root()", "-m", "unrelated"]);
    let outcome = repo
        .converge_apply(
            &plan,
            &ConvergeChoices {
                description: Some(DescriptionChoice::Custom("merged".into())),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(
        repo.log(&outcome.successor_commit_id).unwrap()[0].description,
        "merged"
    );
    assert_eq!(repo.log("@").unwrap()[0].description.trim(), "unrelated");
}

#[test]
fn converge_chosen_versions_override_solved_attributes_and_contents() {
    let temp = init_jj_repo();
    let path = temp.path().join("repo");
    let base_op = current_op_id(&path);
    fs::write(path.join("hello.txt"), "edited\n").unwrap();
    run_jj_in(&path, &["metaedit", "--author", "Left <left@example.com>"]);
    run_jj_in(
        &path,
        &[
            "--at-op",
            &base_op,
            "metaedit",
            "--author",
            "Right <right@example.com>",
        ],
    );
    run_jj_in(&path, &["status"]);
    let repo = Repo::open(&path).unwrap();
    let plan = repo.converge_preview("divergent()").unwrap();
    assert!(matches!(plan.author, ConvergeAuthorAttribute::Unsolved));
    assert!(matches!(
        plan.description,
        ConvergeDescriptionAttribute::Solved { .. }
    ));
    let version = |email: &str| {
        plan.versions
            .iter()
            .find(|version| version.author.email == email)
            .unwrap()
            .commit_id
            .clone()
    };
    let (left, right) = (version("left@example.com"), version("right@example.com"));
    assert_eq!(plan.contents_source.as_ref(), Some(&left));
    let outcome = repo
        .converge_apply(
            &plan,
            &ConvergeChoices {
                author: Some(right.clone()),
                description: Some(DescriptionChoice::Custom("override".into())),
                contents: Some(right),
                ..Default::default()
            },
        )
        .unwrap();
    let commit = repo.log(&outcome.successor_commit_id).unwrap().remove(0);
    assert_eq!(commit.author.email, "right@example.com");
    assert_eq!(commit.description, "override");
    assert_eq!(
        repo.file_content(&outcome.successor_commit_id, "hello.txt")
            .unwrap(),
        "hello from jayjay\n"
    );
}

#[test]
fn converge_preview_snapshots_edits_before_the_first_apply() {
    let (_temp, path) = divergent_repo();
    run_jj_in(&path, &["edit", "subject(right)"]);
    let repo = Repo::open(&path).unwrap();
    fs::write(path.join("pending.txt"), "unsnapshotted content\n").unwrap();
    let plan = repo.converge_preview("@").unwrap();
    let outcome = repo
        .converge_apply(
            &plan,
            &ConvergeChoices {
                description: Some(DescriptionChoice::Custom("merged".into())),
                ..Default::default()
            },
        )
        .expect("first apply must use the snapshotted preview basis");
    assert_eq!(
        repo.file_content(&outcome.successor_commit_id, "pending.txt")
            .unwrap(),
        "unsnapshotted content\n"
    );
}

fn divergent_repo() -> (TempDir, PathBuf) {
    let temp = init_jj_repo();
    let path = temp.path().join("repo");
    let base_op = current_op_id(&path);
    run_jj_in(&path, &["describe", "-m", "left"]);
    run_jj_in(&path, &["--at-op", &base_op, "describe", "-m", "right"]);
    run_jj_in(&path, &["status"]);
    (temp, path)
}
