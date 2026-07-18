mod support;

use gpui::{AppContext, TestAppContext};
use jayjay_gpui::repo::RepoWindow;
use jayjay_gpui::repo::revset;
use jayjay_gpui::repo::view_model::RepoViewModel;
use jayjay_gpui::ui::context_menu::ContextAction;
use jj_test::{LinearFixture, run_jj_in};
use support::*;

#[gpui::test]
fn describe_change_refreshes_graph(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let vm = cx.new(|_| RepoViewModel::new(fixture.path.clone()));

    let rev = vm.read_with(cx, |vm, _| {
        revset::change_revision(vm.selected_change().expect("selected change"))
    });
    vm.update(cx, |vm, cx| {
        vm.describe_change(rev, "updated from gpui".to_owned(), cx)
            .detach();
    });
    settle(cx);

    vm.read_with(cx, |vm, _| {
        let selected = vm
            .selected_change()
            .expect("selected change after describe");
        assert_eq!(selected.description, "updated from gpui");
        assert!(vm.error.is_none(), "describe errored: {:?}", vm.error);
    });
}

#[gpui::test]
fn working_copy_description_cannot_be_edited(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    suppress_fs_watcher(cx);
    let view = cx.new(|cx| RepoWindow::new(fixture.path.clone(), cx));
    settle(cx); // repo opens async now

    view.update(cx, |view, cx| {
        assert!(
            view.view_model()
                .read(cx)
                .selected_change()
                .expect("selected change")
                .is_working_copy
        );
        view.edit_selected_description(cx);
        assert!(!view.has_text_modal());
    });
}

#[gpui::test]
fn committing_working_copy_selects_new_working_copy(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let vm = cx.new(|_| RepoViewModel::new(fixture.path.clone()));

    vm.update(cx, |vm, cx| {
        vm.commit_working_copy("commit from gpui".to_owned(), cx)
            .detach();
    });
    settle(cx);

    vm.read_with(cx, |vm, _| {
        assert!(vm.error.is_none(), "commit errored: {:?}", vm.error);
        assert!(
            vm.graph
                .changes
                .iter()
                .any(|change| change.description.trim() == "commit from gpui")
        );
        let selected = vm.selected_change().expect("selected change after commit");
        assert!(selected.is_working_copy);
    });
}

#[gpui::test]
fn change_context_action_creates_new_change_on_top(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    suppress_fs_watcher(cx);
    let view = cx.new(|cx| RepoWindow::new(fixture.path.clone(), cx));
    settle(cx); // repo opens async now

    let parent_commit_id = view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .graph
            .changes
            .iter()
            .find(|change| change.description.trim() == "add hello")
            .expect("fixture should contain add hello change")
            .commit_id
            .id
            .clone()
    });
    let parent_rev = parent_commit_id.clone();
    view.update(cx, |view, cx| {
        view.dispatch_context_action(ContextAction::NewChangeOnTop(parent_rev.into()), cx);
    });
    settle(cx);

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert!(vm.error.is_none(), "new change errored: {:?}", vm.error);
        let selected = vm
            .selected_change()
            .expect("selected change after new change");
        assert!(selected.is_working_copy);
        assert_eq!(selected.parents.first(), Some(&parent_commit_id));
    });
}

#[gpui::test]
fn change_context_action_abandons_change(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    suppress_fs_watcher(cx);
    let view = cx.new(|cx| RepoWindow::new(fixture.path.clone(), cx));
    settle(cx); // repo opens async now

    let target_commit_id = view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .graph
            .changes
            .iter()
            .find(|change| change.description.trim() == "add hello")
            .expect("fixture should contain add hello change")
            .commit_id
            .id
            .clone()
    });
    let target_rev = target_commit_id.clone();
    view.update(cx, |view, cx| {
        view.dispatch_context_action(ContextAction::AbandonChange(target_rev.into()), cx);
    });
    settle(cx);

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert!(vm.error.is_none(), "abandon errored: {:?}", vm.error);
        assert!(
            vm.graph
                .changes
                .iter()
                .all(|change| change.commit_id != target_commit_id)
        );
        assert!(
            vm.selected_change()
                .is_some_and(|change| change.is_working_copy)
        );
    });
}

#[gpui::test]
fn create_bookmark_adds_bookmark_to_selected_change(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let vm = cx.new(|_| RepoViewModel::new(fixture.path.clone()));

    let rev = vm.read_with(cx, |vm, _| {
        revset::change_revision(vm.selected_change().expect("selected change"))
    });
    vm.update(cx, |vm, cx| {
        vm.create_bookmark("feature-x".to_owned(), rev, cx).detach();
    });
    settle(cx);

    vm.read_with(cx, |vm, _| {
        assert!(
            vm.error.is_none(),
            "create bookmark errored: {:?}",
            vm.error
        );
        assert!(
            vm.graph
                .changes
                .iter()
                .any(|change| change.bookmarks.iter().any(|b| b == "feature-x")),
            "new bookmark should be visible in graph"
        );
    });
}

#[gpui::test]
fn move_bookmark_drops_bookmark_onto_target_revision(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    run_jj_in(&fixture.path, &["bookmark", "create", "drag-me", "-r", "@"]);
    let vm = cx.new(|_| RepoViewModel::new(fixture.path.clone()));

    // The grandparent of @ — a distinct destination to drop the bookmark on.
    let target_commit_id = vm.read_with(cx, |vm, _| {
        vm.graph
            .changes
            .iter()
            .find(|change| change.description.trim() == "add hello")
            .expect("fixture should contain add hello change")
            .commit_id
            .id
            .clone()
    });
    let dest = target_commit_id.clone();
    vm.update(cx, |vm, cx| {
        vm.move_bookmark("drag-me".to_owned(), dest, cx).detach();
    });
    settle(cx);

    vm.read_with(cx, |vm, _| {
        assert!(vm.error.is_none(), "move bookmark errored: {:?}", vm.error);
        let moved = vm
            .graph
            .changes
            .iter()
            .find(|change| change.bookmarks.iter().any(|b| b == "drag-me"))
            .expect("dragged bookmark should be visible in graph");
        assert_eq!(moved.commit_id.id, target_commit_id);
    });
}

#[gpui::test]
fn bookmark_context_action_moves_bookmark_to_parent(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    run_jj_in(
        &fixture.path,
        &["bookmark", "create", "move-me", "-r", "@--"],
    );
    suppress_fs_watcher(cx);
    let view = cx.new(|cx| RepoWindow::new(fixture.path.clone(), cx));
    settle(cx); // repo opens async now

    let parent_commit_id = view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .selected_change()
            .expect("selected working copy")
            .parents
            .first()
            .expect("working copy parent")
            .clone()
    });
    view.update(cx, |view, cx| {
        view.dispatch_context_action(ContextAction::MoveBookmarkToParent("move-me".into()), cx);
    });
    settle(cx);

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert!(vm.error.is_none(), "move bookmark errored: {:?}", vm.error);
        let moved = vm
            .graph
            .changes
            .iter()
            .find(|change| {
                change
                    .bookmarks
                    .iter()
                    .any(|bookmark| bookmark == "move-me")
            })
            .expect("moved bookmark should be visible in graph");
        assert_eq!(moved.commit_id, parent_commit_id);
    });
}

#[gpui::test]
fn repeated_push_while_in_flight_shows_feedback(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    suppress_fs_watcher(cx);
    let view = cx.new(|cx| RepoWindow::new(fixture.path.clone(), cx));
    settle(cx);

    view.update(cx, |view, cx| {
        view.view_model()
            .update(cx, |vm, _| vm.loading.refresh_indicator = false);
        view.git_push_default(cx);
        let vm = view.view_model();
        let vm = vm.read(cx);
        assert!(
            vm.loading.refreshing,
            "push should keep repository tasks gated"
        );
        assert!(
            !vm.loading.refresh_indicator,
            "refresh should not spin until push completes"
        );
        view.git_push_default(cx);
        assert_eq!(view.toast().as_deref(), Some("Push already in progress"));
    });
}

#[gpui::test]
fn repeated_pull_while_in_flight_shows_feedback(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    suppress_fs_watcher(cx);
    let view = cx.new(|cx| RepoWindow::new(fixture.path.clone(), cx));
    settle(cx);

    view.update(cx, |view, cx| {
        view.view_model()
            .update(cx, |vm, _| vm.loading.refresh_indicator = false);
        view.git_fetch_origin(cx);
        let vm = view.view_model();
        let vm = vm.read(cx);
        assert!(
            vm.loading.refreshing,
            "pull should keep repository tasks gated"
        );
        assert!(
            !vm.loading.refresh_indicator,
            "refresh should not spin until pull completes"
        );
        view.git_fetch_origin(cx);
        assert_eq!(view.toast().as_deref(), Some("Pull already in progress"));
    });
}
