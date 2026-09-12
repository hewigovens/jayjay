use crate::harness::{open_fixture, settle_visual};
use gpui::{Modifiers, TestAppContext};
use jayjay_gpui::repo::ActivePane;
use jj_test::LinearFixture;

#[gpui::test]
fn file_column_filter_searches_paths_and_clears_when_closed(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    fixture.add_tracked_working_copy_edits();
    let (view, cx) = open_fixture(&fixture, cx);

    let all_paths = view.read_with(cx, |view, cx| view.visible_file_paths(cx));
    assert!(all_paths.len() > 1, "fixture should expose multiple files");

    let toggle = cx
        .debug_bounds("toggle-file-filter")
        .expect("file filter button");
    cx.simulate_click(toggle.center(), Modifiers::default());
    settle_visual(cx);
    assert!(cx.debug_bounds("file-filter-bar").is_some());

    let input = cx
        .debug_bounds("file-filter-input")
        .expect("file filter input");
    cx.simulate_click(input.center(), Modifiers::default());
    cx.simulate_input("READme");
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        assert!(view.file_filter_visible());
        assert_eq!(view.file_filter_query(), Some("READme"));
        assert_eq!(view.visible_file_paths(cx), ["README.md"]);
    });

    let close = cx
        .debug_bounds("file-filter-close")
        .expect("file filter close button");
    cx.simulate_click(close.center(), Modifiers::default());
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        assert!(!view.file_filter_visible());
        assert_eq!(view.visible_file_paths(cx), all_paths);
    });
}

#[gpui::test]
fn return_keeps_the_filter_and_escape_closes_it_back_to_the_file_list(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    fixture.add_tracked_working_copy_edits();
    let (view, cx) = open_fixture(&fixture, cx);

    let toggle = cx
        .debug_bounds("toggle-file-filter")
        .expect("file filter button");
    cx.simulate_click(toggle.center(), Modifiers::default());
    settle_visual(cx);
    cx.simulate_input("READme");
    settle_visual(cx);

    cx.simulate_keystrokes("enter");
    settle_visual(cx);
    view.read_with(cx, |view, cx| {
        assert_eq!(view.file_filter_query(), Some("READme"));
        assert_eq!(view.visible_file_paths(cx), ["README.md"]);
        assert_eq!(view.active_pane(), ActivePane::FileColumn);
    });

    cx.simulate_keystrokes("escape");
    settle_visual(cx);
    view.read_with(cx, |view, cx| {
        assert!(!view.file_filter_visible());
        assert!(view.visible_file_paths(cx).len() > 1);
        assert_eq!(view.active_pane(), ActivePane::FileColumn);
    });
}
