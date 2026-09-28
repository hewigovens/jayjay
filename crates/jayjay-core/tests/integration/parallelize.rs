use std::fs;

use jayjay_core::{ChangeInfo, MutationEffect, Repo};
use jj_test::{change_by_description, run_jj_in, setup_stack};

fn changes(repo: &Repo) -> Vec<ChangeInfo> {
    repo.log("all()").expect("load changes")
}

fn commit_ids(repo: &Repo) -> Vec<String> {
    changes(repo)
        .into_iter()
        .map(|change| change.commit_id.id)
        .collect()
}

fn parallelize(repo: &Repo, descriptions: &[&str]) -> MutationEffect {
    let changes = changes(repo);
    let revs: Vec<String> = descriptions
        .iter()
        .map(|description| {
            change_by_description(&changes, description)
                .change_id
                .id
                .clone()
        })
        .collect();
    repo.parallelize(&revs).expect("parallelize")
}

#[test]
fn parallelize_turns_a_run_into_siblings_under_their_outside_descendant() {
    let (_temp_dir, _, repo) = setup_stack(&["first", "second", "outside"]);

    assert_eq!(
        parallelize(&repo, &["second", "first"]),
        MutationEffect::Changed
    );

    assert!(
        repo.op_log().expect("op log")[0]
            .description
            .contains("parallelize 2 commits")
    );
    let changes = changes(&repo);
    let base = change_by_description(&changes, "base");
    let first = change_by_description(&changes, "first");
    let second = change_by_description(&changes, "second");
    let outside = change_by_description(&changes, "outside");
    assert_eq!(first.parents, std::slice::from_ref(&base.commit_id.id));
    assert_eq!(second.parents, std::slice::from_ref(&base.commit_id.id));
    assert_eq!(
        repo.file_content(&second.commit_id.id, "first.txt")
            .expect("read first.txt from second"),
        "",
        "second no longer carries first's file"
    );
    let mut parents = outside.parents.clone();
    parents.sort();
    let mut expected = vec![first.commit_id.id.clone(), second.commit_id.id.clone()];
    expected.sort();
    assert_eq!(parents, expected);
    for name in ["first", "second", "outside"] {
        assert_eq!(
            repo.file_content(&outside.commit_id.id, &format!("{name}.txt"))
                .expect("read file from outside"),
            format!("{name}\n")
        );
    }
}

#[test]
fn parallelize_reports_nothing_to_do_when_no_parent_would_change() {
    let (_temp_dir, repo_path, _) = setup_stack(&["one", "two", "three"]);
    run_jj_in(&repo_path, &["new", "-m", "left", "subject(exact:two)"]);
    let repo = Repo::open(&repo_path).expect("open repo");
    let before = commit_ids(&repo);
    let operations = repo.op_log().expect("op log").len();

    assert_eq!(
        parallelize(&repo, &["three", "one"]),
        MutationEffect::Unchanged,
        "an unselected change sits between them"
    );
    assert_eq!(
        parallelize(&repo, &["left", "three"]),
        MutationEffect::Unchanged,
        "sibling heads"
    );

    assert_eq!(commit_ids(&repo), before);
    assert_eq!(repo.op_log().expect("op log").len(), operations);
}

#[test]
fn parallelize_keeps_the_working_copy_on_disk_after_rewriting_it() {
    let (_temp_dir, repo_path, repo) = setup_stack(&["first", "second", "wip"]);
    let working_copy = repo.log("@").expect("log @")[0].clone();

    assert_eq!(
        parallelize(&repo, &["wip", "second"]),
        MutationEffect::Changed
    );

    let changes = changes(&repo);
    let first = change_by_description(&changes, "first");
    let rewritten = repo.log("@").expect("log @")[0].clone();
    assert_ne!(rewritten.commit_id.id, working_copy.commit_id.id);
    assert_eq!(rewritten.change_id.id, working_copy.change_id.id);
    assert_eq!(rewritten.parents, std::slice::from_ref(&first.commit_id.id));
    assert!(repo_path.join("wip.txt").exists());
    assert!(
        !repo_path.join("second.txt").exists(),
        "the checkout follows the rewritten working copy"
    );

    fs::write(repo_path.join("after.txt"), "after\n").expect("write after");
    repo.refresh_working_copy()
        .expect("snapshot the follow-up edit");
    let snapshot = repo.log("@").expect("log @")[0].clone();
    assert_eq!(snapshot.change_id.id, working_copy.change_id.id);
    assert!(!snapshot.is_divergent);
    assert_eq!(
        repo.file_content("@", "after.txt").expect("read after.txt"),
        "after\n"
    );
}
