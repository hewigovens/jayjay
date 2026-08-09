mod harness;

use std::sync::Arc;

use gpui::{AppContext, TestAppContext};
use harness::*;
use jayjay_gpui::repo::RepoWindow;
use jayjay_gpui::repo::revset;
use jayjay_gpui::repo::view_model::RepoViewModel;
use jayjay_gpui::repo::window::ChangeAction;
use jayjay_gpui::ui::context_menu::ContextAction;
use jj_test::{LinearFixture, run_jj_in};

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
fn change_menu_exposes_full_mutation_set_and_selected_pair_actions(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    suppress_fs_watcher(cx);
    let view = cx.new(|cx| RepoWindow::new(fixture.path.clone(), cx));
    settle(cx);

    view.update(cx, |view, cx| {
        let (selected_ix, clicked) = {
            let vm = view.view_model().read(cx);
            let selected_ix = vm
                .graph
                .changes
                .iter()
                .position(|change| change.description.trim() == "add hello")
                .expect("add hello change");
            let clicked = vm
                .graph
                .changes
                .iter()
                .find(|change| change.description.trim() == "add feature")
                .expect("add feature change")
                .clone();
            (selected_ix, clicked)
        };
        view.view_model()
            .update(cx, |vm, _| vm.selected = Some(selected_ix));
        let labels: Vec<_> = view
            .build_change_menu(&clicked, cx)
            .iter()
            .map(|item| item.label.to_string())
            .collect();
        for expected in [
            "Edit (modify this change)",
            "Squash into parent",
            "Move changes to working copy",
            "Rebase selected onto this",
            "Squash selected into this",
            "Merge with selected",
            "Duplicate",
            "Absorb into ancestors",
            "Revert change",
        ] {
            assert!(
                labels.iter().any(|label| label == expected),
                "{expected}: {labels:?}"
            );
        }
    });
}

#[gpui::test]
fn change_menu_hides_squash_when_parent_is_immutable(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    suppress_fs_watcher(cx);
    let view = cx.new(|cx| RepoWindow::new(fixture.path.clone(), cx));
    settle(cx);

    view.update(cx, |view, cx| {
        let clicked = view
            .view_model()
            .read(cx)
            .graph
            .changes
            .iter()
            .find(|change| change.description.trim() == "add feature")
            .expect("add feature change")
            .clone();
        let parent_id = clicked.parents.first().expect("first parent").clone();
        view.view_model().update(cx, |vm, _| {
            Arc::make_mut(&mut vm.graph.changes)
                .iter_mut()
                .find(|change| change.commit_id.id == parent_id)
                .expect("visible parent")
                .is_immutable = true;
        });

        let labels: Vec<_> = view
            .build_change_menu(&clicked, cx)
            .iter()
            .map(|item| item.label.to_string())
            .collect();
        assert!(
            labels
                .iter()
                .any(|label| label == "Edit (modify this change)")
        );
        assert!(!labels.iter().any(|label| label == "Squash into parent"));
    });
}

#[gpui::test]
fn change_menu_keeps_squash_when_parent_is_outside_loaded_page(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    suppress_fs_watcher(cx);
    let view = cx.new(|cx| RepoWindow::new(fixture.path.clone(), cx));
    settle(cx);

    view.update(cx, |view, cx| {
        let clicked = view
            .view_model()
            .read(cx)
            .graph
            .changes
            .iter()
            .find(|change| change.description.trim() == "add feature")
            .expect("add feature change")
            .clone();
        let parent_id = clicked.parents.first().expect("first parent").clone();
        view.view_model().update(cx, |vm, _| {
            Arc::make_mut(&mut vm.graph.changes).retain(|change| change.commit_id.id != parent_id);
        });

        let labels: Vec<_> = view
            .build_change_menu(&clicked, cx)
            .iter()
            .map(|item| item.label.to_string())
            .collect();
        assert!(labels.iter().any(|label| label == "Squash into parent"));
    });
}

#[gpui::test]
fn edit_change_context_action_makes_target_the_working_copy(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    suppress_fs_watcher(cx);
    let view = cx.new(|cx| RepoWindow::new(fixture.path.clone(), cx));
    settle(cx);

    let rev = view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        revset::change_revision(
            vm.graph
                .changes
                .iter()
                .find(|change| change.description.trim() == "add hello")
                .expect("add hello change"),
        )
    });
    view.update(cx, |view, cx| {
        view.dispatch_context_action(
            ContextAction::Change(Arc::new(ChangeAction::Edit { rev })),
            cx,
        );
    });
    settle(cx);

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert!(vm.error.is_none(), "edit errored: {:?}", vm.error);
        let selected = vm.selected_change().expect("selected edited change");
        assert!(selected.is_working_copy);
        assert_eq!(selected.description.trim(), "add hello");
    });
}

#[gpui::test]
fn duplicate_change_context_action_refreshes_the_graph(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    suppress_fs_watcher(cx);
    let view = cx.new(|cx| RepoWindow::new(fixture.path.clone(), cx));
    settle(cx);

    let rev = view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        revset::change_revision(
            vm.graph
                .changes
                .iter()
                .find(|change| change.description.trim() == "add hello")
                .expect("add hello change"),
        )
    });
    view.update(cx, |view, cx| {
        view.dispatch_context_action(
            ContextAction::Change(Arc::new(ChangeAction::Duplicate { rev })),
            cx,
        );
    });
    settle(cx);

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert!(vm.error.is_none(), "duplicate errored: {:?}", vm.error);
        assert_eq!(
            vm.graph
                .changes
                .iter()
                .filter(|change| change.description.trim() == "add hello")
                .count(),
            2
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
        view.dispatch_context_action(
            ContextAction::MoveBookmark {
                name: "move-me".into(),
                to_rev: "@-".into(),
            },
            cx,
        );
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
fn bookmark_context_action_moves_bookmark_to_working_copy(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    run_jj_in(
        &fixture.path,
        &["bookmark", "create", "resolve-me", "-r", "@--"],
    );
    suppress_fs_watcher(cx);
    let view = cx.new(|cx| RepoWindow::new(fixture.path.clone(), cx));
    settle(cx);

    let working_copy_commit_id = view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .selected_change()
            .expect("selected working copy")
            .commit_id
            .clone()
    });
    view.update(cx, |view, cx| {
        view.dispatch_context_action(
            ContextAction::MoveBookmark {
                name: "resolve-me".into(),
                to_rev: "@".into(),
            },
            cx,
        );
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
                    .any(|bookmark| bookmark == "resolve-me")
            })
            .expect("resolved bookmark should be visible in graph");
        assert_eq!(moved.commit_id, working_copy_commit_id);
    });
}

#[gpui::test]
fn bookmark_context_action_removes_bookmark_from_change(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    run_jj_in(
        &fixture.path,
        &["bookmark", "create", "remove-me", "-r", "@--"],
    );
    suppress_fs_watcher(cx);
    let view = cx.new(|cx| RepoWindow::new(fixture.path.clone(), cx));
    settle(cx);

    let rev = view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .graph
            .changes
            .iter()
            .find(|change| {
                change
                    .bookmarks
                    .iter()
                    .any(|bookmark| bookmark == "remove-me")
            })
            .expect("bookmark should be visible in graph")
            .commit_id
            .id
            .clone()
    });
    view.update(cx, |view, cx| {
        view.dispatch_context_action(
            ContextAction::DeleteBookmark {
                name: "remove-me".into(),
                rev: rev.into(),
            },
            cx,
        );
    });
    settle(cx);

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert!(
            vm.error.is_none(),
            "remove bookmark errored: {:?}",
            vm.error
        );
        assert!(
            vm.graph.changes.iter().all(|change| {
                !change
                    .bookmarks
                    .iter()
                    .any(|bookmark| bookmark == "remove-me")
            }),
            "remove-me should no longer appear on any change"
        );
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
