use std::fs;
use std::path::PathBuf;

use jayjay_core::{ChangeInfo, Repo};
use jj_test::{init_jj_repo, run_git, run_jj_in};
use tempfile::TempDir;

fn change_by_description(repo: &Repo, description: &str) -> ChangeInfo {
    repo.log(&format!("description(\"{description}\")"))
        .expect("log by description")
        .into_iter()
        .find(|change| change.description.trim() == description)
        .unwrap_or_else(|| panic!("change '{description}' present"))
}

/// Merge topology for from-parent restores: base holds a.txt = "a base"; parent one edits it to "a from p1"; parent two (also on base) adds b.txt only; @ is their merge with a.txt edited to "a from merge".
fn merge_fixture() -> (TempDir, PathBuf, Repo) {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    fs::write(repo_path.join("a.txt"), "a base\n").expect("write a base");
    repo.describe("@", "base").expect("describe base");

    repo.new_change("@", "parent one").expect("create p1");
    fs::write(repo_path.join("a.txt"), "a from p1\n").expect("edit a in p1");
    repo.refresh_working_copy().expect("snapshot p1");

    repo.new_change("description(\"base\")", "parent two")
        .expect("create p2");
    fs::write(repo_path.join("b.txt"), "b from p2\n").expect("write b in p2");
    repo.refresh_working_copy().expect("snapshot p2");

    repo.merge(&[
        "description(\"parent one\")".to_owned(),
        "description(\"parent two\")".to_owned(),
    ])
    .expect("create merge");
    repo.describe("@", "merge").expect("describe merge");
    fs::write(repo_path.join("a.txt"), "a from merge\n").expect("edit a in merge");
    repo.refresh_working_copy().expect("snapshot merge");

    (temp_dir, repo_path, repo)
}

/// Non-working-copy targets take the custom matcher + restore_tree + descendant-rebase
/// branch, where a matcher bug could clobber unrelated files; exercise it on a historical commit.
#[test]
fn restore_files_reverts_only_selected_path_in_historical_commit() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    // Parent P establishes a.txt and b.txt.
    fs::write(repo_path.join("a.txt"), "a base\n").expect("write a base");
    fs::write(repo_path.join("b.txt"), "b base\n").expect("write b base");
    repo.describe("@", "parent P").expect("describe parent");

    // Target X (not @) modifies both files.
    repo.new_change("@", "target X").expect("create X");
    fs::write(repo_path.join("a.txt"), "a from X\n").expect("write a in X");
    fs::write(repo_path.join("b.txt"), "b from X\n").expect("write b in X");
    repo.refresh_working_copy().expect("snapshot X");

    // Descendant @ adds its own edit on top of X.
    repo.new_change("@", "child C").expect("create child");
    fs::write(repo_path.join("c.txt"), "c from child\n").expect("write c in child");
    repo.refresh_working_copy().expect("snapshot child");

    let x = change_by_description(&repo, "target X");

    repo.restore_files(&x.change_id, None, &["a.txt".to_owned()])
        .expect("restore a.txt in historical commit X");

    // X's diff must drop a.txt (reverted) but keep b.txt's modification.
    let x_detail = repo.show(&x.change_id).expect("show X after restore");
    assert!(
        x_detail.diff.iter().all(|hunk| hunk.path != "a.txt"),
        "a.txt should be reverted out of X's diff: {:?}",
        x_detail.diff.iter().map(|h| &h.path).collect::<Vec<_>>()
    );
    let b_hunk = x_detail
        .diff
        .iter()
        .find(|hunk| hunk.path == "b.txt")
        .expect("b.txt modification must remain in X");
    assert_eq!(
        b_hunk.new.content.as_deref(),
        Some("b from X\n"),
        "unrelated file b.txt must keep X's edit"
    );

    // The reverted file now matches the parent's content inside X.
    assert_eq!(
        repo.file_content(&x.change_id, "a.txt")
            .expect("read a.txt from X")
            .trim_end(),
        "a base",
        "a.txt in X should hold the parent's content after restore"
    );

    // Descendant C was rebased on the rewritten X and kept its own edit.
    let child = repo.show("@").expect("show rebased child");
    assert_eq!(child.info.description.trim(), "child C");
    assert!(
        child.diff.iter().any(|hunk| hunk.path == "c.txt"),
        "child's own edit must survive the descendant rebase"
    );
    assert_eq!(
        repo.file_content("@", "a.txt")
            .expect("read a.txt from child")
            .trim_end(),
        "a base",
        "the restore must propagate to the rebased descendant"
    );
    assert_eq!(
        repo.file_content("@", "b.txt")
            .expect("read b.txt from child")
            .trim_end(),
        "b from X",
        "unrelated file must keep X's edit in the descendant too"
    );
}

/// "Restore to Parent N" on a merge must treat the chosen parent as the content SOURCE and rewrite only the merge; treating the parent as the target rev rewrote the parent in place instead.
#[test]
fn restore_files_from_a_parent_rewrites_the_merge_not_the_parent() {
    let (_tmp, _repo_path, repo) = merge_fixture();
    // A child on top makes the merge take the non-working-copy rewrite branch.
    repo.new_change("@", "child").expect("create child");

    let merge = change_by_description(&repo, "merge");
    assert_eq!(merge.parents.len(), 2, "fixture merge has two parents");
    let (p1_commit, p2_commit) = (merge.parents[0].clone(), merge.parents[1].clone());

    // Parent two left a.txt at the base content, so the result is distinguishable from parent one AND from the auto-merged parent tree (both hold "a from p1").
    repo.restore_files(&merge.change_id, Some(&p2_commit), &["a.txt".to_owned()])
        .expect("restore a.txt in merge from parent two");

    assert_eq!(
        repo.file_content(&merge.change_id, "a.txt")
            .expect("read a.txt from merge")
            .trim_end(),
        "a base",
        "the merge's file must hold parent two's content"
    );
    assert_eq!(
        change_by_description(&repo, "parent one").commit_id.id,
        p1_commit,
        "parent one must not be rewritten"
    );
    assert_eq!(
        change_by_description(&repo, "parent two").commit_id.id,
        p2_commit,
        "parent two must not be rewritten"
    );
    assert_eq!(
        repo.file_content(&p1_commit, "a.txt")
            .expect("read a.txt from parent one")
            .trim_end(),
        "a from p1",
        "parent one's content must be untouched"
    );
    assert_eq!(
        repo.file_content("@", "a.txt")
            .expect("read a.txt from child")
            .trim_end(),
        "a base",
        "the restore must propagate to the rebased child"
    );
}

/// Octopus adjacency: with three parents each holding a DISTINCT a.txt, the result can only match the parent actually passed as `from`, so a parent-index or source/target mixup cannot pass.
#[test]
fn restore_files_from_the_third_parent_of_an_octopus_merge_uses_that_parent() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    fs::write(repo_path.join("a.txt"), "a base\n").expect("write a base");
    repo.describe("@", "base").expect("describe base");
    for n in 1..=3 {
        repo.new_change("description(\"base\")", &format!("parent {n}"))
            .expect("create parent");
        fs::write(repo_path.join("a.txt"), format!("a p{n}\n")).expect("edit a in parent");
        repo.refresh_working_copy().expect("snapshot parent");
    }

    repo.merge(&[
        "description(\"parent 1\")".to_owned(),
        "description(\"parent 2\")".to_owned(),
        "description(\"parent 3\")".to_owned(),
    ])
    .expect("create octopus merge");
    repo.describe("@", "merge").expect("describe merge");
    // Overwriting the conflicted file resolves the three-way conflict on snapshot.
    fs::write(repo_path.join("a.txt"), "a from merge\n").expect("resolve a in merge");
    repo.refresh_working_copy().expect("snapshot merge");
    repo.new_change("@", "child").expect("create child");

    let merge = change_by_description(&repo, "merge");
    assert_eq!(merge.parents.len(), 3, "fixture merge has three parents");
    let parents = merge.parents.clone();

    repo.restore_files(&merge.change_id, Some(&parents[2]), &["a.txt".to_owned()])
        .expect("restore a.txt in octopus merge from parent 3");

    assert_eq!(
        repo.file_content(&merge.change_id, "a.txt")
            .expect("read a.txt from merge")
            .trim_end(),
        "a p3",
        "the merge's file must hold exactly the third parent's content"
    );
    for (ix, parent) in parents.iter().enumerate() {
        assert_eq!(
            change_by_description(&repo, &format!("parent {}", ix + 1))
                .commit_id
                .id,
            *parent,
            "parent {} must not be rewritten",
            ix + 1
        );
    }
    assert_eq!(
        repo.file_content("@", "a.txt")
            .expect("read a.txt from child")
            .trim_end(),
        "a p3",
        "the restore must propagate to the rebased child"
    );
}

/// Defense in depth behind the shells' menu gating: the direct jj-lib rewrite branch gets no immutability enforcement from jj, so it must refuse an immutable target itself and leave the commit untouched.
#[test]
fn restore_files_refuses_to_rewrite_an_immutable_commit() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");

    fs::write(repo_path.join("a.txt"), "a protected\n").expect("write a.txt");
    run_jj_in(&repo_path, &["describe", "-m", "protected"]);
    run_jj_in(&repo_path, &["new", "-m", "child"]);
    // Colocated git HEAD tracks @-, so this tags "protected"; tags() are inside the built-in immutable_heads(), and `jj st` imports the new ref.
    run_git(&repo_path, &["tag", "release"]);
    run_jj_in(&repo_path, &["st"]);

    let repo = Repo::open(&repo_path).expect("open repo");
    // The jj CLI's describe appends a newline, so match on the trimmed description instead of a description() revset (which matches exactly).
    let target = repo
        .log("all()")
        .expect("log all")
        .into_iter()
        .find(|c| c.description.trim() == "protected")
        .expect("protected change present");
    assert!(target.is_immutable, "fixture change must be immutable");

    let err = repo
        .restore_files(&target.change_id, None, &["a.txt".to_owned()])
        .expect_err("restore on an immutable change must fail");
    assert!(
        err.to_string().contains("immutable"),
        "unclear error: {err}"
    );

    let after = repo
        .log(&target.change_id)
        .expect("log target after failed restore")
        .into_iter()
        .next()
        .expect("target still present");
    assert_eq!(
        after.commit_id.id, target.commit_id.id,
        "the immutable commit must not be rewritten"
    );
    assert_eq!(
        repo.file_content(&target.change_id, "a.txt")
            .expect("read a.txt")
            .trim_end(),
        "a protected",
        "the immutable commit's content must be untouched"
    );
}

/// A merge that IS the working copy takes the `jj restore --from <parent>` fast path: the disk file reverts to the chosen parent's content and neither parent is rewritten.
#[test]
fn restore_files_from_a_parent_on_a_working_copy_merge_updates_the_disk_file() {
    let (_tmp, repo_path, repo) = merge_fixture();

    let merge = change_by_description(&repo, "merge");
    let (p1_commit, p2_commit) = (merge.parents[0].clone(), merge.parents[1].clone());

    repo.restore_files("@", Some(&p2_commit), &["a.txt".to_owned()])
        .expect("restore a.txt in working-copy merge from parent two");

    assert_eq!(
        fs::read_to_string(repo_path.join("a.txt")).expect("read a.txt from disk"),
        "a base\n",
        "the working-copy file must materialize parent two's content"
    );
    assert_eq!(
        change_by_description(&repo, "parent one").commit_id.id,
        p1_commit,
        "parent one must not be rewritten"
    );
    assert_eq!(
        change_by_description(&repo, "parent two").commit_id.id,
        p2_commit,
        "parent two must not be rewritten"
    );
}
