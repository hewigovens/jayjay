use std::fs;

use jayjay_core::diff;
use jayjay_core::{DiffHunk, Repo};
use jayjay_review::{NoteAnchor, NoteSide, NoteStatus, ReviewStore};
use jj_test::{init_jj_repo, run_jj_in};

/// Anchor for a hunk's first change group, derived exactly like the GUI: the Histogram file diff, conflict display lines, then canonical change groups.
fn gui_anchor(change_id: &str, hunk: &DiffHunk) -> NoteAnchor {
    let file_diff = diff::compute_file_diff(
        &hunk.path,
        hunk.old_content.as_deref().unwrap_or(""),
        hunk.new_content.as_deref().unwrap_or(""),
        false,
    );
    let lines = diff::build_diff_display_lines(&file_diff.lines);
    let group = diff::change_groups(&lines)
        .into_iter()
        .next()
        .expect("change group");
    NoteAnchor {
        change_id: change_id.to_owned(),
        path: hunk.path.clone(),
        identity: hunk.review_identity.clone(),
        side: match group.anchor_side {
            diff::DiffSide::Old => NoteSide::Old,
            diff::DiffSide::New => NoteSide::New,
        },
        line: group.anchor_line,
        anchor_excerpt: group.anchor_excerpt,
        anchor_context: group.anchor_context,
        ignore_whitespace: false,
    }
}

fn setup_note() -> (tempfile::TempDir, Repo, ReviewStore, String) {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    fs::write(
        repo_path.join("hello.txt"),
        "hello from jayjay\nplease check this\n",
    )
    .expect("edit file");
    let repo = Repo::open(&repo_path).expect("open repo");
    repo.refresh_working_copy().expect("snapshot");
    let detail = repo.show("@").expect("show working copy");
    let change_id = detail.info.change_id.id;
    let hunk = detail
        .diff
        .into_iter()
        .find(|hunk| hunk.path == "hello.txt")
        .expect("hello hunk");
    let mut store = ReviewStore::in_memory();
    let note = store.add_note(gui_anchor(&change_id, &hunk), "Please check this edge case");
    (temp_dir, repo, store, note.id)
}

fn reconcile(repo: &Repo, store: &ReviewStore, include_resolved: bool) -> Vec<NoteStatus> {
    repo.review_notes_report(store, "@", include_resolved)
        .expect("reconcile")
        .notes
        .into_iter()
        .map(|status| status.status)
        .collect()
}

#[test]
fn reconcile_current_note_maps_to_group() {
    let (_temp_dir, repo, store, _note_id) = setup_note();

    let report = repo
        .review_notes_report(&store, "@", false)
        .expect("reconcile");

    assert_eq!(report.notes[0].status, NoteStatus::Current);
    assert_eq!(report.notes[0].group_index, Some(0));
}

#[test]
fn reconcile_content_edit_marks_note_stale() {
    let (temp_dir, repo, store, _note_id) = setup_note();
    fs::write(
        temp_dir.path().join("repo").join("hello.txt"),
        "rewritten\n",
    )
    .expect("rewrite");
    repo.refresh_working_copy().expect("snapshot edit");

    assert_eq!(reconcile(&repo, &store, false), vec![NoteStatus::Stale]);
}

#[test]
fn reconcile_removed_diff_marks_note_orphaned() {
    let (temp_dir, repo, store, _note_id) = setup_note();
    fs::remove_file(temp_dir.path().join("repo").join("hello.txt")).expect("remove file");
    repo.refresh_working_copy().expect("snapshot restore");

    assert_eq!(reconcile(&repo, &store, false), vec![NoteStatus::Orphaned]);
}

#[test]
fn reconcile_resolved_note_is_resolved_even_if_anchor_changed() {
    let (temp_dir, repo, mut store, note_id) = setup_note();
    store.resolve_note(&note_id).expect("resolve note");
    fs::write(
        temp_dir.path().join("repo").join("hello.txt"),
        "rewritten\n",
    )
    .expect("rewrite");
    repo.refresh_working_copy().expect("snapshot edit");

    assert_eq!(reconcile(&repo, &store, true), vec![NoteStatus::Resolved]);
}

#[test]
fn reconcile_note_on_renamed_file_with_edit_stays_current() {
    // Regression: a CLI reconciling through its own duplicated diff stack reported notes on renamed files as stale while the app showed them current; anchor exactly like the GUI (identity from show_summary, content from show_file_rename) and reconcile through the same provider.
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    fs::create_dir(repo_path.join("src")).expect("mkdir src");
    fs::write(repo_path.join("src/x.txt"), "one\ntwo\nthree\nfour\n").expect("seed file");
    run_jj_in(&repo_path, &["commit", "-m", "seed"]);
    fs::create_dir(repo_path.join("lib")).expect("mkdir lib");
    fs::write(repo_path.join("lib/x.txt"), "one\ntwo\nthree\nfour\nfive\n")
        .expect("write moved file");
    fs::remove_file(repo_path.join("src/x.txt")).expect("remove old path");

    let repo = Repo::open(&repo_path).expect("open repo");
    repo.refresh_working_copy().expect("snapshot");
    let detail = repo.show_summary("@").expect("summary");
    let change_id = detail.info.change_id.id;
    let summary_hunk = detail
        .diff
        .into_iter()
        .find(|hunk| hunk.path == "lib/x.txt")
        .expect("renamed hunk");
    assert_eq!(summary_hunk.old_path.as_deref(), Some("src/x.txt"));

    let mut content_hunk = repo
        .show_file_rename("@", "src/x.txt", "lib/x.txt")
        .expect("rename content");
    content_hunk.review_identity = summary_hunk.review_identity.clone();

    let mut store = ReviewStore::in_memory();
    store.add_note(
        gui_anchor(&change_id, &content_hunk),
        "check the added line",
    );

    let report = repo
        .review_notes_report(&store, "@", false)
        .expect("reconcile");
    assert_eq!(report.notes[0].status, NoteStatus::Current);
    assert_eq!(report.change_id, change_id);
}
