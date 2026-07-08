mod support;

use std::fs;

use gpui::{Modifiers, TestAppContext, VisualTestContext, px, size};
use jayjay_core::DiffProjectionMode;
use jayjay_gpui::app::fonts;
use jayjay_gpui::repo::RepoWindow;
use jj_test::{FormatFixture, LinearFixture, run_jj_in};
use support::{install_test_globals, load_selected_change_files, settle_visual};

#[gpui::test]
fn diff_header_keeps_medium_repo_paths_visible(cx: &mut TestAppContext) {
    let target = "shell/gpui/src/diff/diff_view/header.rs";
    let (_fixture, _view, cx) = open_repo_with_selected_file(cx, target);
    cx.simulate_resize(size(px(1280.), px(720.)));
    settle_visual(cx);

    let path_bounds = cx.debug_bounds("diff-file-path").expect("path bounds");
    let copy = cx.debug_bounds("diff-copy-path").expect("copy path button");
    let mode = cx.debug_bounds("toggle-mode").expect("mode toggle");
    let advance = cx.cx.update(|cx| fonts::mono_advance(cx, px(13.)));
    let expected_width = f32::from(advance) * target.chars().count() as f32;
    let actual_width = f32::from(path_bounds.size.width);
    assert!(
        actual_width > expected_width * 0.85,
        "path header collapsed to {:?}, expected roughly {expected_width}px",
        path_bounds.size.width,
    );
    assert!(
        mode.origin.x - (copy.origin.x + copy.size.width) > px(24.),
        "copy button should stay with the path instead of pinning to the mode toggle"
    );
}

#[gpui::test]
fn diff_header_copy_path_button_copies_selected_file_path(cx: &mut TestAppContext) {
    let target = "shell/gpui/src/repo/toolbar.rs";
    let (_fixture, _view, cx) = open_repo_with_selected_file(cx, target);

    let path = cx.debug_bounds("diff-file-path").expect("path bounds");
    let copy = cx.debug_bounds("diff-copy-path").expect("copy path button");
    let gap = copy.origin.x - (path.origin.x + path.size.width);
    assert!(
        gap <= px(8.),
        "copy button should sit next to the path, gap was {gap:?}"
    );

    cx.simulate_click(copy.center(), Modifiers::default());

    let copied = cx
        .read_from_clipboard()
        .and_then(|item| item.text())
        .expect("copy path should write text");
    assert_eq!(copied, target);
}

#[gpui::test]
fn diff_header_opens_working_copy_html_in_default_app(cx: &mut TestAppContext) {
    let target = "docs/preview page.html";
    let (_fixture, _view, cx) = open_repo_with_selected_file(cx, target);

    let button = cx
        .debug_bounds("open-html-external")
        .expect("html external-open button");
    let copy = cx.debug_bounds("diff-copy-path").expect("copy path button");
    let gap = button.origin.x - (copy.origin.x + copy.size.width);
    assert!(
        gap <= px(8.),
        "html external-open button should sit next to copy, gap was {gap:?}"
    );

    cx.simulate_click(button.center(), Modifiers::default());

    let opened = cx.opened_url().expect("html button should open a URL");
    assert!(opened.starts_with("file:///"), "{opened}");
    assert!(opened.ends_with("/docs/preview%20page.html"), "{opened}");
}

#[gpui::test]
fn projection_preview_button_toggles_processed_diff(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_format_repo_with_selected_file(cx, FormatFixture::NOTEBOOK);

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert_eq!(
            vm.current_projection.as_ref().map(|p| p.mode),
            Some(DiffProjectionMode::Raw)
        );
        assert_eq!(
            vm.current_diff.as_ref().map(|diff| diff.path.as_str()),
            Some(FormatFixture::NOTEBOOK)
        );
    });

    let toggle = cx
        .debug_bounds("toggle-projection-preview")
        .expect("projection preview toggle");
    cx.simulate_click(toggle.center(), Modifiers::default());
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert_eq!(
            vm.current_projection.as_ref().map(|p| p.mode),
            Some(DiffProjectionMode::Processed)
        );
        assert_eq!(
            vm.current_diff.as_ref().map(|diff| diff.path.as_str()),
            Some("analysis.ipynb.md")
        );
    });
}

#[gpui::test]
fn svg_preview_button_toggles_rendered_svg(cx: &mut TestAppContext) {
    let target = "assets/logo.svg";
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32"><rect width="32" height="32" fill="#24c85a"/></svg>"##;
    let (_fixture, view, cx) = open_repo_with_selected_file_content(cx, target, svg);

    let toggle = cx
        .debug_bounds("toggle-svg-preview")
        .expect("svg preview toggle");
    let copy = cx.debug_bounds("diff-copy-path").expect("copy path button");
    let gap = toggle.origin.x - (copy.origin.x + copy.size.width);
    assert!(
        gap <= px(8.),
        "svg preview button should sit next to copy, gap was {gap:?}"
    );

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert_eq!(
            vm.current_svg_preview.as_ref().map(|p| p.new.as_deref()),
            Some(Some(svg))
        );
    });
    assert!(cx.debug_bounds("svg-preview-pane").is_none());

    cx.simulate_click(toggle.center(), Modifiers::default());
    settle_visual(cx);
    assert!(cx.debug_bounds("svg-preview-pane").is_some());

    let toggle = cx
        .debug_bounds("toggle-svg-preview")
        .expect("svg preview toggle after activation");
    cx.simulate_click(toggle.center(), Modifiers::default());
    settle_visual(cx);
    assert!(cx.debug_bounds("svg-preview-pane").is_none());
}

fn open_repo_with_selected_file<'a>(
    cx: &'a mut TestAppContext,
    target: &str,
) -> (
    LinearFixture,
    gpui::Entity<RepoWindow>,
    &'a mut VisualTestContext,
) {
    open_repo_with_selected_file_content(cx, target, "tools\n")
}

fn open_repo_with_selected_file_content<'a>(
    cx: &'a mut TestAppContext,
    target: &str,
    content: &str,
) -> (
    LinearFixture,
    gpui::Entity<RepoWindow>,
    &'a mut VisualTestContext,
) {
    let fixture = LinearFixture::build();
    let parent = fixture
        .path
        .join(target)
        .parent()
        .expect("target has parent")
        .to_owned();
    fs::create_dir_all(parent).expect("create nested dirs");
    fs::write(fixture.path.join(target), content).expect("write target file");
    run_jj_in(&fixture.path, &["st"]);

    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);

    view.update_in(cx, |view, _, cx| {
        let ix = {
            let vm = view.view_model().read(cx);
            vm.files
                .as_ref()
                .expect("working copy files")
                .iter()
                .position(|hunk| hunk.path == target)
                .expect("target hunk")
        };
        view.select_file(ix, cx);
    });
    settle_visual(cx);

    (fixture, view, cx)
}

fn open_format_repo_with_selected_file<'a>(
    cx: &'a mut TestAppContext,
    target: &str,
) -> (
    FormatFixture,
    gpui::Entity<RepoWindow>,
    &'a mut VisualTestContext,
) {
    let fixture = FormatFixture::build();

    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);

    view.update_in(cx, |view, _, cx| {
        let ix = {
            let vm = view.view_model().read(cx);
            vm.files
                .as_ref()
                .expect("working copy files")
                .iter()
                .position(|hunk| hunk.path == target)
                .expect("target hunk")
        };
        view.select_file(ix, cx);
    });
    settle_visual(cx);

    (fixture, view, cx)
}
