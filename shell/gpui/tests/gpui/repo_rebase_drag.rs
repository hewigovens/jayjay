use crate::harness::*;
use gpui::{Modifiers, TestAppContext};
use jj_test::{LinearFixture, run_git, run_jj_in};

#[gpui::test]
fn change_row_drag_confirms_and_rebases_onto_target(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_fixture(&fixture, cx);
    let source = change_with_subject(&view, cx, "add feature");
    let target_commit_id = change_with_subject(&view, cx, "initial").commit_id.id;
    let source_change_id = source.change_id.id;
    let source_commit_id = source.commit_id.id;

    drag_between(
        cx,
        selector(format!("dag-change-{source_commit_id}")),
        selector(format!("dag-change-{target_commit_id}")),
    );

    assert!(cx.debug_bounds("rebase-confirmation").is_some());
    let confirm = cx
        .debug_bounds("rebase-confirm-submit")
        .expect("rebase confirmation button");
    cx.simulate_click(confirm.center(), Modifiers::default());
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        let rebased = vm
            .graph
            .changes
            .iter()
            .find(|change| change.change_id.id == source_change_id)
            .expect("rebased change");
        assert_eq!(rebased.parents, vec![target_commit_id]);
        assert_eq!(
            vm.selected_change()
                .map(|change| change.change_id.id.as_str()),
            Some(source_change_id.as_str())
        );
        assert_eq!(view.toast().as_deref(), Some("Rebased main onto initial"));
    });
}

#[gpui::test]
fn change_row_drag_can_disable_future_confirmation(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_fixture(&fixture, cx);
    let source = change_with_subject(&view, cx, "add feature");
    let target_commit_id = change_with_subject(&view, cx, "initial").commit_id.id;
    let source_change_id = source.change_id.id;
    let source_selector = selector(format!("dag-change-{}", source.commit_id.id));
    let target_selector = selector(format!("dag-change-{target_commit_id}"));

    drag_between(cx, source_selector, target_selector);
    let toggle = cx
        .debug_bounds("rebase-confirm-toggle")
        .expect("confirm drag-to-rebase toggle");
    cx.simulate_click(toggle.center(), Modifiers::default());
    settle_visual(cx);
    let cancel = cx
        .debug_bounds("rebase-confirm-cancel")
        .expect("rebase cancel button");
    cx.simulate_click(cancel.center(), Modifiers::default());
    settle_visual(cx);

    drag_between(cx, source_selector, target_selector);

    assert!(cx.debug_bounds("rebase-confirmation").is_none());
    view.read_with(cx, |view, cx| {
        let rebased = view
            .view_model()
            .read(cx)
            .graph
            .changes
            .iter()
            .find(|change| change.change_id.id == source_change_id)
            .expect("rebased change");
        assert_eq!(rebased.parents, vec![target_commit_id]);
    });
}

#[gpui::test]
fn change_row_drop_on_only_parent_is_a_no_op(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_fixture(&fixture, cx);
    let source = change_with_subject(&view, cx, "add feature");
    let parent_commit_id = change_with_subject(&view, cx, "add hello").commit_id.id;
    let source_change_id = source.change_id.id;

    drag_between(
        cx,
        selector(format!("dag-change-{}", source.commit_id.id)),
        selector(format!("dag-change-{parent_commit_id}")),
    );

    assert!(cx.debug_bounds("rebase-confirmation").is_none());
    view.read_with(cx, |view, cx| {
        let source = view
            .view_model()
            .read(cx)
            .graph
            .changes
            .iter()
            .find(|change| change.change_id.id == source_change_id)
            .expect("source change");
        assert_eq!(source.parents, vec![parent_commit_id]);
    });
}

#[gpui::test]
fn rebasing_a_divergent_sibling_keeps_that_sibling_selected(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let base_op = run_jj_in(
        &fixture.path,
        &["op", "log", "--no-graph", "--limit", "1", "-T", "id"],
    );
    let base_op = String::from_utf8(base_op.stdout).expect("utf-8 op id");
    run_jj_in(
        &fixture.path,
        &[
            "describe",
            "-r",
            "subject(\"add hello\")",
            "-m",
            "add hello (alt)",
        ],
    );
    run_jj_in(
        &fixture.path,
        &[
            "--at-op",
            base_op.trim(),
            "describe",
            "-r",
            "subject(\"add hello\")",
            "-m",
            "add hello (orig)",
        ],
    );
    run_jj_in(
        &fixture.path,
        &["new", "--no-edit", "-m", "side", "subject(\"initial\")"],
    );
    let (view, cx) = open_fixture(&fixture, cx);
    let source = change_with_subject(&view, cx, "add hello (alt)");
    assert!(source.is_divergent, "fixture must be divergent");
    let target_commit_id = change_with_subject(&view, cx, "side").commit_id.id;

    drag_between(
        cx,
        selector(format!("dag-change-{}", source.commit_id.id)),
        selector(format!("dag-change-{target_commit_id}")),
    );
    let confirm = cx
        .debug_bounds("rebase-confirm-submit")
        .expect("rebase confirmation button");
    cx.simulate_click(confirm.center(), Modifiers::default());
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert!(vm.error.is_none(), "{:?}", vm.error);
        let selected = vm.selected_change().expect("selected change");
        assert_eq!(selected.description.trim(), "add hello (alt)");
        assert_eq!(selected.parents, vec![target_commit_id]);
    });
}

#[gpui::test]
fn change_row_drop_on_a_descendant_is_refused(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_fixture(&fixture, cx);
    let source = change_with_subject(&view, cx, "add hello");
    let descendant_commit_id = change_with_subject(&view, cx, "add feature").commit_id.id;
    let source_change_id = source.change_id.id;
    let source_parents = source.parents.clone();

    drag_between(
        cx,
        selector(format!("dag-change-{}", source.commit_id.id)),
        selector(format!("dag-change-{descendant_commit_id}")),
    );

    assert!(cx.debug_bounds("rebase-confirmation").is_none());
    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert!(vm.error.is_none(), "{:?}", vm.error);
        let source = vm
            .graph
            .changes
            .iter()
            .find(|change| change.change_id.id == source_change_id)
            .expect("source change");
        assert_eq!(source.parents, source_parents);
        assert!(view.toast().is_none());
    });
}

#[gpui::test]
fn immutable_change_row_cannot_be_dragged_to_rebase(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    run_git(&fixture.path, &["tag", "release"]);
    run_jj_in(&fixture.path, &["st"]);
    let (view, cx) = open_fixture(&fixture, cx);
    // The default view's context depth no longer reaches "initial"; widen it so both rows are loaded.
    view.update(cx, |view, cx| {
        view.view_model()
            .update(cx, |vm, cx| vm.apply_revset("all()", cx));
    });
    settle_visual(cx);
    let source = change_with_subject(&view, cx, "add feature");
    let target_commit_id = change_with_subject(&view, cx, "initial").commit_id.id;
    let source_change_id = source.change_id.id;

    drag_between(
        cx,
        selector(format!("dag-change-{}", source.commit_id.id)),
        selector(format!("dag-change-{target_commit_id}")),
    );

    assert!(cx.debug_bounds("rebase-confirmation").is_none());
    view.read_with(cx, |view, cx| {
        let source = view
            .view_model()
            .read(cx)
            .graph
            .changes
            .iter()
            .find(|change| change.change_id.id == source_change_id)
            .expect("immutable source change");
        assert!(source.is_immutable);
    });
}

#[gpui::test]
fn rebase_confirmation_is_cancelled_when_the_source_was_rewritten_meanwhile(
    cx: &mut TestAppContext,
) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_fixture(&fixture, cx);
    let source = change_with_subject(&view, cx, "add feature");
    let target_commit_id = change_with_subject(&view, cx, "initial").commit_id.id;
    let original_parents = source.parents.clone();

    drag_between(
        cx,
        selector(format!("dag-change-{}", source.commit_id.id)),
        selector(format!("dag-change-{target_commit_id}")),
    );
    assert!(cx.debug_bounds("rebase-confirmation").is_some());
    run_jj_in(
        &fixture.path,
        &[
            "describe",
            "-r",
            &source.change_id.id,
            "-m",
            "add feature (rewritten)",
        ],
    );
    view.update_in(cx, |view, _, cx| {
        view.view_model().update(cx, |vm, cx| vm.refresh(false, cx));
    });
    settle_visual(cx);
    let confirm = cx
        .debug_bounds("rebase-confirm-submit")
        .expect("rebase confirmation button");
    cx.simulate_click(confirm.center(), Modifiers::default());
    settle_visual(cx);

    assert!(cx.debug_bounds("rebase-confirmation").is_none());
    view.read_with(cx, |view, _| {
        assert_eq!(
            view.toast().as_deref(),
            Some("Rebase cancelled: the changes moved while confirming")
        );
    });
    let rewritten = change_with_subject(&view, cx, "add feature (rewritten)");
    assert_eq!(rewritten.parents, original_parents);
}
