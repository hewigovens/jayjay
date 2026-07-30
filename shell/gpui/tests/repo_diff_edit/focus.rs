use gpui::{Entity, Focusable, TestAppContext, VisualTestContext};
use jayjay_gpui::repo::window::RepoWindow;

use super::fixtures::*;
use super::harness::*;

fn focus_window(view: &Entity<RepoWindow>, cx: &mut VisualTestContext) {
    view.update_in(cx, |view, window, cx| {
        view.focus_handle(cx).focus(window, cx);
    });
}

fn ordered_paths(view: &Entity<RepoWindow>, cx: &mut VisualTestContext) -> Vec<String> {
    view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .files
            .as_ref()
            .expect("files loaded")
            .iter()
            .map(|hunk| hunk.path.clone())
            .collect()
    })
}

fn focused(view: &Entity<RepoWindow>, cx: &mut VisualTestContext) -> Option<String> {
    view.read_with(cx, |view, _| view.diff_edit_focused())
}

fn enter_focused_diff_edit(
    cx: &mut TestAppContext,
) -> (Entity<RepoWindow>, &mut VisualTestContext) {
    let fixture = two_file_edits_fixture();
    let (view, cx) = open_fixture(&fixture, cx);
    view.update_in(cx, |view, _, cx| view.enter_diff_edit(cx));
    settle_visual(cx);
    focus_window(&view, cx);
    (view, cx)
}

#[gpui::test]
fn entering_diff_edit_has_no_focus_until_first_keypress(cx: &mut TestAppContext) {
    let (view, cx) = enter_focused_diff_edit(cx);
    assert_eq!(focused(&view, cx), None);
}

#[gpui::test]
fn j_and_k_move_focus_through_files_with_clamping(cx: &mut TestAppContext) {
    let (view, cx) = enter_focused_diff_edit(cx);
    let paths = ordered_paths(&view, cx);
    assert!(paths.len() >= 2, "fixture exposes two edited files");

    cx.simulate_keystrokes("j");
    assert_eq!(focused(&view, cx), Some(paths[0].clone()));

    cx.simulate_keystrokes("j");
    assert_eq!(focused(&view, cx), Some(paths[1].clone()));

    cx.simulate_keystrokes("j");
    assert_eq!(
        focused(&view, cx),
        Some(paths[1].clone()),
        "j clamps at the last file"
    );

    cx.simulate_keystrokes("k");
    assert_eq!(focused(&view, cx), Some(paths[0].clone()));

    cx.simulate_keystrokes("k");
    assert_eq!(
        focused(&view, cx),
        Some(paths[0].clone()),
        "k clamps at the first file"
    );
}

#[gpui::test]
fn k_with_no_focus_lands_on_the_last_file(cx: &mut TestAppContext) {
    let (view, cx) = enter_focused_diff_edit(cx);
    let paths = ordered_paths(&view, cx);
    cx.simulate_keystrokes("k");
    assert_eq!(focused(&view, cx), paths.last().cloned());
}

#[gpui::test]
fn enter_collapses_then_reexpands_the_focused_file(cx: &mut TestAppContext) {
    let (view, cx) = enter_focused_diff_edit(cx);
    let paths = ordered_paths(&view, cx);
    let first = paths[0].clone();

    cx.simulate_keystrokes("j");
    assert_eq!(focused(&view, cx), Some(first.clone()));
    assert!(!view.read_with(cx, |view, _| view.diff_edit_collapsed(&first)));

    cx.simulate_keystrokes("enter");
    assert!(view.read_with(cx, |view, _| view.diff_edit_collapsed(&first)));

    cx.simulate_keystrokes("enter");
    assert!(!view.read_with(cx, |view, _| view.diff_edit_collapsed(&first)));
}

#[gpui::test]
fn arrows_move_focus_and_left_right_collapse_and_expand(cx: &mut TestAppContext) {
    let (view, cx) = enter_focused_diff_edit(cx);
    let paths = ordered_paths(&view, cx);
    let first = paths[0].clone();

    cx.simulate_keystrokes("down");
    assert_eq!(focused(&view, cx), Some(first.clone()));

    cx.simulate_keystrokes("left");
    assert!(view.read_with(cx, |view, _| view.diff_edit_collapsed(&first)));

    cx.simulate_keystrokes("left");
    assert!(
        view.read_with(cx, |view, _| view.diff_edit_collapsed(&first)),
        "left on an already-collapsed card stays collapsed"
    );

    cx.simulate_keystrokes("right");
    assert!(!view.read_with(cx, |view, _| view.diff_edit_collapsed(&first)));

    cx.simulate_keystrokes("up");
    assert_eq!(
        focused(&view, cx),
        Some(first),
        "up clamps at the first file"
    );
}

#[gpui::test]
fn space_toggles_selection_of_the_focused_file(cx: &mut TestAppContext) {
    let fixture = two_file_edits_fixture();
    let (view, cx) = open_fixture(&fixture, cx);
    select_file(&view, "edit.txt", cx);
    view.update_in(cx, |view, _, cx| view.enter_diff_edit(cx));
    settle_visual(cx);
    focus_window(&view, cx);
    let paths = ordered_paths(&view, cx);
    let first = paths[0].clone();
    let reviewed = |view: &Entity<RepoWindow>, cx: &mut VisualTestContext| {
        view.read_with(cx, |view, cx| {
            let vm = view.view_model().read(cx);
            let change_id = vm.selected_change().expect("change").change_id.id.clone();
            let hunk = vm
                .files
                .as_ref()
                .and_then(|files| files.iter().find(|h| h.path == "edit.txt").cloned())
                .expect("hunk");
            view.is_reviewed(&change_id, &hunk.path, &hunk.review_identity)
        })
    };
    assert!(!reviewed(&view, cx));

    cx.simulate_keystrokes("space");
    assert_eq!(
        view.read_with(cx, |view, _| view.diff_edit_file_state(&first)),
        jayjay_gpui::repo::window::DiffEditCheckboxState::None,
        "space without focus selects nothing"
    );
    assert!(
        !reviewed(&view, cx),
        "unfocused space is consumed; it must not toggle the file column's review mark behind the overlay"
    );

    cx.simulate_keystrokes("j");
    cx.simulate_keystrokes("space");
    assert_eq!(
        view.read_with(cx, |view, _| view.diff_edit_file_state(&first)),
        jayjay_gpui::repo::window::DiffEditCheckboxState::All,
        "space selects the focused file's changed lines"
    );

    cx.simulate_keystrokes("space");
    assert_eq!(
        view.read_with(cx, |view, _| view.diff_edit_file_state(&first)),
        jayjay_gpui::repo::window::DiffEditCheckboxState::None,
        "space toggles the selection off again"
    );
}

#[gpui::test]
fn focus_keys_do_not_move_the_underlying_selection(cx: &mut TestAppContext) {
    let (view, cx) = enter_focused_diff_edit(cx);
    let (selected, selected_file_ix) = view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        (vm.selected, vm.selected_file_ix)
    });

    for key in ["j", "j", "k"] {
        cx.simulate_keystrokes(key);
    }

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert_eq!(vm.selected, selected, "DAG selection stays put");
        assert_eq!(
            vm.selected_file_ix, selected_file_ix,
            "file-list selection stays put"
        );
    });
    assert!(
        focused(&view, cx).is_some(),
        "the focus keys were consumed by diff edit"
    );
}
