use std::fs;
use std::sync::Arc;

use crate::harness::*;
use gpui::{AppContext, Focusable, ScrollStrategy, TestAppContext, VisualTestContext, px};
use jayjay_gpui::diff::{DiffSelection, SbsSide};
use jayjay_gpui::repo::view_model::RepoViewModel;
use jayjay_gpui::repo::{ActivePane, RepoWindow, revset};
use jj_test::{LinearFixture, run_jj_in};

#[gpui::test]
fn reselecting_current_file_does_not_reset_diff_panel(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    suppress_fs_watcher(cx);
    let view = cx.new(|cx| RepoWindow::new(fixture.path.clone(), cx));

    view.update(cx, |view, cx| {
        view.view_model().update(cx, |vm, _| {
            vm.selected_file_ix = Some(0);
        });
        view.set_active_pane(ActivePane::Sidebar);
        view.set_diff_selection(Some(DiffSelection::start(2, 3, SbsSide::Unified)));

        view.select_file(0, cx);

        assert_eq!(view.active_pane(), ActivePane::FileColumn);
        assert!(view.has_diff_selection());
        assert_eq!(view.pending_diff_scroll_target(), None);
    });
}

#[gpui::test]
fn selecting_new_file_resets_diff_scroll_to_top(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    suppress_fs_watcher(cx);
    let view = cx.new(|cx| RepoWindow::new(fixture.path.clone(), cx));

    view.update(cx, |view, cx| {
        view.view_model().update(cx, |vm, _| {
            vm.selected_file_ix = Some(0);
        });
        view.set_diff_selection(Some(DiffSelection::start(2, 3, SbsSide::Unified)));
        view.set_diff_scroll_offset_y(px(-240.));

        view.select_file(1, cx);

        assert_eq!(view.view_model().read(cx).selected_file_ix, Some(1));
        assert!(!view.has_diff_selection());
        assert_eq!(view.diff_scroll_offset_y(), px(0.));
        assert_eq!(
            view.pending_diff_scroll_target(),
            Some((0, ScrollStrategy::Top, true))
        );
    });
}

#[gpui::test]
fn selecting_preloaded_file_invalidates_an_older_diff_request(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    fixture.add_tracked_working_copy_edits();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);

    let (target_ix, target_path, previous_generation) = view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        let files = vm.files.as_ref().expect("files loaded");
        assert_eq!(
            vm.diff_cache.len(),
            files.len(),
            "the fixture should preload every file so this exercises the cache-hit path"
        );
        let target_ix = files
            .iter()
            .position(|hunk| hunk.path == "feature.txt")
            .expect("feature.txt hunk");
        assert_ne!(
            vm.selected_file_ix,
            Some(target_ix),
            "the target must differ from the current selection"
        );
        (
            target_ix,
            files[target_ix].path.clone(),
            vm.loading.diff_gen,
        )
    });

    view.update_in(cx, |view, _, cx| view.select_file(target_ix, cx));

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert_eq!(
            vm.loading.diff_gen,
            previous_generation.wrapping_add(1),
            "a cache hit must supersede any older in-flight diff completion"
        );
        assert!(
            !vm.loading.diff,
            "the preloaded diff should apply synchronously"
        );
        assert_eq!(
            vm.current_diff.as_ref().map(|diff| diff.path.as_str()),
            Some(target_path.as_str())
        );
    });
}

#[gpui::test]
fn clear_compare_selects_fallback_when_target_is_missing(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let vm = cx.new(|_| RepoViewModel::new(fixture.path.clone()));

    let fallback = vm.read_with(cx, |vm, _| {
        vm.graph
            .changes
            .iter()
            .position(|change| change.is_working_copy)
            .unwrap_or(0)
    });
    vm.update(cx, |vm, cx| {
        vm.compare = Some(revset::CompareState {
            from_rev: "main".to_owned(),
            to_rev: "missing-change".to_owned(),
            source_change_id: None,
            target_change_id: Some("missing-change".to_owned()),
            display: revset::CompareDisplay {
                title: "Comparing".to_owned(),
                from: "main".to_owned(),
                to: "missing-change".to_owned(),
                is_combined_selection: false,
            },
        });
        vm.selected = None;
        vm.files = Some(Arc::new(Vec::new()));
        vm.selected_file_ix = Some(0);
        vm.clear_compare(cx);

        assert_eq!(vm.compare, None);
        assert_eq!(vm.selected, Some(fallback));
        assert!(vm.files.is_none());
        assert_eq!(vm.selected_file_ix, None);
        assert!(vm.current_diff.is_none());
    });
}

#[gpui::test]
fn ctrl_n_navigates_working_copy_file_list(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    fixture.add_tracked_working_copy_edits();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);

    view.update_in(cx, |view, window, cx| {
        view.set_active_pane(ActivePane::FileColumn);
        view.focus_handle(cx).focus(window, cx);
        let vm = view.view_model().read(cx);
        assert!(vm.files.as_ref().map(|files| files.len()).unwrap_or(0) >= 2);
        assert_eq!(vm.selected_file_ix, Some(0));
    });

    cx.simulate_keystrokes("ctrl-n");

    view.read_with(cx, |view, cx| {
        assert_eq!(view.view_model().read(cx).selected_file_ix, Some(1));
    });
}

#[gpui::test]
fn tree_mode_nav_walks_visible_files_through_the_keystroke_path(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    // A subdirectory adds a directory row, so visible tree row indices no longer match hunk indices.
    fs::create_dir_all(fixture.path.join("src")).expect("create src dir");
    fs::write(fixture.path.join("src/a.rs"), "a\n").expect("write src/a.rs");
    fs::write(fixture.path.join("src/b.rs"), "b\n").expect("write src/b.rs");
    run_jj_in(&fixture.path, &["st"]);

    install_test_globals(cx);
    cx.update(|cx| {
        jayjay_gpui::app::config::update(cx, |c| c.diff.tree_file_list = true);
    });
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);

    view.update_in(cx, |view, window, cx| {
        view.set_active_pane(ActivePane::FileColumn);
        view.focus_handle(cx).focus(window, cx);
        view.view_model().update(cx, |vm, _| {
            vm.selected_file_ix = Some(0);
            assert!(
                vm.files.as_ref().map(|f| f.len()).unwrap_or(0) >= 4,
                "fixture should expose the subdir + root files"
            );
        });
    });

    cx.simulate_keystrokes("ctrl-n");

    view.read_with(cx, |view, cx| {
        assert_eq!(
            view.view_model().read(cx).selected_file_ix,
            Some(1),
            "tree mode navigation should advance over visible files"
        );
    });
}

#[gpui::test]
fn tree_mode_nav_skips_files_hidden_under_collapsed_dirs(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    fs::create_dir_all(fixture.path.join("src")).expect("create src dir");
    fs::write(fixture.path.join("src/a.rs"), "a\n").expect("write src/a.rs");
    fs::write(fixture.path.join("src/b.rs"), "b\n").expect("write src/b.rs");
    run_jj_in(&fixture.path, &["st"]);

    install_test_globals(cx);
    cx.update(|cx| {
        jayjay_gpui::app::config::update(cx, |c| c.diff.tree_file_list = true);
    });
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);

    // Collapse src/ to hide its files; start selection on a still-visible root file.
    let hidden_hunks: Vec<usize> = view.update_in(cx, |view, window, cx| {
        view.set_active_pane(ActivePane::FileColumn);
        view.focus_handle(cx).focus(window, cx);
        view.toggle_dir("src".to_owned(), cx);
        let (hidden, first_visible) = {
            let vm = view.view_model().read(cx);
            let files = vm.files.as_ref().expect("files loaded");
            let hidden: Vec<usize> = files
                .iter()
                .enumerate()
                .filter(|(_, h)| h.path.starts_with("src/"))
                .map(|(ix, _)| ix)
                .collect();
            let first_visible = files
                .iter()
                .position(|h| !h.path.starts_with("src/"))
                .expect("a root file stays visible");
            (hidden, first_visible)
        };
        view.view_model()
            .update(cx, |vm, _| vm.selected_file_ix = Some(first_visible));
        hidden
    });
    assert!(!hidden_hunks.is_empty(), "src/ files should exist");

    // Walk the visible list back and forth; navigation must never land on a hidden file.
    for key in ["ctrl-n", "ctrl-n", "ctrl-n", "ctrl-p", "ctrl-p"] {
        cx.simulate_keystrokes(key);
        view.read_with(cx, |view, cx| {
            let sel = view
                .view_model()
                .read(cx)
                .selected_file_ix
                .expect("a file stays selected");
            assert!(
                !hidden_hunks.contains(&sel),
                "navigation landed on a file hidden under a collapsed dir"
            );
        });
    }
}
