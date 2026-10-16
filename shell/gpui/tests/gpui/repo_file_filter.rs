use std::sync::Arc;

use crate::harness::{keys, open_fixture, settle_visual};
use gpui::{Modifiers, TestAppContext};
use jayjay_core::mock::diff_hunk;
use jayjay_core::{DiffContent, DiffHunk};
use jayjay_gpui::repo::ActivePane;
use jj_test::{LinearFixture, run_jj_in};

const LFS_POINTER: &str = "version https://git-lfs.github.com/spec/v1\noid sha256:496634778d7b9bdbdb4b98b43a08a00ce8d794ed135a0cb1f345bf6febc5b9b4\nsize 742800\n";

fn set_hide_git_lfs(cx: &mut gpui::VisualTestContext, hide: bool) {
    cx.cx.update(|cx| {
        jayjay_gpui::app::config::update(cx, |config| config.diff.hide_git_lfs = hide);
    });
}

fn set_submodule_support(cx: &mut gpui::VisualTestContext, enabled: bool) {
    cx.cx.update(|cx| {
        jayjay_gpui::app::config::update(cx, |config| {
            config.diff.enable_git_submodule_support = enabled
        });
    });
}

fn visible_paths(
    view: &gpui::Entity<jayjay_gpui::repo::RepoWindow>,
    cx: &mut gpui::VisualTestContext,
) -> Vec<String> {
    view.read_with(cx, |view, cx| view.visible_file_paths(cx))
}

/// Summary hunks carry no content, so the placeholder clauses need the content a loaded hunk would have.
fn set_hunk_content(
    view: &gpui::Entity<jayjay_gpui::repo::RepoWindow>,
    cx: &mut gpui::VisualTestContext,
    path: &str,
    content: &str,
) {
    view.update_in(cx, |view, _, cx| {
        view.view_model().update(cx, |vm, cx| {
            let files = vm.files.as_mut().expect("files loaded");
            let hunk = Arc::make_mut(files)
                .iter_mut()
                .find(|hunk| hunk.path == path)
                .expect("hunk present");
            hunk.new = DiffContent::new(Some(content.to_owned()), None);
            cx.notify();
        });
    });
    settle_visual(cx);
}

#[gpui::test]
fn lfs_pointer_files_hide_by_default_and_reappear_when_the_setting_is_off(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    fixture.add_tracked_working_copy_edits();
    std::fs::write(fixture.path.join("model.bin"), LFS_POINTER).expect("write lfs pointer");
    run_jj_in(&fixture.path, &["st"]);

    let (view, cx) = open_fixture(&fixture, cx);
    set_hunk_content(
        &view,
        cx,
        "model.bin",
        "<git lfs pointer sha256:496634778d7b (742800 bytes)>",
    );

    let visible = visible_paths(&view, cx);
    assert!(!visible.contains(&"model.bin".to_owned()), "{visible:?}");

    set_hide_git_lfs(cx, false);
    settle_visual(cx);

    let visible = visible_paths(&view, cx);
    assert!(visible.contains(&"model.bin".to_owned()), "{visible:?}");
}

#[gpui::test]
fn submodule_placeholders_hide_until_submodule_support_is_enabled(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    fixture.add_tracked_working_copy_edits();
    let (view, cx) = open_fixture(&fixture, cx);

    let submodule = DiffHunk {
        path: "vendor/sub".to_owned(),
        new: DiffContent::new(Some("<git submodule deadbeef>".to_owned()), None),
        ..diff_hunk("vendor/sub")
    };
    view.update_in(cx, |view, _, cx| {
        view.view_model().update(cx, |vm, cx| {
            let mut files = (**vm.files.as_ref().expect("files loaded")).clone();
            files.push(submodule);
            vm.files = Some(Arc::new(files));
            cx.notify();
        });
    });
    settle_visual(cx);

    let visible = visible_paths(&view, cx);
    assert!(!visible.contains(&"vendor/sub".to_owned()), "{visible:?}");

    set_submodule_support(cx, true);
    settle_visual(cx);

    let visible = visible_paths(&view, cx);
    assert!(visible.contains(&"vendor/sub".to_owned()), "{visible:?}");
}

#[gpui::test]
fn hiding_every_file_shows_the_hidden_only_diff_placeholder(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    run_jj_in(&fixture.path, &["new", "-m", "seed"]);
    std::fs::write(fixture.path.join("model.bin"), LFS_POINTER).expect("write lfs pointer");
    run_jj_in(&fixture.path, &["st"]);

    let (view, cx) = open_fixture(&fixture, cx);
    set_hunk_content(
        &view,
        cx,
        "model.bin",
        "<git lfs pointer sha256:496634778d7b (742800 bytes)>",
    );

    let visible = visible_paths(&view, cx);
    assert!(visible.is_empty(), "{visible:?}");
    assert!(
        cx.debug_bounds("hidden-git-files-placeholder").is_some(),
        "the diff pane must explain that every changed file is hidden"
    );

    set_hide_git_lfs(cx, false);
    settle_visual(cx);

    assert!(
        cx.debug_bounds("hidden-git-files-placeholder").is_none(),
        "the placeholder must go away once the file is visible"
    );
    assert_eq!(visible_paths(&view, cx), ["model.bin"]);
}

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

    keys(cx, "enter");
    view.read_with(cx, |view, cx| {
        assert_eq!(view.file_filter_query(), Some("READme"));
        assert_eq!(view.visible_file_paths(cx), ["README.md"]);
        assert_eq!(view.active_pane(), ActivePane::FileColumn);
    });

    keys(cx, "escape");
    view.read_with(cx, |view, cx| {
        assert!(!view.file_filter_visible());
        assert!(view.visible_file_paths(cx).len() > 1);
        assert_eq!(view.active_pane(), ActivePane::FileColumn);
    });
}
