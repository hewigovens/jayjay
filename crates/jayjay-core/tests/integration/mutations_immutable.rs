use std::fs;

use jayjay_core::{InsertPosition, RebaseMode, Repo};
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
        (
            "abandon selected",
            Box::new(|| repo.abandon_many(&[rev.to_owned(), "@".to_owned()])),
        ),
        (
            "rebase",
            Box::new(|| repo.rebase(rev, "@", RebaseMode::Source).map(drop)),
        ),
        (
            "rebase selected",
            Box::new(|| repo.rebase_many(&[rev.to_owned(), "@".to_owned()], "root()")),
        ),
        ("squash", Box::new(|| repo.squash(rev, Some("@")))),
        (
            "squash selected",
            Box::new(|| {
                repo.squash_many(&["@".to_owned(), rev.to_owned()])
                    .map(drop)
            }),
        ),
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

    repo.rebase("@", "main", RebaseMode::Source)
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
            .rebase("base", dest, RebaseMode::Source)
            .expect_err("rebasing onto itself or a descendant must be refused");
        assert!(error.to_string().contains("descendants"), "{error}");
    }

    assert_eq!(repo.op_log().expect("op log").len(), before.len());
}

fn protect_bookmark(repo_path: &std::path::Path, name: &str) {
    run_jj_in(
        repo_path,
        &[
            "config",
            "set",
            "--repo",
            "revset-aliases.'immutable_heads()'",
            &format!("bookmarks(\"{name}\")"),
        ],
    );
}

#[test]
fn tracking_a_protected_bookmark_onto_the_working_copy_starts_a_new_change() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");
    let protected = repo.log("@").expect("log")[0].clone();
    run_git(
        &repo_path,
        &[
            "remote",
            "add",
            "origin",
            "https://example.invalid/origin.git",
        ],
    );
    run_git(
        &repo_path,
        &[
            "update-ref",
            "refs/remotes/origin/freeze",
            &protected.commit_id.id,
        ],
    );
    protect_bookmark(&repo_path, "freeze");
    run_jj_in(&repo_path, &["status"]);
    let repo = Repo::open(&repo_path).expect("reopen repo");

    repo.track_bookmark("freeze", "origin").expect("track");

    let head = repo.log("@").expect("log")[0].clone();
    assert!(
        head.is_empty && !head.is_immutable,
        "@ must move to a fresh mutable change"
    );
    assert_eq!(head.parents, std::slice::from_ref(&protected.commit_id.id));
    fs::write(repo_path.join("hello.txt"), "edited after freeze\n").expect("edit");
    repo.refresh_working_copy().expect("snapshot");
    let frozen = repo.log(&protected.change_id.id).expect("log frozen");
    assert_eq!(frozen.len(), 1);
    assert_eq!(
        frozen[0].commit_id.id, protected.commit_id.id,
        "the protected commit must not be rewritten"
    );
    assert!(frozen[0].is_immutable);
    assert!(
        !repo.log("@").expect("log")[0].is_empty,
        "the edit lands in the new change"
    );
}

#[test]
fn snapshot_never_rewrites_an_immutable_working_copy() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    run_jj_in(&repo_path, &["bookmark", "create", "freeze", "-r", "@"]);
    protect_bookmark(&repo_path, "freeze");
    let repo = Repo::open(&repo_path).expect("open repo");
    let protected = repo.log("@").expect("log")[0].clone();
    assert!(
        protected.is_immutable,
        "fixture working copy must be immutable"
    );
    fs::write(repo_path.join("hello.txt"), "edited while frozen\n").expect("edit");

    repo.refresh_working_copy().expect("snapshot");

    let head = repo.log("@").expect("log")[0].clone();
    assert_ne!(head.change_id.id, protected.change_id.id);
    assert_eq!(head.parents, std::slice::from_ref(&protected.commit_id.id));
    assert!(!head.is_empty && !head.is_immutable);
    assert_eq!(
        repo.log(&protected.change_id.id).expect("log frozen")[0]
            .commit_id
            .id,
        protected.commit_id.id,
        "the protected commit must not be rewritten"
    );
    assert_eq!(
        run_git(&repo_path, &["rev-parse", "HEAD"]).stdout,
        format!("{}\n", protected.commit_id.id).into_bytes(),
        "the colocated HEAD follows the new working copy's parent"
    );
}
