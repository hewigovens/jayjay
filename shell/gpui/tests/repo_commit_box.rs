mod support;

use gpui::{Modifiers, TestAppContext, VisualContext, VisualTestContext};
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

    let input = view.read_with(cx, |view, _| view.summary_input().clone());
    cx.focus(&input);
    cx.simulate_input("commit from gpui commit box");
    view.read_with(cx, |view, cx| {
        assert_eq!(
            view.summary_input().read(cx).text(),
            "commit from gpui commit box"
        );
    });

    view.update_in(cx, |view, _, cx| {
        view.commit_working_copy_from_input(cx);
    });
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        assert_eq!(view.summary_input().read(cx).text(), "");
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
fn commit_clears_working_copy_review_marks(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    add_tracked_working_copy_edits(&fixture);
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);

    let (change_id, files) = view.update_in(cx, |view, _, cx| {
        let (change_id, files) = {
            let vm = view.view_model().read(cx);
            let change = vm.selected_change().expect("selected working copy");
            assert!(change.is_working_copy);
            let files: Vec<_> = vm
                .files
                .as_ref()
                .expect("working copy files loaded")
                .iter()
                .map(|hunk| (hunk.path.clone(), hunk.review_identity.clone()))
                .collect();
            (change.change_id.id.clone(), files)
        };
        assert!(
            !files.is_empty(),
            "fixture should expose working copy files"
        );

        for (path, identity) in &files {
            view.toggle_reviewed(change_id.clone(), path.clone(), identity.clone(), cx);
            assert!(view.is_reviewed(&change_id, path, identity));
        }
        (change_id, files)
    });

    let input = view.read_with(cx, |view, _| view.summary_input().clone());
    cx.focus(&input);
    cx.simulate_input("commit reviewed working copy");
    view.update_in(cx, |view, _, cx| {
        view.commit_working_copy_from_input(cx);
    });
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert!(
            vm.graph
                .changes
                .iter()
                .any(|change| change.change_id.id == change_id && !change.is_working_copy),
            "committed change should keep the working copy change id"
        );
        for (path, identity) in &files {
            assert!(
                !view.is_reviewed(&change_id, path, identity),
                "committed file {path} should not inherit the working copy review mark"
            );
        }
    });
}

#[gpui::test]
fn commit_box_keeps_input_when_commit_fails(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);

    let input = view.read_with(cx, |view, _| view.summary_input().clone());
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
        assert_eq!(view.summary_input().read(cx).text(), "keep this message");
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
            change.change_id.id.clone(),
            hunk.path.clone(),
            hunk.review_identity.clone(),
        );
        view.mark_unreviewed(&marker.0, &marker.1);
        marker
    });

    let input = view.read_with(cx, |view, _| view.summary_input().clone());
    cx.focus(&input);
    cx.simulate_keystrokes("space");

    view.read_with(cx, |view, cx| {
        assert_eq!(view.summary_input().read(cx).text(), " ");
        assert!(!view.is_reviewed(&change_id, &path, &identity));
    });
}

#[gpui::test]
fn file_column_hide_reviewed_button_filters_reviewed_files(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    add_tracked_working_copy_edits(&fixture);
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);

    let reviewed_ix = view.update_in(cx, |view, _, cx| {
        let (change_id, files) = {
            let vm = view.view_model().read(cx);
            let change = vm.selected_change().expect("selected working copy");
            assert!(change.is_working_copy);
            let files: Vec<_> = vm
                .files
                .as_ref()
                .expect("working copy files loaded")
                .iter()
                .map(|hunk| (hunk.path.clone(), hunk.review_identity.clone()))
                .collect();
            (change.change_id.id.clone(), files)
        };
        assert!(files.len() >= 2, "fixture should expose multiple files");
        view.view_model()
            .update(cx, |vm, _| vm.selected_file_ix = Some(0));
        let (path, identity) = files[0].clone();
        view.toggle_reviewed(change_id, path, identity, cx);
        0
    });
    settle_visual(cx);

    let toggle = cx
        .debug_bounds("file-hide-reviewed")
        .expect("hide reviewed button");
    cx.simulate_click(toggle.center(), Modifiers::default());
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        assert!(view.hide_reviewed_files());
        assert_ne!(
            view.view_model().read(cx).selected_file_ix,
            Some(reviewed_ix),
            "hiding reviewed files should move selection off the hidden reviewed file"
        );
    });
}

#[gpui::test]
fn review_marks_are_shared_across_windows(cx: &mut TestAppContext) {
    // Two windows on the same repo must share one process-wide review store, not per-window copies.
    let fixture = LinearFixture::build();
    add_tracked_working_copy_edits(&fixture);
    install_test_globals(cx);

    let window_b = cx
        .add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx))
        .0;
    let (window_a, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&window_a, cx);
    settle_visual(cx);

    let (change_id, path, identity) = window_a.update_in(cx, |view, _, cx| {
        let vm = view.view_model().read(cx);
        let change = vm.selected_change().expect("selected change");
        let hunk = vm.selected_hunk().expect("selected hunk");
        let marker = (
            change.change_id.id.clone(),
            hunk.path.clone(),
            hunk.review_identity.clone(),
        );
        view.mark_unreviewed(&marker.0, &marker.1);
        marker
    });

    window_a.update_in(cx, |view, _, cx| {
        view.toggle_reviewed(change_id.clone(), path.clone(), identity.clone(), cx);
        assert!(view.is_reviewed(&change_id, &path, &identity), "A marked");
    });

    window_b.read_with(cx, |view, _| {
        assert!(
            view.is_reviewed(&change_id, &path, &identity),
            "B should observe the mark A made"
        );
    });

    // Unmarking in B is visible in A — and leaves the on-disk store as found.
    window_b.update_in(cx, |view, _, cx| {
        view.toggle_reviewed(change_id.clone(), path.clone(), identity.clone(), cx);
    });
    window_a.read_with(cx, |view, _| {
        assert!(
            !view.is_reviewed(&change_id, &path, &identity),
            "A should observe the unmark B made"
        );
    });
}
