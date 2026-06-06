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

#[gpui::test]
fn boot_snapshots_small_working_copy(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let vm = cx.new(|_| RepoViewModel::new(fixture.path.clone()));

    // Edit made before "open" — the FS watcher would miss it, so boot must snapshot.
    fs::write(fixture.path.join("wip1.txt"), "wip 1\nedited before boot\n")
        .expect("edit working copy file");

    vm.update(cx, |vm, cx| vm.boot(cx));
    settle(cx);

    vm.read_with(cx, |vm, _| {
        assert!(vm.error.is_none(), "boot errored: {:?}", vm.error);
        let hunk = vm
            .files
            .as_ref()
            .expect("working copy files after boot")
            .iter()
            .find(|hunk| hunk.path == "wip1.txt")
            .expect("wip1 hunk after boot");
        assert!(
            !hunk.review_identity.is_empty(),
            "boot should snapshot pre-open working copy edits on a small repo"
        );
    });
}

#[gpui::test]
fn fs_change_badges_while_reviewing_working_copy(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let vm = cx.new(|_| RepoViewModel::new(fixture.path.clone()));
    vm.update(cx, |vm, cx| vm.boot(cx));
    settle(cx);

    // Reviewing the WC in an active window → badge, don't reload the diff.
    vm.update(cx, |vm, cx| {
        vm.is_repo_window_active = true;
        assert!(
            vm.selected_change().is_some_and(|c| c.is_working_copy),
            "boot should select the working copy"
        );
        vm.handle_working_copy_change(cx);
    });

    vm.read_with(cx, |vm, _| {
        assert!(vm.loading.wc_changes, "reviewing the WC should badge");
        assert!(
            !vm.loading.refreshing,
            "badge path must not start a refresh"
        );
    });
}

#[gpui::test]
fn fs_change_after_own_mutation_is_ignored(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let vm = cx.new(|_| RepoViewModel::new(fixture.path.clone()));

    vm.update(cx, |vm, cx| {
        // Deselect so only the mutation-echo guard, not the badge path, can suppress the refresh.
        vm.last_internal_mutation_at = Some(std::time::Instant::now());
        vm.selected = None;
        vm.handle_working_copy_change(cx);
    });

    vm.read_with(cx, |vm, _| {
        assert!(
            !vm.loading.refreshing,
            "FS echo within the mutation window must not refresh"
        );
        assert!(!vm.loading.wc_changes);
    });
}

#[gpui::test]
fn load_more_shows_refresh_indicator(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let vm = cx.new(|_| RepoViewModel::new(fixture.path.clone()));

    vm.update(cx, |vm, cx| vm.load_more(cx));

    vm.read_with(cx, |vm, _| {
        assert!(vm.loading.more);
        assert!(vm.loading.refreshing);
        assert!(vm.loading.refresh_indicator);
    });

    settle(cx);

    vm.read_with(cx, |vm, _| {
        assert!(!vm.loading.more);
        assert!(vm.error.is_none(), "load more errored: {:?}", vm.error);
    });
}
