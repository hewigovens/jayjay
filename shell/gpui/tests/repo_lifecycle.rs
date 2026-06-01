mod support;

use std::fs;

use gpui::{AppContext, TestAppContext, VisualTestContext};
use jayjay_gpui::repo::RepoWindow;
use jayjay_gpui::repo::view_model::RepoViewModel;
use jj_test::LinearFixture;
use support::*;

#[gpui::test]
fn opens_linear_fixture_with_working_copy_selected(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let vm = cx.new(|_| RepoViewModel::new(fixture.path.clone()));

    vm.read_with(cx, |vm, _| {
        assert!(vm.error.is_none(), "open errored: {:?}", vm.error);
        assert!(vm.repo.is_some(), "repo handle should be populated");
        assert!(
            vm.graph.entries.len() >= 4,
            "linear fixture should expose at least 4 changes (initial, hello, feature, wc), got {}",
            vm.graph.entries.len()
        );
        let selected_ix = vm.selected.expect("working copy should be selected");
        let selected = &vm.graph.entries[selected_ix].change;
        assert!(selected.is_working_copy);
    });
}

#[gpui::test]
fn invalid_repo_can_be_initialized(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let repo_path = fixture.path.parent().unwrap().join("empty-repo");
    fs::create_dir(&repo_path).expect("create empty repo dir");

    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(repo_path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert!(vm.repo.is_none());
        assert!(vm.error.is_some());
    });
    view.update_in(cx, |view, _, cx| {
        view.view_model()
            .update(cx, |vm, cx| vm.initialize_repo(cx))
            .detach();
    });
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert!(vm.repo.is_some(), "repo should open after jj git init");
        assert!(vm.error.is_none(), "init/open errored: {:?}", vm.error);
    });
    assert!(repo_path.join(".jj").exists());
}

#[gpui::test]
fn manual_refresh_snapshots_working_copy(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let vm = cx.new(|_| RepoViewModel::new(fixture.path.clone()));

    fs::write(
        fixture.path.join("wip1.txt"),
        "wip 1\nchanged after gpui refresh\n",
    )
    .expect("edit working copy file");

    vm.update(cx, |vm, cx| vm.refresh(false, cx));
    settle(cx);

    vm.read_with(cx, |vm, _| {
        assert!(vm.error.is_none(), "refresh errored: {:?}", vm.error);
        let hunk = vm
            .files
            .as_ref()
            .expect("refreshed working copy files")
            .iter()
            .find(|hunk| hunk.path == "wip1.txt")
            .expect("refreshed wip1 hunk");
        assert!(
            !hunk.review_identity.is_empty(),
            "manual refresh should snapshot working copy edits"
        );
    });
}
