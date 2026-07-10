mod support;

use std::fs;

use gpui::{
    Entity, Pixels, Point, ScrollDelta, ScrollWheelEvent, TestAppContext, TouchPhase,
    VisualTestContext, point, px, size,
};
use jayjay_gpui::repo::RepoWindow;
use jj_test::{LinearFixture, run_jj_in};
use support::{install_test_globals, load_selected_change_files, settle_visual};

/// Regression: the preview once grew to fit its full content instead of being clamped to the pane, leaving nothing to scroll.
#[gpui::test]
fn scrolling_a_long_markdown_preview_moves_its_content(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_repo_with_markdown_files(cx, &[("BIG.md", 400)]);
    select_and_preview(&view, cx, "BIG.md");

    let pane = cx
        .debug_bounds("markdown-preview-pane")
        .expect("preview pane");
    let before = view.read_with(cx, |view, _| view.markdown_preview_scroll_offset_y());
    assert_eq!(before, px(0.), "preview should open scrolled to the top");

    scroll_pane(cx, pane.center());

    let after = view.read_with(cx, |view, _| view.markdown_preview_scroll_offset_y());
    assert_eq!(
        after,
        px(-300.),
        "scrolling over the preview should move its content, not leave it stuck"
    );
}

#[gpui::test]
fn switching_files_resets_the_markdown_preview_scroll(cx: &mut TestAppContext) {
    let (_fixture, view, cx) =
        open_repo_with_markdown_files(cx, &[("BIG.md", 400), ("SMALL.md", 1)]);
    select_and_preview(&view, cx, "BIG.md");

    let pane = cx
        .debug_bounds("markdown-preview-pane")
        .expect("preview pane");
    scroll_pane(cx, pane.center());
    let scrolled = view.read_with(cx, |view, _| view.markdown_preview_scroll_offset_y());
    assert_ne!(
        scrolled,
        px(0.),
        "scroll should have moved before switching files"
    );

    select_and_preview(&view, cx, "SMALL.md");

    let after_switch = view.read_with(cx, |view, _| view.markdown_preview_scroll_offset_y());
    assert_eq!(
        after_switch,
        px(0.),
        "selecting a different file should reset the preview scroll to the top"
    );
}

/// Regression: laid out as a flex row, the document once ignored the pane width and long paragraphs ran off the right edge.
#[gpui::test]
fn markdown_document_is_clamped_to_the_pane_width(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_repo_with_markdown_files(cx, &[("WIDE.md", 40)]);
    select_and_preview(&view, cx, "WIDE.md");

    let pane = cx
        .debug_bounds("markdown-preview-pane")
        .expect("preview pane");
    let document = cx
        .debug_bounds("markdown-document")
        .expect("markdown document");
    assert!(
        document.size.width <= pane.size.width,
        "document width {:?} must not exceed pane width {:?}",
        document.size.width,
        pane.size.width
    );
}

fn scroll_pane(cx: &mut VisualTestContext, at: Point<Pixels>) {
    cx.simulate_event(ScrollWheelEvent {
        position: at,
        delta: ScrollDelta::Pixels(point(px(0.), px(-300.))),
        modifiers: Default::default(),
        touch_phase: TouchPhase::Moved,
    });
    settle_visual(cx);
}

fn select_and_preview(view: &Entity<RepoWindow>, cx: &mut VisualTestContext, path: &str) {
    let ix = view.update_in(cx, |view, _, cx| {
        view.view_model()
            .read(cx)
            .files
            .as_ref()
            .unwrap()
            .iter()
            .position(|h| h.path == path)
            .unwrap_or_else(|| panic!("{path} present"))
    });
    view.update_in(cx, |view, _, cx| view.select_file(ix, cx));
    view.update_in(cx, |view, _, cx| view.toggle_markdown_rich_preview(cx));
    settle_visual(cx);
}

fn open_repo_with_markdown_files<'a>(
    cx: &'a mut TestAppContext,
    files: &[(&str, usize)],
) -> (LinearFixture, Entity<RepoWindow>, &'a mut VisualTestContext) {
    let fixture = LinearFixture::build();
    for (name, line_count) in files {
        let mut body = String::new();
        for i in 0..*line_count {
            let sentence = format!("Line number {i} of a long markdown document. ").repeat(8);
            body.push_str(&format!("{}\n\n", sentence.trim_end()));
        }
        fs::write(fixture.path.join(name), body).expect("write markdown fixture");
    }
    run_jj_in(&fixture.path, &["st"]);

    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    cx.simulate_resize(size(px(1200.), px(800.)));
    load_selected_change_files(&view, cx);
    settle_visual(cx);

    (fixture, view, cx)
}
