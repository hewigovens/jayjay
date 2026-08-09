mod harness;

use gpui::{Modifiers, TestAppContext, VisualTestContext};
use harness::{install_test_globals, load_selected_change_files, settle_visual};
use jayjay_gpui::repo::RepoWindow;
use jj_test::LinearFixture;

#[gpui::test]
fn file_column_filter_searches_paths_and_clears_when_closed(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    fixture.add_tracked_working_copy_edits();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);

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
