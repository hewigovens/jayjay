use crate::harness::{
    drag_handle, install_test_globals, keys, load_selected_change_files, settle, settle_slide,
    settle_visual,
};
use gpui::{AppContext, Entity, Modifiers, TestAppContext, VisualContext, VisualTestContext, px};
use jayjay_core::{DEFAULT_REVSET_DEPTH, build_default_revset};
use jayjay_gpui::repo::RepoWindow;
use jayjay_gpui::ui::context_menu::ContextAction;
use jj_test::{LinearFixture, current_op_id, run_jj_in};

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
fn change_list_divergent_badge_filters_to_divergent_changes(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let base_op = current_op_id(&fixture.path);
    run_jj_in(&fixture.path, &["describe", "-m", "left"]);
    run_jj_in(
        &fixture.path,
        &["--at-op", &base_op, "describe", "-m", "right"],
    );
    run_jj_in(&fixture.path, &["status"]);
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);

    assert!(cx.debug_bounds("sidebar-conflicts").is_none());
    click(cx, "sidebar-divergent");
    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert_eq!(vm.revset(), "divergent()");
        assert_eq!(vm.graph.changes.len(), 2);
        assert!(vm.graph.changes.iter().all(|change| change.is_divergent));
    });
}

#[gpui::test]
fn column_toggles_hold_still_while_sync_controls_leave_with_the_sidebar(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);

    let toggle = cx
        .debug_bounds("toolbar-sidebar-toggle")
        .expect("sidebar toggle");
    let sync_cluster = cx.debug_bounds("toolbar-sync-cluster").expect("sync group");
    let log_toggle = cx
        .debug_bounds("toolbar-operation-log")
        .expect("operation log toggle");
    assert!(toggle.origin.x + toggle.size.width < sync_cluster.origin.x);
    let title = cx.debug_bounds("repo-switcher-button").expect("title");
    let header_title = cx
        .debug_bounds("description-title")
        .or_else(|| cx.debug_bounds("description-empty"))
        .expect("change header title");
    assert_eq!(title.origin.x + px(10.), header_title.origin.x);

    view.update_in(cx, |view, _, cx| view.toggle_sidebar(cx));
    settle_slide(cx);
    assert!(cx.debug_bounds("toolbar-refresh").is_none());
    let hidden = cx
        .debug_bounds("toolbar-sidebar-toggle")
        .expect("sidebar toggle");
    assert_eq!(hidden.origin, toggle.origin);
    let title = cx.debug_bounds("repo-switcher-button").expect("title");
    assert!(title.origin.x > hidden.origin.x + hidden.size.width);

    click(cx, "toolbar-operation-log");
    settle_slide(cx);
    assert!(cx.debug_bounds("operation-log-pane").is_some());
    let open = cx
        .debug_bounds("toolbar-operation-log")
        .expect("operation log toggle");
    assert_eq!(open.origin, log_toggle.origin);

    drag_handle(cx, "operation-log-resize-handle", -40.);
    let dragged = cx
        .debug_bounds("toolbar-operation-log")
        .expect("operation log toggle");
    assert_eq!(dragged.origin, log_toggle.origin);

    click(cx, "toolbar-operation-log");
    settle_slide(cx);
    let closed = cx
        .debug_bounds("toolbar-operation-log")
        .expect("operation log toggle");
    assert_eq!(closed.origin, log_toggle.origin);
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

    cx.simulate_keystrokes("secondary-a");
    cx.simulate_input("@ | trunk()");
    keys(cx, "enter");
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

    click(cx, "revset-presets");
    click(cx, "revset-popup-forget-@ | trunk()");
    assert!(view.read_with(cx, |view, _| view.revset_popup_open()));
    assert!(
        cx.debug_bounds("revset-popup-row-Recent-@ | trunk()")
            .is_none()
    );
    view.read_with(cx, |view, cx| {
        assert!(view.view_model().read(cx).revset_filter.recent.is_empty());
    });
}

#[gpui::test]
fn recent_revsets_reach_a_new_window_of_the_repository(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    install_test_globals(cx);
    {
        let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
        settle_visual(cx);
        view.update(cx, |view, cx| {
            view.view_model()
                .update(cx, |vm, cx| vm.apply_revset("@ | trunk()", cx));
        });
        settle(cx);
    }

    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    settle_visual(cx);
    view.read_with(cx, |view, cx| {
        assert_eq!(
            view.view_model().read(cx).revset_filter.recent,
            ["@ | trunk()"]
        );
    });
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

    keys(cx, &format!("{}-l", jayjay_gpui::platform::MOD_KEY));
    cx.simulate_input("all()");
    keys(cx, "escape");
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
    keys(cx, "enter");
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
    cx.simulate_keystrokes("secondary-a");
    cx.simulate_input("invalid(");
    keys(cx, "enter");
    assert!(cx.debug_bounds("revset-popup-error").is_some());
    assert!(view.read_with(cx, |view, _| view.revset_popup_open()));
    assert_eq!(graph_commit_ids(&view, cx), before);

    keys(cx, "escape");
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
            .find(|item| item.label.as_ref() == "Show Ancestors…")
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
        view.dispatch_context_action(ContextAction::FilterRevset("trunk()".into()), cx);
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
        cx.simulate_keystrokes("secondary-a");
        cx.simulate_input(text);
        settle_visual(cx);
    };

    retype(cx, "@ | main");
    assert!(
        cx.debug_bounds("revset-completion-0").is_some(),
        "the bookmark itself is offered"
    );
    keys(cx, "enter");
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
    assert!(
        (list.origin.x - bar.origin.x).abs() <= px(0.5) && list.size.width == bar.size.width,
        "the list sits under the bar, edge to edge, at whole pixels"
    );
    keys(cx, "down enter");
    assert_eq!(
        view.read_with(cx, |view, _| view.revset_editor_text()),
        Some("mine()".to_owned()),
        "Down picks a row and Return inserts it"
    );
    assert!(cx.debug_bounds("revset-completions").is_none());
    keys(cx, "enter");
    view.read_with(cx, |view, cx| {
        assert_eq!(view.view_model().read(cx).revset(), "mine()");
    });

    retype(cx, "tag");
    keys(cx, "tab");
    assert_eq!(
        view.read_with(cx, |view, _| view.revset_editor_text()),
        Some("tags()".to_owned()),
        "Tab finishes the first row without leaving the field"
    );
    assert!(cx.debug_bounds("revset-completions").is_none());
    keys(cx, "enter");
    view.read_with(cx, |view, cx| {
        assert_eq!(view.view_model().read(cx).revset(), "tags()");
    });

    retype(cx, "au");
    keys(cx, "down down enter");
    assert_eq!(
        view.read_with(cx, |view, _| view.revset_editor_text()),
        Some("author_date(".to_owned())
    );

    cx.simulate_keystrokes("secondary-a");
    cx.simulate_input("au");
    keys(cx, "escape");
    assert!(cx.debug_bounds("revset-completions").is_none());
    assert_eq!(
        view.read_with(cx, |view, _| view.revset_editor_text()),
        Some("au".to_owned()),
        "Escape closes the list before the editor"
    );
    keys(cx, "escape");
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
    assert!(
        (popup.origin.x - bar.origin.x).abs() <= px(0.5) && popup.size.width == bar.size.width,
        "the popup sits under the bar, edge to edge, at whole pixels"
    );
    cx.simulate_input("ma");
    settle_visual(cx);
    assert!(
        cx.debug_bounds("revset-popup-completions").is_none(),
        "a bookmark typed alone is left to the popup's Bookmarks rows"
    );
    assert!(cx.debug_bounds("revset-popup-row-Bookmark-main").is_some());

    cx.simulate_keystrokes("secondary-a");
    cx.simulate_input("@ | mai");
    settle_visual(cx);
    assert!(
        cx.debug_bounds("revset-completion-0").is_some(),
        "inside an expression the Bookmarks rows match nothing, so the bookmark completes"
    );

    cx.simulate_keystrokes("secondary-a");
    cx.simulate_input("@ | min");
    settle_visual(cx);
    assert!(cx.debug_bounds("revset-popup-completions").is_some());
    keys(cx, "down enter");
    assert!(
        view.read_with(cx, |view, _| view.revset_popup_open()),
        "a completion fills the field instead of applying"
    );
    keys(cx, "enter");
    view.read_with(cx, |view, cx| {
        assert_eq!(view.view_model().read(cx).revset(), "@ | mine()");
        assert!(!view.revset_popup_open());
    });

    click(cx, "revset-presets");
    cx.simulate_input("@ | tag");
    settle_visual(cx);
    keys(cx, "tab");
    assert!(
        view.read_with(cx, |view, _| view.revset_popup_open()),
        "Tab fills the field instead of moving focus"
    );
    assert!(cx.debug_bounds("revset-popup-completions").is_none());
    keys(cx, "enter");
    view.read_with(cx, |view, cx| {
        assert_eq!(view.view_model().read(cx).revset(), "@ | tags()");
    });
}

#[gpui::test]
fn a_configured_log_revset_is_the_default_filter(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    jj_test::run_jj_in(
        &fixture.path,
        &["config", "set", "--repo", "revsets.log", "::@"],
    );
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    settle_visual(cx);
    let revset = |cx: &VisualTestContext| {
        view.read_with(cx, |view, cx| {
            view.view_model().read(cx).revset().to_owned()
        })
    };
    assert_eq!(revset(cx), "::@");

    view.update(cx, |view, cx| {
        view.view_model()
            .update(cx, |vm, cx| vm.apply_revset("trunk()", cx));
    });
    settle(cx);
    view.update(cx, |view, cx| {
        view.view_model()
            .update(cx, |vm, cx| vm.apply_revset(" ", cx));
    });
    settle(cx);
    assert_eq!(revset(cx), "::@");
}
