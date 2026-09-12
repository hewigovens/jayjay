use crate::harness::*;
use gpui::{Entity, Focusable, Modifiers, TestAppContext, VisualContext, VisualTestContext};
use jayjay_gpui::app::config;
use jayjay_gpui::repo::{ActivePane, FocusStop, RepoWindow};
use jj_test::LinearFixture;

fn tree_mode(cx: &mut VisualTestContext) -> bool {
    cx.cx.read(|cx| config::current(cx).diff.tree_file_list)
}

/// `attach_to_window` does this when the app opens a repo window; `add_window_view` does not.
fn open_focused<'a>(
    fixture: &LinearFixture,
    cx: &'a mut TestAppContext,
) -> (Entity<RepoWindow>, &'a mut VisualTestContext) {
    let (view, cx) = open_fixture(fixture, cx);
    view.update_in(cx, |view, window, cx| {
        view.focus_handle(cx).focus(window, cx)
    });
    (view, cx)
}

#[gpui::test]
fn tab_enters_the_file_list_then_lands_on_its_toggles(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_focused(&fixture, cx);
    view.update(cx, |view, cx| {
        view.view_model().update(cx, |vm, _| {
            vm.selected_file_ix = None;
        });
    });

    cx.simulate_keystrokes("tab");
    settle_visual(cx);
    view.read_with(cx, |view, cx| {
        assert_eq!(view.active_pane(), ActivePane::FileColumn);
        assert_eq!(view.focused_control(), None);
        assert_eq!(view.view_model().read(cx).selected_file_ix, Some(0));
    });

    cx.simulate_keystrokes("tab");
    settle_visual(cx);
    view.read_with(cx, |view, _| {
        assert_eq!(view.focused_control(), Some(FocusStop::TreeToggle));
    });

    let was_tree_mode = tree_mode(cx);
    cx.simulate_keystrokes("space");
    settle_visual(cx);
    assert_ne!(tree_mode(cx), was_tree_mode);

    cx.simulate_keystrokes("escape");
    settle_visual(cx);
    view.read_with(cx, |view, _| {
        assert_eq!(view.focused_control(), None);
        assert_eq!(view.active_pane(), ActivePane::FileColumn);
    });

    cx.simulate_keystrokes("shift-tab");
    settle_visual(cx);
    view.read_with(cx, |view, _| {
        assert_eq!(view.active_pane(), ActivePane::Sidebar);
        assert_eq!(view.focused_control(), None);
    });
}

#[gpui::test]
fn commit_box_stops_take_text_focus_and_tab_still_leaves_them(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_focused(&fixture, cx);

    cx.simulate_keystrokes("shift-tab");
    settle_visual(cx);
    view.read_with(cx, |view, _| {
        assert_eq!(view.focused_control(), Some(FocusStop::CommitDescription));
    });
    cx.simulate_input("body");
    cx.simulate_keystrokes("shift-tab");
    settle_visual(cx);
    view.read_with(cx, |view, _| {
        assert_eq!(view.focused_control(), Some(FocusStop::CommitSummary));
    });
    cx.simulate_input("summary");
    settle_visual(cx);
    view.read_with(cx, |view, cx| {
        assert_eq!(view.summary_input().read(cx).text(), "summary");
        assert_eq!(view.description_input().read(cx).text(), "body");
    });

    cx.simulate_keystrokes("tab tab");
    settle_visual(cx);
    view.read_with(cx, |view, _| {
        assert_eq!(view.focused_control(), None);
        assert_eq!(view.active_pane(), ActivePane::Sidebar);
    });
}

#[gpui::test]
fn find_keeps_its_keys_after_a_control_had_focus(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_focused(&fixture, cx);
    cx.simulate_keystrokes("tab tab");
    settle_visual(cx);
    let was_tree_mode = tree_mode(cx);
    view.update(cx, |view, cx| view.open_find(cx));
    cx.simulate_keystrokes("tab space");
    settle_visual(cx);
    assert_eq!(tree_mode(cx), was_tree_mode);
    view.read_with(cx, |view, _| {
        assert_eq!(view.find_query_text(), Some(" "));
        assert_eq!(view.focused_control(), None);
    });
    cx.simulate_keystrokes("escape");
    settle_visual(cx);
    view.read_with(cx, |view, _| assert!(view.find_query_text().is_none()));
}

#[gpui::test]
fn row_clicks_and_diff_edit_release_the_focused_control(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_focused(&fixture, cx);

    cx.simulate_keystrokes("tab tab");
    settle_visual(cx);
    view.read_with(cx, |view, _| {
        assert_eq!(view.focused_control(), Some(FocusStop::TreeToggle));
    });

    let row = cx.debug_bounds("file-row-0").expect("first file row");
    cx.simulate_click(row.center(), Modifiers::default());
    settle_visual(cx);
    view.read_with(cx, |view, _| {
        assert_eq!(view.focused_control(), None);
        assert_eq!(view.active_pane(), ActivePane::FileColumn);
    });

    cx.simulate_keystrokes("tab");
    settle_visual(cx);
    view.update_in(cx, |view, _, cx| view.enter_diff_edit(cx));
    settle_visual(cx);
    view.read_with(cx, |view, _| assert_eq!(view.focused_control(), None));

    cx.simulate_keystrokes("tab");
    settle_visual(cx);
    view.read_with(cx, |view, _| assert_eq!(view.focused_control(), None));
}

#[gpui::test]
fn text_focus_overrides_the_previous_tab_stop(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_focused(&fixture, cx);

    cx.simulate_keystrokes("tab tab");
    settle_visual(cx);
    let was_tree_mode = tree_mode(cx);
    let summary = view.read_with(cx, |view, _| view.summary_input().clone());
    cx.focus(&summary);
    cx.simulate_keystrokes("space");
    settle_visual(cx);
    assert_eq!(tree_mode(cx), was_tree_mode);
    cx.simulate_keystrokes("tab");
    settle_visual(cx);
    view.read_with(cx, |view, cx| {
        assert_eq!(view.focused_control(), Some(FocusStop::CommitDescription));
        assert_eq!(view.summary_input().read(cx).text(), " ");
    });

    cx.focus(&summary);
    cx.simulate_keystrokes("tab");
    settle_visual(cx);
    view.read_with(cx, |view, _| {
        assert_eq!(view.focused_control(), Some(FocusStop::CommitDescription));
    });
}

#[gpui::test]
fn keyboard_opened_filters_submit_and_release_text_focus(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_focused(&fixture, cx);

    for stop in [FocusStop::FilterToggle, FocusStop::RevsetFilter] {
        for _ in 0..18 {
            if view.read_with(cx, |view, _| view.focused_control()) == Some(stop) {
                break;
            }
            cx.simulate_keystrokes("tab");
            settle_visual(cx);
        }
        assert_eq!(
            view.read_with(cx, |view, _| view.focused_control()),
            Some(stop)
        );
        cx.simulate_keystrokes("space enter");
        settle_visual(cx);
        view.read_with(cx, |view, _| {
            assert_eq!(view.focused_control(), None);
            match stop {
                FocusStop::FilterToggle => assert!(view.file_filter_visible()),
                FocusStop::RevsetFilter => assert!(view.revset_filter_text().is_some()),
                _ => unreachable!(),
            }
        });
        cx.simulate_keystrokes("escape");
        settle_visual(cx);
        view.read_with(cx, |view, _| {
            assert!(!view.file_filter_visible());
            assert!(view.revset_filter_text().is_none());
        });
    }
}
