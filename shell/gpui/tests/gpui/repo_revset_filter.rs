use crate::harness::{install_test_globals, settle, settle_visual, suppress_fs_watcher};
use gpui::{AppContext, Modifiers, TestAppContext, VisualTestContext};
use jayjay_core::{DEFAULT_REVSET_DEPTH, build_default_revset};
use jayjay_gpui::repo::RepoWindow;
use jj_test::LinearFixture;

#[gpui::test]
fn toolbar_sync_arrows_are_centered_in_their_circles(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    install_test_globals(cx);
    let (_, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);

    for (action, icon_selector, arrow_selector) in [
        ("tb-pull", "sync-icon-tb-pull", "sync-arrow-tb-pull"),
        ("tb-push", "sync-icon-tb-push", "sync-arrow-tb-push"),
    ] {
        let icon = cx.debug_bounds(icon_selector).expect("sync icon bounds");
        let arrow = cx.debug_bounds(arrow_selector).expect("sync arrow bounds");
        assert_eq!(
            arrow.center(),
            icon.center(),
            "{action} arrow should be centered in its fixed circle"
        );
    }
}

#[gpui::test]
fn toolbar_revset_filter_applies_custom_input_and_resets(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);

    let filter = cx
        .debug_bounds("toolbar-revset-filter")
        .expect("revset filter toolbar button");
    let refresh = cx
        .debug_bounds("toolbar-refresh")
        .expect("refresh toolbar button");
    let sync_cluster = cx
        .debug_bounds("toolbar-sync-cluster")
        .expect("shared filter and sync toolbar group");
    assert_eq!(
        filter.origin.x + filter.size.width,
        refresh.origin.x,
        "filter should be the leading segment of the refresh/pull/push group"
    );
    assert!(
        filter.origin.x >= sync_cluster.origin.x
            && refresh.origin.x + refresh.size.width
                <= sync_cluster.origin.x + sync_cluster.size.width
    );
    cx.simulate_click(filter.center(), Modifiers::default());
    settle_visual(cx);
    assert!(cx.debug_bounds("revset-filter").is_some());
    assert!(cx.debug_bounds("revset-chip-heads").is_some());
    let input = cx
        .debug_bounds("revset-filter-input")
        .expect("revset text field");
    cx.simulate_click(input.center(), Modifiers::default());
    settle_visual(cx);
    let caret = cx
        .debug_bounds("revset-filter-caret")
        .expect("focused revset caret");
    assert!(
        caret.origin.x >= input.origin.x
            && caret.origin.x + caret.size.width <= input.origin.x + input.size.width,
        "long revset should scroll horizontally to keep its trailing caret inside the field"
    );

    cx.simulate_keystrokes("cmd-a");
    settle_visual(cx);
    assert!(
        cx.debug_bounds("line-input-selection").is_some(),
        "select-all should render a visible selection segment"
    );
    cx.simulate_input("trunk()");
    cx.simulate_keystrokes("enter");
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        let expected: Vec<_> = vm
            .repo
            .as_ref()
            .expect("open repo")
            .log_graph("trunk()")
            .expect("trunk revset")
            .into_iter()
            .map(|entry| entry.change.commit_id.id)
            .collect();
        let actual: Vec<_> = vm
            .graph
            .changes
            .iter()
            .map(|change| change.commit_id.id.clone())
            .collect();
        assert_eq!(vm.revset.as_ref(), "trunk()");
        assert_eq!(actual, expected);
        assert!(!vm.can_load_more);
    });
    assert!(cx.debug_bounds("load-more").is_none());

    let conflicts = cx
        .debug_bounds("revset-chip-conflicts")
        .expect("conflicts revset chip");
    cx.simulate_click(conflicts.center(), Modifiers::default());
    settle_visual(cx);
    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert_eq!(vm.revset.as_ref(), "conflicts()");
        assert!(vm.error.is_none());
        assert!(vm.graph.changes.is_empty());
        assert!(vm.selected.is_none());
        assert!(vm.files.is_none());
        assert!(vm.current_diff.is_none());
    });

    let reset = cx
        .debug_bounds("revset-filter-reset")
        .expect("reset revset button");
    cx.simulate_click(reset.center(), Modifiers::default());
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        let expected = build_default_revset(DEFAULT_REVSET_DEPTH);
        assert_eq!(view.view_model().read(cx).revset.as_ref(), expected);
    });
}

#[gpui::test]
fn invalid_revset_keeps_the_loaded_graph(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    suppress_fs_watcher(cx);
    let view = cx.new(|cx| RepoWindow::new(fixture.path.clone(), cx));
    settle(cx);

    let before = view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .graph
            .changes
            .iter()
            .map(|change| change.commit_id.id.clone())
            .collect::<Vec<_>>()
    });
    view.update(cx, |view, cx| {
        view.view_model()
            .update(cx, |vm, cx| vm.apply_revset("invalid(", cx));
    });
    settle(cx);

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        let after: Vec<_> = vm
            .graph
            .changes
            .iter()
            .map(|change| change.commit_id.id.clone())
            .collect();
        assert_eq!(vm.revset.as_ref(), "invalid(");
        assert!(vm.error.is_some());
        assert_eq!(after, before);
        assert!(!vm.can_load_more);
    });
}
