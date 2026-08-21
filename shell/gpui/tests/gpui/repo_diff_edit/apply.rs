use std::fs;

use gpui::TestAppContext;
use jayjay_core::{DiffEditDestination, Repo};
use jj_test::run_jj_in;

use super::fixtures::*;
use super::harness::*;

#[gpui::test]
fn remove_from_working_copy_exits_and_reselects_file(cx: &mut TestAppContext) {
    let (fixture, view, cx) = open_changed_repo(cx);
    view.update_in(cx, |view, _, cx| view.enter_diff_edit(cx));
    settle_visual(cx);
    select_first_changed_line(&view, cx);
    view.update_in(cx, |view, _, cx| {
        view.start_diff_edit_apply(DiffEditDestination::RemoveFromSource, cx)
    });
    settle_visual(cx);
    settle_visual(cx);
    assert!(!view.read_with(cx, |view, _| view.diff_edit_active()));
    assert_eq!(
        view.read_with(cx, |view, cx| {
            view.view_model()
                .read(cx)
                .selected_hunk()
                .map(|hunk| hunk.path.clone())
        }),
        Some("README.md".to_owned())
    );
    let repo = Repo::open(&fixture.path).unwrap();
    assert_ne!(
        repo.file_content("@", "README.md").unwrap(),
        "# Sample project\nfirst edit\nsecond edit\n"
    );
}

#[gpui::test]
fn done_keeps_only_selected_lines(cx: &mut TestAppContext) {
    let fixture = two_file_edits_fixture();
    let (view, cx) = open_fixture(&fixture, cx);
    select_file(&view, "edit.txt", cx);
    enter_and_select_group(&view, cx, "selected two");
    view.update_in(cx, |view, _, cx| {
        view.start_diff_edit_apply(DiffEditDestination::RemoveFromSource, cx)
    });
    settle_visual(cx);
    settle_visual(cx);

    assert_eq!(
        fs::read_to_string(fixture.path.join("edit.txt")).unwrap(),
        "one\nselected two\nthree\nfour\n"
    );
    assert_eq!(
        fs::read_to_string(fixture.path.join("untouched.txt")).unwrap(),
        "alpha\nbeta\ngamma\n"
    );
}

#[gpui::test]
fn done_waits_for_every_editable_file(cx: &mut TestAppContext) {
    let (fixture, view, cx) = open_changed_repo(cx);
    enter_and_select_line(&view, cx, "first edit");
    append_unloaded_file(&view, cx);
    view.update_in(cx, |view, _, cx| {
        view.start_diff_edit_apply(DiffEditDestination::RemoveFromSource, cx);
    });

    assert!(view.read_with(cx, |view, _| view.diff_edit_active()));
    assert_toast(
        &view,
        cx,
        "Wait for all editable files to finish loading before applying diff edit.",
    );
    assert_eq!(
        fs::read_to_string(fixture.path.join("README.md")).unwrap(),
        "# Sample project\nfirst edit\nsecond edit\n"
    );
}

#[gpui::test]
fn destinations_wait_for_select_all_to_finish(cx: &mut TestAppContext) {
    let fixture = separated_edits_fixture(true);
    let (view, cx) = open_fixture(&fixture, cx);
    select_change_by_description(&view, cx, "edit source");
    select_file(&view, "edit.txt", cx);
    view.update_in(cx, |view, _, cx| view.enter_diff_edit(cx));
    settle_visual(cx);
    append_unloaded_file(&view, cx);
    view.update_in(cx, |view, _, cx| {
        view.toggle_diff_edit_all(cx);
        assert!(view.diff_edit_selecting_all());
        assert!(view.diff_edit_selection_summary().0 > 0);
        view.start_diff_edit_apply(DiffEditDestination::NewChild, cx);
    });

    assert!(view.read_with(cx, |view, _| view.diff_edit_active()));
    assert_toast(
        &view,
        cx,
        "Wait for Select All to finish loading before applying diff edit.",
    );
}

#[gpui::test]
fn done_with_empty_selection_is_inert(cx: &mut TestAppContext) {
    let fixture = two_file_edits_fixture();
    let before_edit = fs::read_to_string(fixture.path.join("edit.txt")).unwrap();
    let before_untouched = fs::read_to_string(fixture.path.join("untouched.txt")).unwrap();
    let (view, cx) = open_fixture(&fixture, cx);
    view.update_in(cx, |view, _, cx| {
        view.enter_diff_edit(cx);
        view.start_diff_edit_apply(DiffEditDestination::RemoveFromSource, cx);
    });

    assert!(view.read_with(cx, |view, _| view.diff_edit_active()));
    assert_toast(
        &view,
        cx,
        "Select at least one file, hunk, or line before applying diff edit.",
    );
    assert_eq!(
        fs::read_to_string(fixture.path.join("edit.txt")).unwrap(),
        before_edit
    );
    assert_eq!(
        fs::read_to_string(fixture.path.join("untouched.txt")).unwrap(),
        before_untouched
    );
}

#[gpui::test]
fn new_child_contains_exactly_the_selected_lines(cx: &mut TestAppContext) {
    let fixture = separated_edits_fixture(false);
    let (view, cx) = open_fixture(&fixture, cx);
    select_file(&view, "edit.txt", cx);
    let source_change_id = selected_change_id(&view, cx);
    enter_and_select_group(&view, cx, "selected two");
    apply_with_message(&view, cx, DiffEditDestination::NewChild, "selected child");

    let repo = Repo::open(&fixture.path).expect("open mutated repo");
    let child = change_by_description(&repo, "selected child");
    let source = change_by_id(&repo, &source_change_id);
    assert_eq!(child.parents, vec![source.commit_id.id.clone()]);
    assert_eq!(
        repo.file_content(&child.change_id, "edit.txt")
            .expect("child file")
            .trim_end(),
        "one\nselected two\nthree\nfour\nfive\nsix\nseven\nremaining eight\nnine\nten"
    );
    assert_eq!(
        repo.file_content(&source.change_id, "edit.txt")
            .expect("source file")
            .trim_end(),
        "one\ntwo\nthree\nfour\nfive\nsix\nseven\nremaining eight\nnine\nten"
    );
    assert!(!view.read_with(cx, |view, _| view.diff_edit_active()));
}

#[gpui::test]
fn new_parallel_creates_a_sibling_with_same_parents(cx: &mut TestAppContext) {
    let fixture = separated_edits_fixture(false);
    let (view, cx) = open_fixture(&fixture, cx);
    select_file(&view, "edit.txt", cx);
    let source_change_id = selected_change_id(&view, cx);
    enter_and_select_group(&view, cx, "selected two");
    apply_with_message(
        &view,
        cx,
        DiffEditDestination::NewParallel,
        "selected parallel",
    );

    let repo = Repo::open(&fixture.path).expect("open mutated repo");
    let parallel = change_by_description(&repo, "selected parallel");
    let source = change_by_id(&repo, &source_change_id);
    assert_eq!(parallel.parents, source.parents);
    assert_ne!(parallel.parents, vec![source.commit_id.id.clone()]);
    assert_eq!(
        repo.file_content(&parallel.change_id, "edit.txt")
            .expect("parallel file")
            .trim_end(),
        "one\nselected two\nthree\nfour\nfive\nsix\nseven\neight\nnine\nten"
    );
    assert_eq!(
        repo.file_content(&source.change_id, "edit.txt")
            .expect("source file")
            .trim_end(),
        "one\ntwo\nthree\nfour\nfive\nsix\nseven\nremaining eight\nnine\nten"
    );
}

#[gpui::test]
fn move_to_working_copy_from_a_parent_change(cx: &mut TestAppContext) {
    let fixture = separated_edits_fixture(true);
    let (view, cx) = open_fixture(&fixture, cx);
    select_change_by_description(&view, cx, "edit source");
    select_file(&view, "edit.txt", cx);
    let source_change_id = selected_change_id(&view, cx);
    enter_and_select_group(&view, cx, "selected two");
    view.update_in(cx, |view, _, cx| {
        view.start_diff_edit_apply(DiffEditDestination::MoveToWorkingCopy, cx)
    });
    settle_visual(cx);

    let repo = Repo::open(&fixture.path).expect("open mutated repo");
    let source = change_by_id(&repo, &source_change_id);
    assert_eq!(
        repo.file_content(&source.change_id, "edit.txt")
            .expect("source file")
            .trim_end(),
        "one\ntwo\nthree\nfour\nfive\nsix\nseven\nremaining eight\nnine\nten"
    );
    assert_eq!(
        repo.file_content("@", "edit.txt")
            .expect("working-copy file")
            .trim_end(),
        "one\nselected two\nthree\nfour\nfive\nsix\nseven\nremaining eight\nnine\nten"
    );
    assert!(!view.read_with(cx, |view, _| view.diff_edit_active()));
}

#[gpui::test]
fn stale_selection_rejection_surfaces_and_refreshes(cx: &mut TestAppContext) {
    let (fixture, view, cx) = open_changed_repo(cx);
    enter_and_select_line(&view, cx, "first edit");
    let changed = "# Sample project\nfirst edit\nsecond edit\nintervening edit\n";
    fs::write(fixture.path.join("README.md"), changed).expect("edit after mode entry");
    run_jj_in(&fixture.path, &["st"]);

    view.update_in(cx, |view, _, cx| {
        view.start_diff_edit_apply(DiffEditDestination::RemoveFromSource, cx)
    });
    settle_visual(cx);

    assert!(!view.read_with(cx, |view, _| view.diff_edit_active()));
    assert_eq!(
        fs::read_to_string(fixture.path.join("README.md")).expect("read current file"),
        changed,
        "the rejected operation must not reconstruct the file from stale content"
    );
}
