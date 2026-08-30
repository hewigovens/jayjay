use std::fs;

use jayjay_core::{InsertPosition, Repo};
use jj_test::{init_jj_repo, run_git, run_jj_in};

/// Defense in depth behind the shells' menu gating: these mutations rewrite through jj-lib directly, so core must refuse immutable targets itself.
#[test]
fn mutations_refuse_to_rewrite_an_immutable_commit() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");

    fs::write(repo_path.join("a.txt"), "protected\n").expect("write a.txt");
    run_jj_in(&repo_path, &["describe", "-m", "protected"]);
    run_jj_in(&repo_path, &["new", "-m", "child"]);
    run_git(&repo_path, &["tag", "release"]);
    run_jj_in(&repo_path, &["st"]);

    let repo = Repo::open(&repo_path).expect("open repo");
    let target = repo
        .log("all()")
        .expect("log all")
        .into_iter()
        .find(|c| c.description.trim() == "protected")
        .expect("protected change present");
    assert!(target.is_immutable, "fixture change must be immutable");
    let rev = target.change_id.id.as_str();

    type Attempt<'a> = Box<dyn Fn() -> jayjay_core::CoreResult<()> + 'a>;
    let attempts: Vec<(&str, Attempt)> = vec![
        ("describe", Box::new(|| repo.describe(rev, "rewritten"))),
        ("edit", Box::new(|| repo.edit(rev))),
        ("abandon", Box::new(|| repo.abandon(rev))),
        ("rebase", Box::new(|| repo.rebase(rev, "@").map(drop))),
        ("squash", Box::new(|| repo.squash(rev, Some("@")))),
        ("squash into", Box::new(|| repo.squash("@", Some(rev)))),
        (
            "new before",
            Box::new(|| repo.new_change_inserted(rev, InsertPosition::Before, "")),
        ),
        (
            "new after",
            Box::new(|| repo.new_change_inserted("root()", InsertPosition::After, "")),
        ),
    ];
    for (name, attempt) in attempts {
        let err = attempt().expect_err(&format!("{name} on an immutable change must fail"));
        assert!(
            err.to_string().contains("immutable"),
            "unclear error for {name}: {err}"
        );
    }

    let unchanged = repo
        .log("all()")
        .expect("log after refusals")
        .into_iter()
        .find(|c| c.description.trim() == "protected")
        .expect("protected change still present");
    assert_eq!(unchanged.commit_id.id, target.commit_id.id);
}

#[test]
fn rebase_onto_the_current_parent_records_nothing() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    run_jj_in(&repo_path, &["bookmark", "create", "main", "-r", "@"]);
    run_jj_in(&repo_path, &["new", "-m", "child"]);
    let repo = Repo::open(&repo_path).expect("open repo");
    let before = repo.op_log().expect("op log");
    let commit_before = repo.show_summary("@").expect("show").info.commit_id.id;

    repo.rebase("@", "main")
        .expect("rebase onto the existing parent");

    assert_eq!(repo.op_log().expect("op log").len(), before.len());
    assert_eq!(
        repo.show_summary("@").expect("show").info.commit_id.id,
        commit_before
    );
}

#[test]
fn rebase_onto_a_descendant_is_refused_without_recording_anything() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    run_jj_in(&repo_path, &["bookmark", "create", "base", "-r", "@"]);
    run_jj_in(&repo_path, &["new", "-m", "child"]);
    run_jj_in(&repo_path, &["new", "-m", "grandchild"]);
    let repo = Repo::open(&repo_path).expect("open repo");
    let before = repo.op_log().expect("op log");

    for dest in ["base", "@"] {
        let error = repo
            .rebase("base", dest)
            .expect_err("rebasing onto itself or a descendant must be refused");
        assert!(error.to_string().contains("descendants"), "{error}");
    }

    assert_eq!(repo.op_log().expect("op log").len(), before.len());
}
