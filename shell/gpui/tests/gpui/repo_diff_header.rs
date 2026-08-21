use std::fs;

use crate::harness::{install_test_globals, load_selected_change_files, settle_visual};
use gpui::{Modifiers, TestAppContext, VisualTestContext, px, size};
use jayjay_core::DiffProjectionMode;
use jayjay_gpui::app::fonts;
use jayjay_gpui::repo::RepoWindow;
use jj_test::{FormatFixture, LinearFixture, run_jj_in};

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
fn empty_working_copy_description_hides_body_and_resize_handle(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_repo_with_selected_file(cx, "README.md");

    view.read_with(cx, |view, cx| {
        let change = view
            .view_model()
            .read(cx)
            .selected_change()
            .expect("selected working copy");
        assert!(change.is_working_copy);
        assert!(change.description.trim().is_empty());
    });

    assert!(cx.debug_bounds("detail-description").is_some());
    assert!(
        cx.debug_bounds("description-body").is_none(),
        "empty descriptions should not show placeholder body text"
    );
    assert!(
        cx.debug_bounds("description-resize-handle").is_none(),
        "empty descriptions should not show a resize handle"
    );
}

#[gpui::test]
fn mutable_change_description_header_shows_pencil_then_edit_diff(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);

    select_change_by_description(&view, cx, "add hello");

    let pencil = cx
        .debug_bounds("edit-description")
        .expect("edit pencil should show for a mutable, non-working-copy change");
    let edit_diff = cx
        .debug_bounds("edit-diff")
        .expect("Edit Diff affordance should show when the change has a diff to edit");
    assert!(
        pencil.origin.x < edit_diff.origin.x,
        "pencil should sit right after the heading, with Edit Diff pinned to the trailing edge"
    );
}

#[gpui::test]
fn empty_change_hides_edit_diff_button(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    // Push the empty change back into history so it's mutable but no longer the working copy, isolating the diff-emptiness gate from the working-copy gate.
    run_jj_in(&fixture.path, &["new", "-m", "empty change"]);
    run_jj_in(&fixture.path, &["new"]);
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);

    select_change_by_description(&view, cx, "empty change");

    assert!(
        cx.debug_bounds("edit-description").is_some(),
        "pencil should still show: the change is mutable and not the working copy"
    );
    assert!(
        cx.debug_bounds("edit-diff").is_none(),
        "Edit Diff should not appear for a change with no diff to edit"
    );
}

#[gpui::test]
fn binary_plist_projection_banner_is_inset(cx: &mut TestAppContext) {
    let (_fixture, _view, cx) = open_format_repo_with_selected_file(cx, FormatFixture::PLIST);

    let banner = cx
        .debug_bounds("projection-banner")
        .expect("binary plist projection banner");
    let gutter = cx.debug_bounds("diff-gutter").expect("text diff gutter");
    assert!(
        banner.origin.x > gutter.origin.x,
        "projection banner should be inset from the diff card edge"
    );
    assert!(
        banner.size.height <= px(42.),
        "projection banner should stay compact, got {:?}",
        banner.size.height
    );
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
    let text_gutter_width = cx
        .debug_bounds("diff-gutter")
        .expect("raw projection text diff gutter")
        .size
        .width;

    let toggle = cx
        .debug_bounds("toggle-projection-preview")
        .expect("projection preview toggle");
    cx.simulate_click(toggle.center(), Modifiers::default());
    cx.run_until_parked();
    cx.cx.run_until_parked();
    let immediate_rich_gutter_width = cx
        .debug_bounds("rich-preview-gutter")
        .expect("projection preview should reserve its gutter while processed diff loads")
        .size
        .width;
    assert_eq!(
        immediate_rich_gutter_width, text_gutter_width,
        "projection rich preview should keep the same gutter width during mode switches"
    );
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
        assert!(vm.current_markdown_preview.is_some());
    });
    assert!(cx.debug_bounds("markdown-preview-pane").is_some());
    let rich_gutter_width = cx
        .debug_bounds("rich-preview-gutter")
        .expect("projection rich preview gutter")
        .size
        .width;
    assert_eq!(
        rich_gutter_width, text_gutter_width,
        "projection rich preview should reserve the same gutter width as the text diff"
    );

    let toggle = cx
        .debug_bounds("toggle-projection-preview")
        .expect("projection preview toggle after activation");
    cx.simulate_click(toggle.center(), Modifiers::default());
    cx.run_until_parked();
    cx.cx.run_until_parked();
    let loading_or_text_gutter_width = cx
        .debug_bounds("diff-loading-gutter")
        .or_else(|| cx.debug_bounds("diff-gutter"))
        .expect("projection raw-mode handoff should keep a gutter")
        .size
        .width;
    assert_eq!(
        loading_or_text_gutter_width, text_gutter_width,
        "projection raw-mode handoff should keep the same gutter width"
    );
    settle_visual(cx);
    assert!(cx.debug_bounds("markdown-preview-pane").is_none());
}

#[gpui::test]
fn csv_projection_preview_uses_plain_table_chrome(cx: &mut TestAppContext) {
    let (_fixture, _view, cx) = open_format_repo_with_selected_file(cx, FormatFixture::CSV);
    let text_gutter_width = cx
        .debug_bounds("diff-gutter")
        .expect("raw csv text diff gutter")
        .size
        .width;

    let toggle = cx
        .debug_bounds("toggle-projection-preview")
        .expect("csv projection preview toggle");
    cx.simulate_click(toggle.center(), Modifiers::default());
    settle_visual(cx);

    let pane = cx
        .debug_bounds("markdown-preview-pane")
        .expect("csv projection preview pane");
    let table = cx
        .debug_bounds("markdown-table-preview")
        .expect("csv projection should render a table");
    assert!(
        cx.debug_bounds("rich-preview-metadata").is_none(),
        "csv table preview should not show Markdown block metadata"
    );
    let rich_gutter_width = cx
        .debug_bounds("rich-preview-gutter")
        .expect("csv projection preview gutter")
        .size
        .width;
    assert_eq!(
        rich_gutter_width, text_gutter_width,
        "csv projection rich preview should reserve the same gutter width as the text diff"
    );
    assert!(
        f32::from(table.size.width) > f32::from(pane.size.width) * 0.7,
        "csv table should use the preview width instead of a compact Markdown table"
    );
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

#[gpui::test]
fn markdown_preview_button_toggles_rendered_markdown(cx: &mut TestAppContext) {
    let target = "docs/README.md";
    let markdown = "# Title\n\n- [x] Done\n- [ ] Todo\n";
    let (_fixture, view, cx) = open_repo_with_selected_file_content(cx, target, markdown);

    let toggle = cx
        .debug_bounds("toggle-markdown-preview")
        .expect("markdown preview toggle");
    let copy = cx.debug_bounds("diff-copy-path").expect("copy path button");
    let gap = toggle.origin.x - (copy.origin.x + copy.size.width);
    assert!(
        gap <= px(8.),
        "markdown preview button should sit next to copy, gap was {gap:?}"
    );

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert_eq!(
            vm.current_markdown_preview
                .as_ref()
                .map(|document| document.source()),
            Some(markdown)
        );
    });
    assert!(cx.debug_bounds("markdown-preview-pane").is_none());
    let text_gutter_width = cx
        .debug_bounds("diff-gutter")
        .expect("text diff gutter")
        .size
        .width;

    cx.simulate_click(toggle.center(), Modifiers::default());
    settle_visual(cx);
    assert!(cx.debug_bounds("markdown-preview-pane").is_some());
    let rich_gutter_width = cx
        .debug_bounds("rich-preview-gutter")
        .expect("rich preview gutter")
        .size
        .width;
    assert_eq!(
        rich_gutter_width, text_gutter_width,
        "rich preview should reserve the same gutter width as the text diff"
    );

    let toggle = cx
        .debug_bounds("toggle-markdown-preview")
        .expect("markdown preview toggle after activation");
    cx.simulate_click(toggle.center(), Modifiers::default());
    settle_visual(cx);
    assert!(cx.debug_bounds("markdown-preview-pane").is_none());
}

fn select_change_by_description(
    view: &gpui::Entity<RepoWindow>,
    cx: &mut VisualTestContext,
    description: &str,
) {
    view.update_in(cx, |view, _, cx| {
        let ix = {
            let vm = view.view_model().read(cx);
            vm.graph
                .changes
                .iter()
                .position(|change| change.description.trim() == description)
                .unwrap_or_else(|| panic!("fixture should contain a \"{description}\" change"))
        };
        view.view_model()
            .update(cx, |vm, cx| vm.select_change(ix, cx));
    });
    settle_visual(cx);
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
