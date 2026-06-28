mod support;

use std::fs;

use gpui::{Modifiers, TestAppContext, VisualTestContext, px, size};
use jayjay_gpui::app::fonts;
use jayjay_gpui::repo::RepoWindow;
use jj_test::{LinearFixture, run_jj_in};
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

fn open_repo_with_selected_file<'a>(
    cx: &'a mut TestAppContext,
    target: &str,
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
    fs::write(fixture.path.join(target), "tools\n").expect("write target file");
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
