use crate::harness::*;
use gpui::{Modifiers, MouseButton, TestAppContext};
use jj_test::{LinearFixture, run_git, run_jj_in};

#[gpui::test]
fn bookmark_chip_drag_moves_bookmark_to_target_change(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    run_jj_in(&fixture.path, &["bookmark", "create", "drag-me", "-r", "@"]);
    let (view, cx) = open_fixture(&fixture, cx);
    let target_commit_id = change_with_subject(&view, cx, "add hello").commit_id.id;

    drag_between(
        cx,
        "dag-bookmark-drag-me",
        selector(format!("dag-change-{target_commit_id}")),
    );

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert!(vm.error.is_none(), "bookmark drag errored: {:?}", vm.error);
    });
    assert!(
        bookmarks_on(&view, cx, &target_commit_id)
            .iter()
            .any(|name| name == "drag-me")
    );
}

#[gpui::test]
fn tracked_bookmark_drag_shows_push_follow_up_and_dismisses(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let _remote = create_tracked_bookmark(&fixture, "tracked-drag");
    let (view, cx) = open_fixture(&fixture, cx);
    let target_commit_id = change_with_subject(&view, cx, "add hello").commit_id.id;

    drag_between(
        cx,
        "dag-bookmark-tracked-drag",
        selector(format!("dag-change-{target_commit_id}")),
    );

    assert!(cx.debug_bounds("pending-push-banner").is_some());
    view.read_with(cx, |view, _| {
        assert_eq!(
            view.pending_push_bookmark().as_deref(),
            Some("tracked-drag")
        );
    });
    let dismiss = cx
        .debug_bounds("pending-push-dismiss")
        .expect("dismiss pending push button");
    cx.simulate_click(dismiss.center(), Modifiers::default());
    settle_visual(cx);

    assert!(cx.debug_bounds("pending-push-banner").is_none());
    view.read_with(cx, |view, _| {
        assert!(view.pending_push_bookmark().is_none());
    });
}

#[gpui::test]
fn pending_push_clears_only_when_push_is_accepted(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let _remote = create_tracked_bookmark(&fixture, "tracked-drag");
    let (view, cx) = open_fixture(&fixture, cx);
    let target_commit_id = change_with_subject(&view, cx, "add hello").commit_id.id;

    drag_between(
        cx,
        "dag-bookmark-tracked-drag",
        selector(format!("dag-change-{target_commit_id}")),
    );

    view.update_in(cx, |view, _, cx| {
        view.git_push_default(cx);
        view.confirm_pending_push(cx);
        assert_eq!(
            view.pending_push_bookmark().as_deref(),
            Some("tracked-drag")
        );
        assert_eq!(view.toast().as_deref(), Some("Push already in progress"));
    });
    settle_visual(cx);

    let push = cx
        .debug_bounds("pending-push-confirm")
        .expect("pending push button");
    cx.simulate_click(push.center(), Modifiers::default());
    view.read_with(cx, |view, _| {
        assert!(view.pending_push_bookmark().is_none());
    });
    settle_visual(cx);
}

#[gpui::test]
fn bookmark_self_drop_is_a_no_op(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    run_jj_in(
        &fixture.path,
        &["bookmark", "create", "stay-put", "-r", "@"],
    );
    let (view, cx) = open_fixture(&fixture, cx);
    let working_copy_commit_id = view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .selected_change()
            .expect("selected working copy")
            .commit_id
            .id
            .clone()
    });

    drag_between(
        cx,
        "dag-bookmark-stay-put",
        selector(format!("dag-change-{working_copy_commit_id}")),
    );

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert!(vm.error.is_none());
        assert!(view.toast().is_none());
    });
    assert!(
        bookmarks_on(&view, cx, &working_copy_commit_id)
            .iter()
            .any(|name| name == "stay-put")
    );
}

#[gpui::test]
fn working_copy_chip_drag_edits_target_change(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_fixture(&fixture, cx);
    let target_commit_id = change_with_subject(&view, cx, "add hello").commit_id.id;

    drag_between(
        cx,
        "dag-working-copy",
        selector(format!("dag-change-{target_commit_id}")),
    );

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert!(
            vm.error.is_none(),
            "working-copy drag errored: {:?}",
            vm.error
        );
        let selected = vm.selected_change().expect("selected edited change");
        assert!(selected.is_working_copy);
        assert_eq!(selected.commit_id.id, target_commit_id);
    });
}

#[gpui::test]
fn escape_cancels_active_dag_drag(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    run_jj_in(
        &fixture.path,
        &["bookmark", "create", "stay-put", "-r", "@"],
    );
    let (view, cx) = open_fixture(&fixture, cx);
    let target_commit_id = change_with_subject(&view, cx, "add hello").commit_id.id;
    let source = cx
        .debug_bounds("dag-bookmark-stay-put")
        .expect("bookmark drag source");
    let target = cx
        .debug_bounds(selector(format!("dag-change-{target_commit_id}")))
        .expect("change drop target");

    cx.simulate_mouse_down(source.center(), MouseButton::Left, Modifiers::default());
    cx.simulate_mouse_move(target.center(), MouseButton::Left, Modifiers::default());
    assert!(cx.cx.update(|cx| cx.has_active_drag()));

    cx.simulate_keystrokes("escape");

    assert!(!cx.cx.update(|cx| cx.has_active_drag()));
    cx.simulate_mouse_up(target.center(), MouseButton::Left, Modifiers::default());
    settle_visual(cx);
    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        let working_copy = vm
            .graph
            .changes
            .iter()
            .find(|change| change.is_working_copy)
            .expect("working copy after cancelled drag");
        assert!(working_copy.bookmarks.iter().any(|name| name == "stay-put"));
    });
    assert!(
        !bookmarks_on(&view, cx, &target_commit_id)
            .iter()
            .any(|name| name == "stay-put")
    );
}

#[gpui::test]
fn working_copy_drag_refuses_immutable_target(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    run_git(&fixture.path, &["tag", "release"]);
    run_jj_in(&fixture.path, &["st"]);
    let (view, cx) = open_fixture(&fixture, cx);
    let working_copy_commit_id = view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .graph
            .changes
            .iter()
            .find(|change| change.is_working_copy)
            .expect("working copy")
            .commit_id
            .id
            .clone()
    });
    let target = change_with_subject(&view, cx, "add feature");
    assert!(target.is_immutable);
    let target_commit_id = target.commit_id.id;

    drag_between(
        cx,
        "dag-working-copy",
        selector(format!("dag-change-{target_commit_id}")),
    );

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert!(vm.error.is_none());
        assert_eq!(
            vm.graph
                .changes
                .iter()
                .find(|change| change.is_working_copy)
                .expect("unchanged working copy")
                .commit_id
                .id,
            working_copy_commit_id
        );
        assert!(view.toast().is_none());
    });
}
