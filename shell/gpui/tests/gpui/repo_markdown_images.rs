use std::fs;

use crate::harness::{
    install_test_globals, load_selected_change_files, select_file, selector, settle_visual,
};
use gpui::{TestAppContext, px, size};
use jayjay_gpui::repo::RepoWindow;
use jj_test::{LinearFixture, run_jj_in};

const INLINE_PNG: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==";

#[gpui::test]
fn markdown_preview_renders_repo_and_inline_images_and_keeps_placeholders_for_missing_ones(
    cx: &mut TestAppContext,
) {
    let fixture = LinearFixture::build();
    let images = fixture.path.join("docs").join("images");
    fs::create_dir_all(&images).expect("create image dir");
    image::RgbaImage::new(2000, 500)
        .save(images.join("flow.png"))
        .expect("write png fixture");
    fs::write(
        images.join("logo.svg"),
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"120\" height=\"40\"><rect width=\"120\" height=\"40\"/></svg>",
    )
    .expect("write svg fixture");
    fs::write(
        fixture.path.join("docs").join("guide.md"),
        format!(
            "# Guide\n\n![Flow](images/flow.png)\n\n![Up](../docs/images/flow.png)\n\n![Escape](../../flow.png)\n\n![Raw](images/flow.png?raw=1)\n\n![Logo](images/logo.svg)\n\n![Missing](images/missing.png)\n\n<img src=\"{INLINE_PNG}\" alt=\"Inline\">\n"
        ),
    )
    .expect("write markdown fixture");
    run_jj_in(&fixture.path, &["st"]);

    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    cx.simulate_resize(size(px(1200.), px(800.)));
    load_selected_change_files(&view, cx);
    settle_visual(cx);
    select_file(&view, "docs/guide.md", cx);
    view.update_in(cx, |view, _, cx| view.toggle_markdown_rich_preview(cx));
    settle_visual(cx);
    cx.update(|window, cx| {
        window.simulate_next_frame(cx);
    });
    settle_visual(cx);

    let pane = cx
        .debug_bounds("markdown-preview-pane")
        .expect("preview pane");
    let flow = cx
        .debug_bounds("markdown-image:images/flow.png")
        .expect("image inside the checkout renders");
    assert!(
        flow.size.width > px(0.) && flow.size.width < pane.size.width,
        "a 2000px image is scaled down to the pane: {flow:?} in {pane:?}"
    );
    assert!(
        (f32::from(flow.size.height) * 4. - f32::from(flow.size.width)).abs() < 1.,
        "scaling keeps the 4:1 aspect ratio: {flow:?}"
    );
    assert!(
        cx.debug_bounds(selector(format!("markdown-image:{INLINE_PNG}")))
            .is_some(),
        "inline data image renders"
    );
    assert!(
        cx.debug_bounds("markdown-image:images/flow.png?raw=1")
            .is_some(),
        "query suffixes do not hide a repo image"
    );
    assert!(
        cx.debug_bounds("markdown-image:../docs/images/flow.png")
            .is_some(),
        "parent-relative paths that stay in the checkout render"
    );
    assert!(
        cx.debug_bounds("markdown-image-placeholder:../../flow.png")
            .is_some(),
        "paths escaping the checkout keep the placeholder"
    );
    let logo = cx
        .debug_bounds("markdown-image:images/logo.svg")
        .expect("svg image renders");
    assert!(
        logo.size.height > px(0.) && logo.size.width < pane.size.width,
        "svg sized from its own dimensions: {logo:?}"
    );
    assert!(
        cx.debug_bounds("markdown-image-placeholder:images/missing.png")
            .is_some(),
        "missing file keeps the labelled placeholder"
    );
    assert!(
        cx.debug_bounds("markdown-image:images/missing.png")
            .is_none()
    );
}
