use std::sync::Arc;

use crate::harness::{keys, open_fixture, settle_visual};
use gpui::{Entity, Modifiers, TestAppContext, VisualTestContext};
use jayjay_core::mock::diff_hunk;
use jayjay_core::{DiffContent, DiffHunk};
use jayjay_gpui::app::config::{self, DiffConfig};
use jayjay_gpui::repo::{ActivePane, RepoWindow};
use jj_test::{LinearFixture, run_jj_in};

const LFS_POINTER: &str = "version https://git-lfs.github.com/spec/v1\noid sha256:496634778d7b9bdbdb4b98b43a08a00ce8d794ed135a0cb1f345bf6febc5b9b4\nsize 742800\n";

const LFS_SUMMARY: &str = "<git lfs pointer sha256:496634778d7b (742800 bytes)>";

fn set_diff_config(cx: &mut VisualTestContext, edit: impl FnOnce(&mut DiffConfig)) {
    cx.cx
        .update(|cx| config::update(cx, |config| edit(&mut config.diff)));
    settle_visual(cx);
}

fn visible_paths(view: &Entity<RepoWindow>, cx: &mut VisualTestContext) -> Vec<String> {
    view.read_with(cx, |view, cx| view.visible_file_paths(cx))
}

// Summary hunks carry no content, so a test sets what a loaded hunk would hold.
fn set_hunk_content(view: &Entity<RepoWindow>, cx: &mut VisualTestContext, hunk: DiffHunk) {
    view.update_in(cx, |view, _, cx| {
        view.view_model().update(cx, |vm, cx| {
            let files = Arc::make_mut(vm.files.as_mut().expect("files loaded"));
            match files.iter_mut().find(|file| file.path == hunk.path) {
                Some(file) => file.new = hunk.new,
                None => files.push(hunk),
            }
            cx.notify();
        });
    });
    settle_visual(cx);
}

fn hunk(path: &str, content: &str) -> DiffHunk {
    DiffHunk {
        new: DiffContent::new(Some(content.to_owned()), None),
        ..diff_hunk(path)
    }
}

#[gpui::test]
fn lfs_and_submodule_files_follow_their_visibility_settings(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    fixture.add_tracked_working_copy_edits();
    std::fs::write(fixture.path.join("model.bin"), LFS_POINTER).expect("write lfs pointer");
    run_jj_in(&fixture.path, &["st"]);
    let (view, cx) = open_fixture(&fixture, cx);
    set_hunk_content(&view, cx, hunk("model.bin", LFS_SUMMARY));
    set_hunk_content(&view, cx, hunk("vendor/sub", "<git submodule deadbeef>"));

    let visible = visible_paths(&view, cx);
    assert!(
        !visible
            .iter()
            .any(|path| path == "model.bin" || path == "vendor/sub"),
        "{visible:?}"
    );

    set_diff_config(cx, |diff| diff.hide_git_lfs = false);
    assert!(visible_paths(&view, cx).contains(&"model.bin".to_owned()));
    set_diff_config(cx, |diff| diff.enable_git_submodule_support = true);
    assert!(visible_paths(&view, cx).contains(&"vendor/sub".to_owned()));
}

#[gpui::test]
fn hiding_every_file_shows_the_hidden_only_diff_placeholder(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    run_jj_in(&fixture.path, &["new", "-m", "seed"]);
    std::fs::write(fixture.path.join("model.bin"), LFS_POINTER).expect("write lfs pointer");
    run_jj_in(&fixture.path, &["st"]);
    let (view, cx) = open_fixture(&fixture, cx);
    set_hunk_content(&view, cx, hunk("model.bin", LFS_SUMMARY));

    assert!(visible_paths(&view, cx).is_empty());
    assert!(cx.debug_bounds("hidden-git-files-placeholder").is_some());

    set_diff_config(cx, |diff| diff.hide_git_lfs = false);
    assert!(cx.debug_bounds("hidden-git-files-placeholder").is_none());
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
