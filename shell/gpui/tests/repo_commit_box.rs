mod support;

use gpui::{TestAppContext, VisualContext, VisualTestContext};
use jayjay_gpui::repo::{ActivePane, RepoWindow};
use jj_test::LinearFixture;
use support::*;

#[gpui::test]
fn commit_box_input_commits_working_copy(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);

    let input = view.read_with(cx, |view, _| view.commit_input().clone());
    cx.focus(&input);
    cx.simulate_input("commit from gpui commit box");
    view.read_with(cx, |view, cx| {
        assert_eq!(
            view.commit_input().read(cx).text(),
            "commit from gpui commit box"
        );
    });

    view.update_in(cx, |view, _, cx| {
        view.commit_working_copy_from_input(cx);
    });
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        assert_eq!(view.commit_input().read(cx).text(), "");
        let vm = view.view_model().read(cx);
        assert!(vm.error.is_none(), "commit errored: {:?}", vm.error);
        assert!(
            vm.graph
                .changes
                .iter()
                .any(|change| change.description.trim() == "commit from gpui commit box")
        );
        let selected = vm.selected_change().expect("selected change after commit");
        assert!(selected.is_working_copy);
    });
}

#[gpui::test]
fn commit_box_keeps_input_when_commit_fails(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);

    let input = view.read_with(cx, |view, _| view.commit_input().clone());
    cx.focus(&input);
    cx.simulate_input("keep this message");

    view.update_in(cx, |view, _, cx| {
        view.view_model().update(cx, |vm, _| {
            vm.repo = None;
        });
        view.commit_working_copy_from_input(cx);
    });
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        assert_eq!(view.commit_input().read(cx).text(), "keep this message");
        assert_eq!(
            view.view_model().read(cx).error.as_deref(),
            Some("repository is not open")
        );
    });
}

#[gpui::test]
fn commit_box_space_does_not_toggle_file_review(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    add_tracked_working_copy_edits(&fixture);
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);

    let (change_id, path, identity) = view.update_in(cx, |view, _, cx| {
        view.set_active_pane(ActivePane::FileColumn);
        let vm = view.view_model().read(cx);
        let change = vm.selected_change().expect("selected change");
        let hunk = vm.selected_hunk().expect("selected hunk");
        let marker = (
            change.change_id.clone(),
            hunk.path.clone(),
            hunk.review_identity.clone(),
        );
        view.mark_unreviewed(&marker.0, &marker.1);
        marker
    });

    let input = view.read_with(cx, |view, _| view.commit_input().clone());
    cx.focus(&input);
    cx.simulate_keystrokes("space");

    view.read_with(cx, |view, cx| {
        assert_eq!(view.commit_input().read(cx).text(), " ");
        assert!(!view.is_reviewed(&change_id, &path, &identity));
    });
}
