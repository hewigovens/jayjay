use std::fs;

use jayjay_core::{DiffEditDestination, Repo};
use jj_test::{
    init_jj_repo, selection_for_lines, setup_source_change_with_child, whole_file_selection,
};

#[test]
fn diffedit_remove_from_source_updates_working_copy() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    fs::write(
        repo_path.join("notes.md"),
        "# keep this file\n\nremove this whole file from source\n",
    )
    .expect("write new file");
    repo.refresh_working_copy()
        .expect("snapshot working copy changes");

    let selection = whole_file_selection(&repo, "@", "notes.md");
    repo.apply_diff_selection(
        "@",
        DiffEditDestination::RemoveFromSource,
        &[selection],
        "",
        false,
    )
    .expect("remove selected line from working copy");

    let current = repo.show("@").expect("show updated working copy");
    assert!(
        current.diff.iter().all(|hunk| hunk.path != "notes.md"),
        "notes.md should be removed from the working copy diff"
    );
    let hello = current
        .diff
        .iter()
        .find(|hunk| hunk.path == "hello.txt")
        .expect("hello.txt initial diff remains");
    assert_eq!(hello.new_content.as_deref(), Some("hello from jayjay\n"));

    assert!(
        !repo_path.join("notes.md").exists(),
        "notes.md should be removed from disk after updating the working copy"
    );

    repo.refresh_working_copy()
        .expect("refresh updated working copy");
    let refreshed = repo.show("@").expect("show refreshed working copy");
    assert!(
        refreshed.diff.iter().all(|hunk| hunk.path != "notes.md"),
        "refresh should not reintroduce notes.md after removing it from @"
    );
}
#[test]
fn diffedit_remove_selected_lines_updates_working_copy_on_disk() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    fs::write(
        repo_path.join("notes.md"),
        "first line\nremove me\nlast line\n",
    )
    .expect("write new file");
    repo.refresh_working_copy()
        .expect("snapshot working copy changes");

    let selection = selection_for_lines(&repo, "@", "notes.md", &[(2, 2)]);
    repo.apply_diff_selection(
        "@",
        DiffEditDestination::RemoveFromSource,
        &[selection],
        "",
        false,
    )
    .expect("remove selected lines from working copy");

    let expected = "first line\nlast line\n";
    let current = repo.show("@").expect("show updated working copy");
    let notes = current
        .diff
        .iter()
        .find(|hunk| hunk.path == "notes.md")
        .expect("notes.md remains in working copy");
    assert_eq!(notes.new_content.as_deref(), Some(expected));
    assert_eq!(
        fs::read_to_string(repo_path.join("notes.md")).expect("read updated working copy file"),
        expected
    );

    repo.refresh_working_copy()
        .expect("refresh updated working copy");
    let refreshed = repo.show("@").expect("show refreshed working copy");
    let refreshed_notes = refreshed
        .diff
        .iter()
        .find(|hunk| hunk.path == "notes.md")
        .expect("notes.md remains after refresh");
    assert_eq!(refreshed_notes.new_content.as_deref(), Some(expected));
}
#[test]
fn diffedit_move_to_working_copy_moves_selected_file() {
    let (_temp_dir, _repo_path, repo) = setup_source_change_with_child();
    let selection = whole_file_selection(&repo, "@-", "notes.md");

    repo.apply_diff_selection(
        "@-",
        DiffEditDestination::MoveToWorkingCopy,
        &[selection],
        "",
        false,
    )
    .expect("move selected file to working copy");

    let source = repo.show("@-").expect("show rewritten source");
    assert!(
        source.diff.iter().all(|hunk| hunk.path != "notes.md"),
        "source change should no longer contain notes.md"
    );

    let current = repo.show("@").expect("show updated working copy");
    let notes = current
        .diff
        .iter()
        .find(|hunk| hunk.path == "notes.md")
        .expect("notes.md moved to working copy");
    assert_eq!(
        notes.new_content.as_deref(),
        Some("# moved content\n\nline for diffedit\n")
    );
}
#[test]
fn diffedit_new_child_extracts_selected_file_between_source_and_working_copy() {
    let (_temp_dir, _repo_path, repo) = setup_source_change_with_child();
    let selection = whole_file_selection(&repo, "@-", "notes.md");

    repo.apply_diff_selection(
        "@-",
        DiffEditDestination::NewChild,
        &[selection],
        "selected child",
        false,
    )
    .expect("extract selected file as child");

    let all = repo.log("all()").expect("read all changes");
    let child = all
        .iter()
        .find(|change| change.description == "selected child")
        .expect("selected child visible");
    let source = all
        .iter()
        .filter(|change| change.description == "source change")
        .find(|change| {
            repo.show(&change.commit_id)
                .expect("show candidate source")
                .diff
                .iter()
                .all(|hunk| hunk.path != "notes.md")
        })
        .expect("rewritten source change still visible");
    assert_eq!(child.parents, vec![source.commit_id.clone()]);

    let source_detail = repo.show(&source.commit_id).expect("show rewritten source");
    assert!(
        source_detail
            .diff
            .iter()
            .all(|hunk| hunk.path != "notes.md"),
        "rewritten source should no longer contain notes.md"
    );

    let child_detail = repo.show(&child.commit_id).expect("show selected child");
    let notes = child_detail
        .diff
        .iter()
        .find(|hunk| hunk.path == "notes.md")
        .expect("notes.md extracted to child");
    assert_eq!(
        notes.new_content.as_deref(),
        Some("# moved content\n\nline for diffedit\n")
    );
}
#[test]
fn diffedit_new_parallel_extracts_selected_file_as_sibling() {
    let (_temp_dir, _repo_path, repo) = setup_source_change_with_child();
    let selection = whole_file_selection(&repo, "@-", "notes.md");

    repo.apply_diff_selection(
        "@-",
        DiffEditDestination::NewParallel,
        &[selection],
        "selected parallel",
        false,
    )
    .expect("extract selected file as parallel");

    let all = repo.log("all()").expect("read all changes");
    let parallel = all
        .iter()
        .find(|change| change.description == "selected parallel")
        .expect("selected parallel visible");
    let source = all
        .iter()
        .filter(|change| change.description == "source change")
        .find(|change| {
            repo.show(&change.commit_id)
                .expect("show candidate source")
                .diff
                .iter()
                .all(|hunk| hunk.path != "notes.md")
        })
        .expect("rewritten source change still visible");

    assert_eq!(parallel.parents, source.parents);

    let source_detail = repo.show(&source.commit_id).expect("show rewritten source");
    assert!(
        source_detail
            .diff
            .iter()
            .all(|hunk| hunk.path != "notes.md"),
        "rewritten source should no longer contain notes.md"
    );

    let parallel_detail = repo
        .show(&parallel.commit_id)
        .expect("show selected parallel");
    let notes = parallel_detail
        .diff
        .iter()
        .find(|hunk| hunk.path == "notes.md")
        .expect("notes.md extracted to parallel change");
    assert_eq!(
        notes.new_content.as_deref(),
        Some("# moved content\n\nline for diffedit\n")
    );
}
