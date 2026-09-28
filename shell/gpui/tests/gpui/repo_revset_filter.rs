use crate::harness::{install_test_globals, settle, settle_visual};
use gpui::{AppContext, Entity, Modifiers, TestAppContext, VisualContext, VisualTestContext};
use jayjay_core::{DEFAULT_REVSET_DEPTH, build_default_revset};
use jayjay_gpui::repo::RepoWindow;
use jayjay_gpui::ui::context_menu::ContextAction;
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

fn graph_commit_ids(view: &Entity<RepoWindow>, cx: &mut VisualTestContext) -> Vec<String> {
    view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .graph
            .changes
            .iter()
            .map(|change| change.commit_id.id.clone())
            .collect()
    })
}

fn click(cx: &mut VisualTestContext, selector: &'static str) {
    let bounds = cx.debug_bounds(selector).expect(selector);
    cx.simulate_click(bounds.center(), Modifiers::default());
    settle_visual(cx);
}

#[gpui::test]
fn sidebar_toggle_leads_the_sync_group(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    install_test_globals(cx);
    let (_, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);

    let toggle = cx
        .debug_bounds("toolbar-sidebar-toggle")
        .expect("sidebar toggle");
    let refresh = cx.debug_bounds("toolbar-refresh").expect("refresh button");
    let sync_cluster = cx.debug_bounds("toolbar-sync-cluster").expect("sync group");
    assert_eq!(toggle.origin.x + toggle.size.width, refresh.origin.x);
    assert_eq!(toggle.origin.x, sync_cluster.origin.x);
    let bar = cx.debug_bounds("revset-bar").expect("revset bar");
    assert!(bar.origin.x > sync_cluster.origin.x + sync_cluster.size.width);
}

#[gpui::test]
fn revset_bar_edits_in_place_and_resets(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);
    let default_revset = build_default_revset(DEFAULT_REVSET_DEPTH);

    click(cx, "revset-summary");
    assert_eq!(
        view.read_with(cx, |view, _| view.revset_editor_text()),
        Some(default_revset.clone())
    );
    let bar = cx.debug_bounds("revset-bar").expect("revset bar");
    let caret = cx
        .debug_bounds("revset-editor-caret")
        .expect("focused revset caret");
    assert!(
        caret.origin.x + caret.size.width <= bar.origin.x + bar.size.width,
        "a long revset scrolls to keep its trailing caret inside the bar"
    );

    cx.simulate_keystrokes("cmd-a");
    cx.simulate_input("@ | trunk()");
    cx.simulate_keystrokes("enter");
    settle_visual(cx);
    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        let expected: Vec<_> = vm
            .repo
            .as_ref()
            .expect("open repo")
            .log_graph("@ | trunk()")
            .expect("trunk revset")
            .into_iter()
            .map(|entry| entry.change.commit_id.id)
            .collect();
        assert_eq!(vm.revset(), "@ | trunk()");
        assert!(!vm.can_load_more);
        assert!(view.revset_editor_text().is_none());
        assert_eq!(
            vm.graph
                .changes
                .iter()
                .map(|change| change.commit_id.id.clone())
                .collect::<Vec<_>>(),
            expected
        );
    });
    assert!(cx.debug_bounds("load-more").is_none());

    click(cx, "revset-presets");
    assert!(cx.debug_bounds("revset-popup").is_some());
    click(cx, "revset-presets");
    assert!(!view.read_with(cx, |view, _| view.revset_popup_open()));
    click(cx, "revset-presets");
    click(cx, "revset-chip-conflicts");
    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert_eq!(vm.revset(), "conflicts()");
        assert!(!view.revset_popup_open());
        assert!(vm.error.is_none());
        assert!(vm.graph.changes.is_empty());
        assert!(vm.selected.is_none());
    });

    click(cx, "revset-presets");
    click(cx, "revset-popup-row-Recent-@ | trunk()");
    view.read_with(cx, |view, cx| {
        assert_eq!(view.view_model().read(cx).revset(), "@ | trunk()");
    });

    click(cx, "revset-reset");
    view.read_with(cx, |view, cx| {
        assert_eq!(view.view_model().read(cx).revset(), default_revset);
    });
    assert!(cx.debug_bounds("revset-reset").is_none());
}

#[gpui::test]
fn revset_edit_cancels_on_escape_or_a_title_bar_click(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);
    let before = view.read_with(cx, |view, cx| {
        view.view_model().read(cx).revset().to_owned()
    });

    cx.simulate_keystrokes(&format!("{}-l", jayjay_gpui::platform::MOD_KEY));
    settle_visual(cx);
    cx.simulate_input("all()");
    cx.simulate_keystrokes("escape");
    settle_visual(cx);
    view.read_with(cx, |view, cx| {
        assert!(view.revset_editor_text().is_none());
        assert_eq!(view.view_model().read(cx).revset(), before);
    });

    click(cx, "revset-summary");
    assert!(view.read_with(cx, |view, _| view.revset_editor_text().is_some()));
    let bar = cx.debug_bounds("revset-bar").expect("revset bar");
    cx.simulate_click(
        gpui::point(bar.origin.x - gpui::px(3.), bar.center().y),
        Modifiers::default(),
    );
    settle_visual(cx);
    assert!(view.read_with(cx, |view, _| view.revset_editor_text().is_none()));
}

#[gpui::test]
fn the_popup_takes_typing_from_a_focused_text_field(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);
    let summary = view.read_with(cx, |view, _| view.summary_input());
    cx.focus(&summary);

    click(cx, "revset-presets");
    cx.simulate_input("all()");
    cx.simulate_keystrokes("enter");
    settle_visual(cx);
    view.read_with(cx, |view, cx| {
        assert_eq!(view.summary_input().read(cx).text(), "");
        assert_eq!(view.view_model().read(cx).revset(), "all()");
    });
}

#[gpui::test]
fn invalid_typed_revset_opens_the_popup_with_its_error(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);
    let before = graph_commit_ids(&view, cx);

    click(cx, "revset-summary");
    cx.simulate_keystrokes("cmd-a");
    cx.simulate_input("invalid(");
    cx.simulate_keystrokes("enter");
    settle_visual(cx);
    assert!(cx.debug_bounds("revset-popup-error").is_some());
    assert!(view.read_with(cx, |view, _| view.revset_popup_open()));
    assert_eq!(graph_commit_ids(&view, cx), before);

    cx.simulate_keystrokes("escape");
    settle_visual(cx);
    assert!(!view.read_with(cx, |view, _| view.revset_popup_open()));
}

#[gpui::test]
fn invalid_revset_keeps_the_loaded_graph(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    install_test_globals(cx);
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
        assert_eq!(vm.revset(), "invalid(");
        assert!(vm.error.is_some());
        assert_eq!(after, before);
        assert!(!vm.can_load_more);
    });
}

#[gpui::test]
fn show_ancestors_preserves_target_and_returns_to_custom_filter(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);
    view.update(cx, |view, cx| {
        view.view_model()
            .update(cx, |vm, cx| vm.apply_revset("all()", cx));
    });
    settle_visual(cx);
    let target = view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .graph
            .changes
            .iter()
            .find(|change| !change.is_working_copy && !change.parents.is_empty())
            .expect("non-working-copy change")
            .clone()
    });
    let action = view.read_with(cx, |view, cx| {
        view.build_change_menu(&target, cx)
            .into_iter()
            .find(|item| item.label.as_ref() == "Show ancestors…")
            .expect("ancestors menu item")
            .action
    });
    view.update(cx, |view, cx| view.dispatch_context_action(action, cx));
    settle_visual(cx);
    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert_eq!(
            vm.graph.changes[vm.selected.unwrap()].commit_id,
            target.commit_id
        );
        assert!(!vm.graph.changes.iter().any(|change| change.is_working_copy));
        assert!(vm.graph.changes.len() >= 2);
        assert!(vm.error.is_none());
    });
    click(cx, "revset-back");
    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert_eq!(vm.revset(), "all()");
        assert!(vm.graph.changes.iter().any(|change| change.is_working_copy));
    });
    assert!(cx.debug_bounds("revset-back").is_none());

    view.update(cx, |view, cx| {
        view.dispatch_context_action(ContextAction::ShowAncestors(target.commit_id.id.into()), cx);
    });
    settle_visual(cx);
    assert!(cx.debug_bounds("revset-back").is_some());
    view.update(cx, |view, cx| {
        view.dispatch_context_action(ContextAction::FilterBookmarkRevset("trunk()".into()), cx);
    });
    settle_visual(cx);
    assert!(cx.debug_bounds("revset-back").is_none());
    view.read_with(cx, |view, cx| {
        assert_eq!(view.view_model().read(cx).revset(), "trunk()");
    });
}

#[gpui::test]
fn an_edit_left_through_the_popup_leaves_no_focus_ring(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);

    click(cx, "revset-summary");
    click(cx, "revset-presets");
    click(cx, "revset-chip-all");
    settle_visual(cx);
    view.read_with(cx, |view, _| {
        assert!(view.revset_editor_text().is_none());
        assert_eq!(view.focused_control(), None);
    });
}

#[gpui::test]
fn revset_editor_completes_the_symbol_being_typed(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);
    let retype = |cx: &mut VisualTestContext, text: &str| {
        click(cx, "revset-summary");
        cx.simulate_keystrokes("cmd-a");
        cx.simulate_input(text);
        settle_visual(cx);
    };

    retype(cx, "@ | main");
    assert!(
        cx.debug_bounds("revset-completion-0").is_some(),
        "the bookmark itself is offered"
    );
    cx.simulate_keystrokes("enter");
    settle_visual(cx);
    view.read_with(cx, |view, cx| {
        assert_eq!(
            view.view_model().read(cx).revset(),
            "@ | main",
            "Return applies the revset while no row is picked"
        );
        assert!(view.revset_editor_text().is_none());
    });
    assert!(cx.debug_bounds("revset-completions").is_none());

    retype(cx, "min");
    let bar = cx.debug_bounds("revset-bar").expect("revset bar");
    let list = cx.debug_bounds("revset-completions").expect("list");
    assert_eq!(
        (list.origin.x, list.size.width),
        (bar.origin.x, bar.size.width),
        "the list sits under the bar, edge to edge"
    );
    cx.simulate_keystrokes("down enter");
    settle_visual(cx);
    assert_eq!(
        view.read_with(cx, |view, _| view.revset_editor_text()),
        Some("mine()".to_owned()),
        "Down picks a row and Return inserts it"
    );
    assert!(cx.debug_bounds("revset-completions").is_none());
    cx.simulate_keystrokes("enter");
    settle_visual(cx);
    view.read_with(cx, |view, cx| {
        assert_eq!(view.view_model().read(cx).revset(), "mine()");
    });

    retype(cx, "au");
    cx.simulate_keystrokes("down down enter");
    settle_visual(cx);
    assert_eq!(
        view.read_with(cx, |view, _| view.revset_editor_text()),
        Some("author_date(".to_owned())
    );

    cx.simulate_keystrokes("cmd-a");
    cx.simulate_input("au");
    cx.simulate_keystrokes("escape");
    settle_visual(cx);
    assert!(cx.debug_bounds("revset-completions").is_none());
    assert_eq!(
        view.read_with(cx, |view, _| view.revset_editor_text()),
        Some("au".to_owned()),
        "Escape closes the list before the editor"
    );
    cx.simulate_keystrokes("escape");
    settle_visual(cx);
    assert!(view.read_with(cx, |view, _| view.revset_editor_text().is_none()));
}

#[gpui::test]
fn revset_popup_lists_completions_above_its_own_rows(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);

    click(cx, "revset-presets");
    let bar = cx.debug_bounds("revset-bar").expect("revset bar");
    let popup = cx.debug_bounds("revset-popup").expect("popup");
    assert_eq!(
        (popup.origin.x, popup.size.width),
        (bar.origin.x, bar.size.width),
        "the popup sits under the bar, edge to edge"
    );
    cx.simulate_input("ma");
    settle_visual(cx);
    assert!(
        cx.debug_bounds("revset-popup-completions").is_none(),
        "a bookmark typed alone is left to the popup's Bookmarks rows"
    );
    assert!(cx.debug_bounds("revset-popup-row-Bookmark-main").is_some());

    cx.simulate_keystrokes("cmd-a");
    cx.simulate_input("@ | mai");
    settle_visual(cx);
    assert!(
        cx.debug_bounds("revset-completion-0").is_some(),
        "inside an expression the Bookmarks rows match nothing, so the bookmark completes"
    );

    cx.simulate_keystrokes("cmd-a");
    cx.simulate_input("@ | min");
    settle_visual(cx);
    assert!(cx.debug_bounds("revset-popup-completions").is_some());
    cx.simulate_keystrokes("down enter");
    settle_visual(cx);
    assert!(
        view.read_with(cx, |view, _| view.revset_popup_open()),
        "a completion fills the field instead of applying"
    );
    cx.simulate_keystrokes("enter");
    settle_visual(cx);
    view.read_with(cx, |view, cx| {
        assert_eq!(view.view_model().read(cx).revset(), "@ | mine()");
        assert!(!view.revset_popup_open());
    });
}
