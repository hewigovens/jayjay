use std::fs;

use jayjay_core::Repo;
use jj_test::init_jj_repo;

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

    let x = repo
        .log("description(\"target X\")")
        .expect("log X")
        .into_iter()
        .find(|change| change.description.trim() == "target X")
        .expect("find X");

    repo.restore_files(&x.change_id, &["a.txt".to_owned()])
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
        b_hunk.new_content.as_deref(),
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
