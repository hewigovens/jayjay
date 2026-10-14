use crate::harness::{
    drag_handle, install_test_globals, load_selected_change_files, open_fixture, pane_width,
    selector, settle_visual,
};
use gpui::{TestAppContext, VisualTestContext, px, size};
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
    drag_handle(cx, "file-column-resize-handle", 80.);

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

    drag_handle(cx, "sidebar-resize-handle", 400.);
    drag_handle(cx, "file-column-resize-handle", 300.);
    assert_eq!(pane_width(cx, "file-column-header"), 480.);

    cx.simulate_resize(size(px(1080.), px(720.)));
    settle_visual(cx);
    assert_eq!(pane_width(cx, "file-column-header"), 220.);
    let detail_width = pane_width(cx, "detail-pane");
    assert!(
        (detail_width - 420.).abs() < 2.,
        "the sidebar should shrink until the preview keeps its minimum: {detail_width}"
    );

    drag_handle(cx, "file-column-resize-handle", 200.);
    assert_eq!(pane_width(cx, "file-column-header"), 220.);
}

#[gpui::test]
fn dag_rows_keep_two_line_summaries_compact_without_clipping_metadata(cx: &mut TestAppContext) {
    use jayjay_gpui::app::config;
    use jj_test::run_jj_in;

    let fixture = LinearFixture::build();
    let description = "feat(theme): add built-in diff palettes with light and dark selections";
    run_jj_in(&fixture.path, &["describe", "-m", description]);
    let (view, cx) = open_fixture(&fixture, cx);
    cx.simulate_resize(size(px(1080.), px(720.)));
    settle_visual(cx);
    let (row_ix, row_selector) = view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        let row_ix = vm
            .graph
            .changes
            .iter()
            .position(|change| change.description.trim() == description)
            .unwrap();
        (
            row_ix,
            format!("dag-change-{}", vm.graph.changes[row_ix].commit_id.id),
        )
    });

    for (interface_size, code_size) in [(12., 12.), (18., 18.), (12., 18.)] {
        cx.cx.update(|cx| {
            config::update(cx, |config| {
                *config = toml::from_str(&format!(
                    "interface_font_size = {interface_size}\ncode_font_size = {code_size}"
                ))
                .unwrap();
                config.onboarding.completed = true;
                config.interface_font_family = "Helvetica Neue".to_owned();
            })
        });
        settle_visual(cx);
        let row = cx
            .debug_bounds(selector(row_selector.clone()))
            .expect("DAG row");
        let summary = cx
            .debug_bounds(selector(format!("dag-summary-{row_ix}")))
            .expect("summary");
        let meta = cx
            .debug_bounds(selector(format!("dag-meta-{row_ix}")))
            .expect("metadata");
        let scale = interface_size / 12.;
        assert!(
            summary.size.height >= px(2. * 13. * scale * 1.2)
                && summary.size.height <= px(2. * 13. * scale * 1.35),
            "two summary lines should stay visible with compact spacing: {summary:?}"
        );
        assert!(
            meta.origin.y >= summary.bottom() + px(3.),
            "metadata overlaps the summary"
        );
        assert!(
            meta.bottom() <= row.bottom() - px(6. * scale),
            "metadata clips outside the row: {meta:?} in {row:?}"
        );
        if interface_size == code_size {
            assert!(
                row.size.height <= px(92. * scale),
                "DAG row is too tall: {row:?}"
            );
        }
    }
}
