use crate::harness::{install_test_globals, load_selected_change_files, settle_visual};
use gpui::{Modifiers, MouseButton, TestAppContext, VisualTestContext, point, px, size};
use jayjay_gpui::repo::RepoWindow;
use jj_test::LinearFixture;

#[gpui::test]
fn file_column_width_survives_a_new_repo_window(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| {
        let mut view = RepoWindow::new(fixture.path.clone(), cx);
        view.boot(cx);
        view
    });
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);

    let initial_width = pane_width(cx, "file-column-header");
    drag(cx, "file-column-resize-handle", 80.);

    let resized_width = pane_width(cx, "file-column-header");
    assert!(
        resized_width > initial_width + 70.,
        "drag should widen the file column: {initial_width} -> {resized_width}"
    );

    let (restored_view, restored_cx) = cx.cx.add_window_view(|_, cx| {
        let mut view = RepoWindow::new(fixture.path.clone(), cx);
        view.boot(cx);
        view
    });
    let restored_cx: &mut VisualTestContext = restored_cx;
    load_selected_change_files(&restored_view, restored_cx);
    settle_visual(restored_cx);

    let restored_width = pane_width(restored_cx, "file-column-header");
    assert!(
        (restored_width - resized_width).abs() < 1.,
        "new window should restore the file column: {resized_width} vs {restored_width}"
    );
}

#[gpui::test]
fn panes_fit_a_narrow_window_and_keep_the_detail(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| {
        let mut view = RepoWindow::new(fixture.path.clone(), cx);
        view.boot(cx);
        view
    });
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);

    drag(cx, "sidebar-resize-handle", 400.);
    drag(cx, "file-column-resize-handle", 300.);
    assert_eq!(pane_width(cx, "file-column-header"), 480.);

    cx.simulate_resize(size(px(1080.), px(720.)));
    settle_visual(cx);
    assert_eq!(pane_width(cx, "file-column-header"), 220.);
    let detail_width = pane_width(cx, "detail-pane");
    assert!(
        (detail_width - 420.).abs() < 2.,
        "the sidebar should shrink until the preview keeps its minimum: {detail_width}"
    );

    drag(cx, "file-column-resize-handle", 200.);
    assert_eq!(pane_width(cx, "file-column-header"), 220.);
}

fn drag(cx: &mut VisualTestContext, handle: &'static str, dx: f32) {
    let start = cx.debug_bounds(handle).expect(handle).center();
    let end = point(start.x + px(dx), start.y);
    cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::default());
    cx.simulate_mouse_move(end, MouseButton::Left, Modifiers::default());
    cx.simulate_mouse_up(end, MouseButton::Left, Modifiers::default());
    settle_visual(cx);
}

fn pane_width(cx: &mut VisualTestContext, selector: &'static str) -> f32 {
    f32::from(cx.debug_bounds(selector).expect(selector).size.width)
}
