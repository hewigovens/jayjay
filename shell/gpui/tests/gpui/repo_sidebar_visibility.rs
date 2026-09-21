use crate::harness::*;
use gpui::{Focusable, Modifiers, TestAppContext, VisualTestContext};
use jayjay_gpui::repo::{ActivePane, FocusStop, RepoWindow};
use jayjay_gpui::ui::context_menu::ContextAction;
use jj_test::LinearFixture;

fn sidebar_visible(cx: &mut VisualTestContext) -> bool {
    cx.debug_bounds("sidebar-resize-handle").is_some()
}

fn open_focused<'a>(
    fixture: &LinearFixture,
    cx: &'a mut TestAppContext,
) -> (gpui::Entity<RepoWindow>, &'a mut VisualTestContext) {
    let (view, cx) = open_fixture(fixture, cx);
    view.update_in(cx, |view, window, cx| {
        view.focus_handle(cx).focus(window, cx)
    });
    (view, cx)
}

fn open_booted<'a>(
    fixture: &LinearFixture,
    cx: &'a mut TestAppContext,
) -> (gpui::Entity<RepoWindow>, &'a mut VisualTestContext) {
    let (view, cx) = cx.add_window_view(|_, cx| {
        let mut view = RepoWindow::new(fixture.path.clone(), cx);
        view.boot(cx);
        view
    });
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);
    (view, cx)
}

#[gpui::test]
fn hide_reclaims_width_and_show_restores_it_exactly(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    install_test_globals(cx);
    let (view, cx) = open_booted(&fixture, cx);

    let change_count = view.read_with(cx, |view, cx| {
        view.view_model().read(cx).graph.changes.len()
    });
    let header = selector(format!("sidebar-changes-header-{change_count}"));

    drag_handle(cx, "sidebar-resize-handle", 60.);
    let sidebar_width = pane_width(cx, header);
    let detail_before = pane_width(cx, "detail-pane");

    view.update_in(cx, |view, _, cx| view.select_change(1, cx));
    view.update_in(cx, |view, _, cx| view.toggle_sidebar(cx));
    settle_slide(cx);

    assert!(!sidebar_visible(cx));
    assert!(cx.debug_bounds(header).is_none());
    let detail_hidden = pane_width(cx, "detail-pane");
    assert!(
        detail_hidden > detail_before + sidebar_width - 1.,
        "the detail pane should reclaim the hidden sidebar's width: {detail_before} -> {detail_hidden}"
    );
    assert!(
        view.read_with(cx, |view, _| view.sidebar_hidden()),
        "runtime state follows the toggle"
    );

    view.update_in(cx, |view, _, cx| view.toggle_sidebar(cx));
    settle_slide(cx);

    assert!(sidebar_visible(cx));
    let restored = pane_width(cx, header);
    assert!(
        (restored - sidebar_width).abs() < 1.,
        "show restores the dragged width exactly: {sidebar_width} vs {restored}"
    );
    view.read_with(cx, |view, cx| {
        assert_eq!(
            view.view_model().read(cx).selected,
            Some(1),
            "selection survives a hide/show cycle"
        );
    });
}

#[gpui::test]
fn hidden_sidebar_persists_into_a_recreated_window(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    install_test_globals(cx);
    let (view, cx) = open_booted(&fixture, cx);

    view.update_in(cx, |view, _, cx| view.toggle_sidebar(cx));
    settle_slide(cx);
    assert!(view.read_with(cx, |view, _| view.sidebar_hidden()));

    let (reopened, reopened_cx) = open_booted(&fixture, &mut cx.cx);

    assert!(!sidebar_visible(reopened_cx));
    reopened.read_with(reopened_cx, |view, _| {
        assert!(view.sidebar_hidden());
        assert_eq!(view.active_pane(), ActivePane::FileColumn);
    });
}

#[gpui::test]
fn tab_cycle_skips_sidebar_stops_when_hidden_and_toggle_is_always_present(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_focused(&fixture, cx);

    view.update_in(cx, |view, _, cx| view.toggle_sidebar(cx));
    settle_slide(cx);

    let mut stops = Vec::new();
    for _ in 0..14 {
        cx.simulate_keystrokes("tab");
        settle_visual(cx);
        stops.push(view.read_with(cx, |view, _| (view.active_pane(), view.focused_control())));
    }
    assert!(
        stops
            .iter()
            .any(|(_, control)| *control == Some(FocusStop::SidebarToggle)),
        "the sidebar toggle stays in the cycle: {stops:?}"
    );
    assert!(
        stops
            .iter()
            .all(|(pane, control)| !(*pane == ActivePane::Sidebar && control.is_none())),
        "the hidden DAG never becomes a stop: {stops:?}"
    );
    assert!(
        stops.iter().all(|(_, control)| !matches!(
            control,
            Some(FocusStop::RevsetInput | FocusStop::CommitSummary | FocusStop::CommitDescription)
        )),
        "sidebar inputs leave the cycle while hidden: {stops:?}"
    );

    for _ in 0..14 {
        if view.read_with(cx, |view, _| view.focused_control()) == Some(FocusStop::SidebarToggle) {
            break;
        }
        cx.simulate_keystrokes("tab");
        settle_visual(cx);
    }
    cx.simulate_keystrokes("space");
    settle_visual(cx);
    assert!(sidebar_visible(cx), "Space on the toggle shows the sidebar");
}

#[gpui::test]
fn filter_button_reveals_hidden_sidebar_and_focuses_the_input(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_focused(&fixture, cx);

    let filter = cx
        .debug_bounds("toolbar-revset-filter")
        .expect("filter button");
    cx.simulate_click(filter.center(), Modifiers::default());
    settle_visual(cx);
    assert!(cx.debug_bounds("revset-filter").is_some());

    view.update_in(cx, |view, _, cx| view.toggle_sidebar(cx));
    settle_slide(cx);
    assert!(!sidebar_visible(cx));
    assert!(
        view.read_with(cx, |view, _| view.revset_filter_text().is_none()),
        "hiding closes the open revset filter"
    );

    let filter = cx
        .debug_bounds("toolbar-revset-filter")
        .expect("filter button while hidden");
    cx.simulate_click(filter.center(), Modifiers::default());
    settle_visual(cx);
    assert!(sidebar_visible(cx), "Filter reveals a hidden sidebar");
    assert!(cx.debug_bounds("revset-filter").is_some());
    assert!(
        cx.debug_bounds("revset-filter-caret").is_some(),
        "the revealed filter input takes focus"
    );

    let filter = cx
        .debug_bounds("toolbar-revset-filter")
        .expect("filter button while visible");
    cx.simulate_click(filter.center(), Modifiers::default());
    settle_visual(cx);
    assert!(cx.debug_bounds("revset-filter").is_none());
    assert!(
        sidebar_visible(cx),
        "closing the filter leaves the sidebar shown"
    );
}

#[gpui::test]
fn revealing_actions_unhide_the_sidebar(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_focused(&fixture, cx);

    view.update_in(cx, |view, _, cx| view.toggle_sidebar(cx));
    settle_slide(cx);
    view.update_in(cx, |view, _, cx| {
        view.dispatch_context_action(ContextAction::FilterBookmarkRevset("all()".into()), cx);
    });
    settle_visual(cx);
    assert!(sidebar_visible(cx), "applying a revset reveals the sidebar");
    view.read_with(cx, |view, cx| {
        assert_eq!(view.view_model().read(cx).revset.as_ref(), "all()");
        assert_eq!(view.active_pane(), ActivePane::Sidebar);
    });

    view.update_in(cx, |view, _, cx| view.toggle_sidebar(cx));
    settle_slide(cx);
    let change_id = view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .graph
            .changes
            .last()
            .expect("a change to reveal")
            .change_id
            .id
            .clone()
    });
    view.update_in(cx, |view, _, cx| {
        view.dispatch_context_action(ContextAction::RevealChange(change_id.into()), cx);
    });
    settle_visual(cx);
    assert!(sidebar_visible(cx), "reveal_change_id reveals the sidebar");
    view.read_with(cx, |view, cx| {
        assert_eq!(
            view.view_model().read(cx).revset.as_ref(),
            "all()",
            "the applied revset survives hide/show cycles"
        );
    });
}

#[gpui::test]
fn sidebar_toggle_keybinding_hides_and_shows(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_focused(&fixture, cx);

    cx.simulate_keystrokes(jayjay_gpui::platform::SIDEBAR_TOGGLE_KEY);
    settle_slide(cx);
    assert!(!sidebar_visible(cx));

    cx.simulate_keystrokes(jayjay_gpui::platform::SIDEBAR_TOGGLE_KEY);
    settle_slide(cx);
    assert!(sidebar_visible(cx));
    assert!(!view.read_with(cx, |view, _| view.sidebar_hidden()));
}
