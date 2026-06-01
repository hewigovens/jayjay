mod support;

use std::sync::Arc;

use gpui::{AppContext, Focusable, ScrollStrategy, TestAppContext, VisualTestContext, px};
use jayjay_gpui::diff::{DiffSelection, SbsSide};
use jayjay_gpui::repo::view_model::RepoViewModel;
use jayjay_gpui::repo::{ActivePane, RepoWindow, revset};
use jj_test::LinearFixture;
use support::*;

#[gpui::test]
fn reselecting_current_file_does_not_reset_diff_panel(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let view = cx.new(|cx| RepoWindow::new(fixture.path.clone(), cx));

    view.update(cx, |view, cx| {
        view.view_model().update(cx, |vm, _| {
            vm.selected_file_ix = Some(0);
        });
        view.set_active_pane(ActivePane::Sidebar);
        view.set_diff_selection(Some(DiffSelection::start(2, 3, SbsSide::Unified)));

        view.select_file(0, cx);

        assert_eq!(view.active_pane(), ActivePane::FileColumn);
        assert!(view.has_diff_selection());
        assert_eq!(view.pending_diff_scroll_target(), None);
    });
}

#[gpui::test]
fn selecting_new_file_resets_diff_scroll_to_top(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let view = cx.new(|cx| RepoWindow::new(fixture.path.clone(), cx));

    view.update(cx, |view, cx| {
        view.view_model().update(cx, |vm, _| {
            vm.selected_file_ix = Some(0);
        });
        view.set_diff_selection(Some(DiffSelection::start(2, 3, SbsSide::Unified)));
        view.set_diff_scroll_offset_y(px(-240.));

        view.select_file(1, cx);

        assert_eq!(view.view_model().read(cx).selected_file_ix, Some(1));
        assert!(!view.has_diff_selection());
        assert_eq!(view.diff_scroll_offset_y(), px(0.));
        assert_eq!(
            view.pending_diff_scroll_target(),
            Some((0, ScrollStrategy::Top, true))
        );
    });
}

#[gpui::test]
fn clear_compare_selects_fallback_when_target_is_missing(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let vm = cx.new(|_| RepoViewModel::new(fixture.path.clone()));

    let fallback = vm.read_with(cx, |vm, _| {
        vm.graph
            .changes
            .iter()
            .position(|change| change.is_working_copy)
            .unwrap_or(0)
    });
    vm.update(cx, |vm, cx| {
        vm.compare = Some(revset::CompareState {
            from_rev: "main".to_owned(),
            to_rev: "missing-change".to_owned(),
            source_change_id: None,
            target_change_id: Some("missing-change".to_owned()),
            display: revset::CompareDisplay {
                title: "Comparing".to_owned(),
                from: "main".to_owned(),
                to: "missing-change".to_owned(),
            },
        });
        vm.selected = None;
        vm.files = Some(Arc::new(Vec::new()));
        vm.selected_file_ix = Some(0);
        vm.clear_compare(cx);

        assert_eq!(vm.compare, None);
        assert_eq!(vm.selected, Some(fallback));
        assert!(vm.files.is_none());
        assert_eq!(vm.selected_file_ix, None);
        assert!(vm.current_diff.is_none());
    });
}

#[gpui::test]
fn ctrl_n_navigates_working_copy_file_list(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    add_tracked_working_copy_edits(&fixture);
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);

    view.update_in(cx, |view, window, cx| {
        view.set_active_pane(ActivePane::FileColumn);
        view.focus_handle(cx).focus(window, cx);
        let vm = view.view_model().read(cx);
        assert!(vm.files.as_ref().map(|files| files.len()).unwrap_or(0) >= 2);
        assert_eq!(vm.selected_file_ix, Some(0));
    });

    cx.simulate_keystrokes("ctrl-n");

    view.read_with(cx, |view, cx| {
        assert_eq!(view.view_model().read(cx).selected_file_ix, Some(1));
    });
}
