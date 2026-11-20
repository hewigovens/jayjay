use crate::harness::*;
use gpui::{TestAppContext, px, size};
use jayjay_core::shortcuts::ShortcutCommand;
use jayjay_gpui::app::actions::RestoreSelectedOperation;
use jayjay_gpui::app::config;
use jj_test::{LinearFixture, run_jj_in};

#[gpui::test]
fn the_operation_log_pane_toggles_from_the_status_bar_and_fits_the_window(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (_view, cx) = open_fixture(&fixture, cx);
    cx.simulate_resize(size(px(1600.), px(900.)));
    settle_visual(cx);

    click(cx, "status-last-op");
    let description = cx
        .debug_bounds("operation-log-current-description")
        .expect("current operation description bounds");
    assert!(
        description.size.width > px(20.),
        "operation description collapsed to {:?}",
        description.size.width
    );
    let badge = cx
        .debug_bounds("operation-log-current-badge")
        .expect("current badge bounds");
    let badge_gap = badge.origin.x - (description.origin.x + description.size.width);
    assert!(
        badge_gap >= px(0.) && badge_gap <= px(10.),
        "current badge should follow description closely, got gap {badge_gap:?}"
    );

    drag_handle(cx, "operation-log-resize-handle", -400.);
    assert_eq!(pane_width(cx, "operation-log-pane"), 560.);
    cx.simulate_resize(size(px(1200.), px(720.)));
    settle_visual(cx);
    let pane = cx.debug_bounds("operation-log-pane").expect("pane bounds");
    assert!(pane.size.width < px(560.) && pane.origin.x + pane.size.width <= px(1200.));

    click(cx, "status-last-op");
    assert!(cx.debug_bounds("operation-log-pane").is_none());
}

#[gpui::test]
fn the_selected_operation_restores_from_its_row_menu(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_fixture(&fixture, cx);
    run_jj_in(&fixture.path, &["describe", "-m", "described elsewhere"]);
    view.update(cx, |view, cx| {
        view.view_model().update(cx, |vm, cx| vm.refresh(false, cx))
    });
    settle_visual(cx);
    let working_copy_description = |cx: &mut gpui::VisualTestContext| {
        view.read_with(cx, |view, cx| {
            view.view_model()
                .read(cx)
                .graph
                .changes
                .iter()
                .find(|change| change.is_working_copy)
                .expect("working copy")
                .description
                .trim()
                .to_owned()
        })
    };
    assert_eq!(working_copy_description(cx), "described elsewhere");

    click(cx, "toolbar-operation-log");
    keys(cx, "ctrl-enter");
    click(
        cx,
        selector(format!("context-menu-{}", jayjay_i18n::repo::restore())),
    );

    assert_ne!(working_copy_description(cx), "described elsewhere");
    assert_eq!(
        view.read_with(cx, |view, _| view.toast()).as_deref(),
        Some(jayjay_i18n::oplog::restored_operation().as_str())
    );
}

#[gpui::test]
fn a_refresh_whose_revset_broke_still_reports_the_operation_that_broke_it(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    run_jj_in(&fixture.path, &["bookmark", "create", "doomed", "-r", "@"]);
    let (view, cx) = open_fixture(&fixture, cx);
    let refresh = |cx: &mut gpui::VisualTestContext, revset: Option<&str>| {
        view.update(cx, |view, cx| {
            view.view_model().update(cx, |vm, cx| match revset {
                Some(revset) => vm.apply_revset(revset, cx),
                None => vm.refresh(false, cx),
            })
        });
        settle_visual(cx);
    };
    refresh(cx, Some("doomed"));

    run_jj_in(&fixture.path, &["bookmark", "delete", "doomed"]);
    refresh(cx, None);

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert!(vm.error.is_some(), "the revset should no longer resolve");
        assert!(
            vm.current_operation_description.contains("delete bookmark"),
            "{}",
            vm.current_operation_description
        );
    });
}

#[gpui::test]
fn a_second_restore_waits_for_the_first(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_fixture(&fixture, cx);
    run_jj_in(&fixture.path, &["describe", "-m", "described elsewhere"]);
    view.update(cx, |view, cx| {
        view.view_model().update(cx, |vm, cx| vm.refresh(false, cx))
    });
    settle_visual(cx);
    click(cx, "toolbar-operation-log");

    cx.update(|window, cx| {
        window.dispatch_action(Box::new(RestoreSelectedOperation), cx);
        window.dispatch_action(Box::new(RestoreSelectedOperation), cx);
    });
    settle_visual(cx);

    let output = run_jj_in(
        &fixture.path,
        &["op", "log", "--no-graph", "-T", "description ++ \"\\n\""],
    );
    let restores = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| line.starts_with("restore"))
        .count();
    assert_eq!(restores, 1);
}

#[gpui::test]
fn a_pane_without_room_restores_nothing(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_fixture(&fixture, cx);
    cx.simulate_resize(size(px(900.), px(720.)));
    run_jj_in(&fixture.path, &["describe", "-m", "described elsewhere"]);
    view.update(cx, |view, cx| {
        view.view_model().update(cx, |vm, cx| vm.refresh(false, cx))
    });
    settle_visual(cx);
    click(cx, "toolbar-operation-log");
    assert!(cx.debug_bounds("operation-log-pane").is_none());

    cx.update(|window, cx| window.dispatch_action(Box::new(RestoreSelectedOperation), cx));
    settle_visual(cx);

    let output = run_jj_in(
        &fixture.path,
        &["op", "log", "--no-graph", "-T", "description ++ \"\\n\""],
    );
    assert!(
        !String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line.starts_with("restore"))
    );
}

#[gpui::test]
fn an_operation_row_menu_opens_at_the_pointer(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (_view, cx) = open_fixture(&fixture, cx);
    click(cx, "toolbar-operation-log");
    let pane = cx.debug_bounds("operation-log-pane").expect("pane");
    let restore = selector(format!("context-menu-{}", jayjay_i18n::repo::restore()));
    let menu_at = |cx: &mut gpui::VisualTestContext, x: f32| {
        let at = gpui::point(pane.origin.x + px(x), pane.origin.y + px(20.));
        cx.simulate_mouse_down(at, gpui::MouseButton::Right, gpui::Modifiers::default());
        settle_visual(cx);
        let item = cx.debug_bounds(restore).expect("restore item");
        keys(cx, "escape");
        (at, item)
    };
    let (near, near_item) = menu_at(cx, 20.);
    let (far, far_item) = menu_at(cx, 160.);

    for (at, item) in [(near, near_item), (far, far_item)] {
        let edge = |x: gpui::Pixels| (x - at.x).abs() <= px(2.);
        assert!(
            edge(item.origin.x) || edge(item.origin.x + item.size.width),
            "the menu opens from the pointer at {at:?}, not at {item:?}"
        );
        assert!(item.origin.y > at.y && item.origin.y - at.y < px(12.));
    }
    assert_ne!(near_item.origin.x, far_item.origin.x);
}

#[gpui::test]
fn the_diff_header_compacts_for_the_room_the_pane_leaves(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_fixture(&fixture, cx);
    cx.simulate_resize(size(px(1400.), px(900.)));
    view.update(cx, |view, cx| view.select_file(0, cx));
    settle_visual(cx);
    let labelled = pane_width(cx, "mark-reviewed");

    click(cx, "toolbar-operation-log");
    assert!(cx.debug_bounds("operation-log-pane").is_some());
    assert!(
        pane_width(cx, "mark-reviewed") < labelled,
        "Mark Reviewed drops its label once the pane narrows the diff"
    );
}

#[gpui::test]
fn closing_the_focused_pane_returns_keys_to_the_changes_list(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (_view, cx) = open_focused(&fixture, cx);
    cx.cx.update(|cx| {
        config::update(cx, |cfg| {
            cfg.shortcuts.insert(
                ShortcutCommand::ChangesAbandon.id().to_owned(),
                vec!["f8".to_owned()],
            );
        })
    });
    cx.simulate_resize(size(px(1600.), px(900.)));
    settle_visual(cx);
    for close in ["shortcut", "room"] {
        keys(cx, "secondary-shift-u");
        assert!(cx.debug_bounds("operation-log-pane").is_some());
        match close {
            "shortcut" => keys(cx, "secondary-shift-u"),
            _ => {
                cx.simulate_resize(size(px(900.), px(720.)));
                settle_visual(cx);
            }
        }
        assert!(cx.debug_bounds("operation-log-pane").is_none());
        keys(cx, "f8");
        assert!(
            cx.debug_bounds("confirmation-submit").is_some(),
            "the changes list takes its keys again once the pane is closed by {close}"
        );
        keys(cx, "escape");
        cx.simulate_resize(size(px(1600.), px(900.)));
        settle_visual(cx);
    }
}

#[gpui::test]
fn showing_the_pane_without_room_keeps_it_wanted_and_says_why(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_focused(&fixture, cx);
    cx.simulate_resize(size(px(900.), px(720.)));
    settle_visual(cx);
    #[cfg(not(target_os = "macos"))]
    cx.cx.update(jayjay_gpui::app::menus::install);
    let label = |cx: &mut gpui::VisualTestContext| {
        let title = view.read_with(cx, |view, _| view.operation_log_toggle_title());
        #[cfg(not(target_os = "macos"))]
        {
            click(
                cx,
                selector(format!("app-menu-bar-{}", jayjay_i18n::menu::view())),
            );
            assert!(
                cx.debug_bounds(selector(format!("app-menu-item-{title}")))
                    .is_some(),
                "the View menu says {title} too"
            );
            keys(cx, "escape");
        }
        title
    };
    assert_eq!(label(cx), jayjay_i18n::menu::show_operation_log());
    for _ in 0..2 {
        click(cx, "toolbar-operation-log");
        assert!(cx.debug_bounds("operation-log-pane").is_none());
        assert_eq!(label(cx), jayjay_i18n::menu::show_operation_log());
        assert_eq!(
            view.read_with(cx, |view, _| view.toast()).as_deref(),
            Some(jayjay_i18n::oplog::no_room_for_operation_log().as_str())
        );
    }
    cx.simulate_resize(size(px(1600.), px(900.)));
    settle_visual(cx);
    assert!(cx.debug_bounds("operation-log-pane").is_some());
    assert_eq!(label(cx), jayjay_i18n::menu::hide_operation_log());
}
