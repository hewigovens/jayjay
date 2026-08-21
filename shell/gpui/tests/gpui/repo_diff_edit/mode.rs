use gpui::{Modifiers, TestAppContext};
use jayjay_core::DiffEditDestination;
use jayjay_gpui::diff::DiffViewMode;
use jj_test::{run_git, run_jj_in};

use super::fixtures::*;
use super::harness::*;

#[gpui::test]
fn button_cancel_and_escape_preserve_view_mode(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_changed_repo(cx);
    view.update_in(cx, |view, _, cx| {
        view.view_model()
            .update(cx, |vm, _| vm.view_mode = DiffViewMode::SideBySide)
    });

    let edit = cx.debug_bounds("edit-diff").expect("Edit Diff button");
    cx.simulate_click(edit.center(), Modifiers::default());
    settle_visual(cx);
    assert!(view.read_with(cx, |view, _| view.diff_edit_active()));
    assert_eq!(
        view.read_with(cx, |view, cx| view.view_model().read(cx).view_mode),
        DiffViewMode::SideBySide
    );

    let cancel = cx.debug_bounds("diff-edit-cancel").expect("Cancel button");
    cx.simulate_click(cancel.center(), Modifiers::default());
    assert!(!view.read_with(cx, |view, _| view.diff_edit_active()));
    assert_eq!(
        view.read_with(cx, |view, cx| view.view_model().read(cx).view_mode),
        DiffViewMode::SideBySide
    );

    view.update_in(cx, |view, _, cx| view.enter_diff_edit(cx));
    cx.simulate_keystrokes("escape");
    assert!(!view.read_with(cx, |view, _| view.diff_edit_active()));
}

#[gpui::test]
fn gutter_menu_enters_mode(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_changed_repo(cx);
    let action = view.update_in(cx, |view, _, cx| {
        let hunk = view.view_model().read(cx).selected_hunk().cloned().unwrap();
        view.build_diff_gutter_menu(&hunk, 0, cx)
            .into_iter()
            .find(|item| item.label == "Open Diff Edit Mode")
            .expect("menu item")
            .action
    });
    view.update_in(cx, |view, _, cx| view.dispatch_context_action(action, cx));
    assert!(view.read_with(cx, |view, _| view.diff_edit_active()));
}

#[gpui::test]
fn compare_mode_has_no_diff_edit_entry_and_cannot_enter(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_changed_repo(cx);
    let other_ix = view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        let selected = vm.selected.expect("selected change");
        (0..vm.graph.changes.len())
            .find(|ix| *ix != selected)
            .expect("another change")
    });
    view.update_in(cx, |view, _, cx| {
        view.select_or_compare_change(other_ix, true, cx);
    });
    settle_visual(cx);

    assert!(view.read_with(cx, |view, cx| view.view_model().read(cx).compare.is_some()));
    assert!(cx.debug_bounds("edit-diff").is_none());
    view.update_in(cx, |view, _, cx| {
        let hunk = view
            .view_model()
            .read(cx)
            .selected_hunk()
            .cloned()
            .expect("compare hunk");
        assert!(
            view.build_diff_gutter_menu(&hunk, 0, cx)
                .iter()
                .all(|item| item.label != "Open Diff Edit Mode")
        );
        view.enter_diff_edit(cx);
    });
    assert!(!view.read_with(cx, |view, _| view.diff_edit_active()));
}

#[gpui::test]
fn switching_changes_clears_diff_edit_session(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_changed_repo(cx);
    view.update_in(cx, |view, _, cx| {
        view.enter_diff_edit(cx);
        let current = view.view_model().read(cx).selected.unwrap();
        let next = if current == 0 { 1 } else { 0 };
        view.select_change(next, cx);
    });
    settle_visual(cx);
    assert!(!view.read_with(cx, |view, _| view.diff_edit_active()));
}

#[gpui::test]
fn non_working_copy_shows_destinations_and_prefills_description(cx: &mut TestAppContext) {
    let fixture = separated_edits_fixture(true);
    let (view, cx) = open_fixture(&fixture, cx);
    select_change_by_description(&view, cx, "edit source");
    view.read_with(cx, |view, cx| {
        let change = view
            .view_model()
            .read(cx)
            .selected_change()
            .expect("edit source selected");
        assert!(!change.is_working_copy);
        assert!(!change.is_immutable);
    });
    let edit = cx
        .debug_bounds("edit-diff")
        .expect("Edit Diff button for mutable parent change");
    cx.simulate_click(edit.center(), Modifiers::default());
    settle_visual(cx);
    view.read_with(cx, |view, _| {
        let snapshot = view.diff_edit_snapshot();
        assert!(!snapshot.working_copy);
        assert_eq!(snapshot.description.trim(), "edit source");
        assert_eq!(
            snapshot.destinations,
            vec![
                DiffEditDestination::NewChild,
                DiffEditDestination::NewParallel,
                DiffEditDestination::MoveToWorkingCopy,
                DiffEditDestination::RemoveFromSource,
            ]
        );
    });
    assert!(
        cx.debug_bounds("diff-edit-message").is_none(),
        "the single-line description input must not render"
    );
    let edit_description = cx
        .debug_bounds("diff-edit-description")
        .expect("Diff Edit description button");
    cx.simulate_click(edit_description.center(), Modifiers::default());
    let input = view
        .read_with(cx, |view, _| view.text_modal_input())
        .expect("shared description modal input");
    let modal_text = view.read_with(cx, |_, cx| input.read(cx).text());
    assert_eq!(modal_text.trim_end(), "edit source");
    input.update(cx, |input, cx| {
        input.set_text("selected work\n\nkeep these lines together", cx)
    });
    view.update_in(cx, |view, _, cx| view.submit_text_modal(cx));
    assert!(!view.read_with(cx, |view, _| view.has_text_modal()));
    let saved = view.read_with(cx, |view, _| view.diff_edit_snapshot().description);
    assert_eq!(
        saved.trim_end(),
        "selected work\n\nkeep these lines together"
    );

    view.update_in(cx, |view, _, cx| view.exit_diff_edit(cx));
    select_change_by_description(&view, cx, "working child");
    view.update_in(cx, |view, _, cx| view.enter_diff_edit(cx));
    settle_visual(cx);
    view.read_with(cx, |view, _| {
        let snapshot = view.diff_edit_snapshot();
        assert!(snapshot.working_copy);
        assert_eq!(
            snapshot.destinations,
            vec![DiffEditDestination::RemoveFromSource]
        );
    });
    assert!(cx.debug_bounds("diff-edit-description").is_none());
}

#[gpui::test]
fn immutable_change_offers_no_diff_edit_entry(cx: &mut TestAppContext) {
    let fixture = separated_edits_fixture(true);
    run_git(&fixture.path, &["tag", "release"]);
    run_jj_in(&fixture.path, &["st"]);
    let (view, cx) = open_fixture(&fixture, cx);
    select_change_by_description(&view, cx, "edit source");
    view.read_with(cx, |view, cx| {
        let change = view
            .view_model()
            .read(cx)
            .selected_change()
            .expect("edit source selected");
        assert!(
            change.is_immutable,
            "tagged fixture change must be immutable"
        );
    });

    assert!(cx.debug_bounds("edit-diff").is_none());
    view.update_in(cx, |view, _, cx| {
        let hunk = view
            .view_model()
            .read(cx)
            .selected_hunk()
            .cloned()
            .expect("immutable hunk");
        assert!(
            view.build_diff_gutter_menu(&hunk, 0, cx)
                .iter()
                .all(|item| item.label != "Open Diff Edit Mode")
        );
        view.enter_diff_edit(cx);
    });
    assert!(!view.read_with(cx, |view, _| view.diff_edit_active()));
}
