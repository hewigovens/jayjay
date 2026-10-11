use std::fs;
use std::sync::Arc;

use crate::harness::*;
use gpui::{AppContext, Focusable, ScrollStrategy, TestAppContext, VisualTestContext, px};
use jayjay_core::compare;
use jayjay_gpui::app::{actions::ToggleIgnoreWhitespace, menus};
use jayjay_gpui::diff::{DiffSelection, SbsSide};
use jayjay_gpui::repo::view_model::RepoViewModel;
use jayjay_gpui::repo::{ActivePane, RepoWindow};
use jj_test::{LinearFixture, run_jj_in};

#[gpui::test]
fn ignore_whitespace_setting_applies_on_open_and_updates_all_windows(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    fs::write(fixture.path.join("ws.txt"), "a b\nkeep\n").unwrap();
    run_jj_in(&fixture.path, &["describe", "-m", "base"]);
    run_jj_in(&fixture.path, &["new"]);
    fs::write(fixture.path.join("ws.txt"), "a  b\nkeep\n").unwrap();
    run_jj_in(&fixture.path, &["st"]);

    install_test_globals(cx);
    cx.update(|cx| {
        menus::install(cx);
        cx.dispatch_action(&ToggleIgnoreWhitespace);
    });
    let (first, first_cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    settle_visual(first_cx);
    select_file(&first, "ws.txt", first_cx);
    let (second, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    settle_visual(cx);
    select_file(&second, "ws.txt", cx);

    for view in [&first, &second] {
        view.read_with(cx, |view, cx| {
            let diff = view.view_model().read(cx).shown.diff.as_ref().unwrap();
            assert!(diff.whitespace_only_hidden);
        });
    }

    cx.cx
        .update(|cx| cx.dispatch_action(&ToggleIgnoreWhitespace));
    settle_visual(cx);
    for view in [&first, &second] {
        view.read_with(cx, |view, cx| {
            let diff = view.view_model().read(cx).shown.diff.as_ref().unwrap();
            assert!(!diff.whitespace_only_hidden);
            assert!(diff.lines.iter().any(|line| line.is_changed()));
        });
    }
}

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
    install_test_globals(cx);
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
            vm.diff_cache.loaded.len(),
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
            vm.shown.diff.as_ref().map(|diff| diff.path.as_str()),
            Some(target_path.as_str())
        );
    });
}

#[gpui::test]
fn historical_diff_loading_skips_review_snapshots(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    run_jj_in(&fixture.path, &["describe", "-m", "historical files"]);
    run_jj_in(&fixture.path, &["new"]);
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;

    let historical_ix = view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .graph
            .changes
            .iter()
            .position(|change| {
                !change.is_working_copy && change.description.trim() == "historical files"
            })
            .expect("historical multi-file change")
    });
    view.update_in(cx, |view, _, cx| view.select_change(historical_ix, cx));
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert_eq!(
            vm.diff_cache.loaded.len(),
            vm.files.as_ref().expect("historical files loaded").len(),
            "the selected and preloaded historical diffs should finish"
        );
        assert!(
            vm.diff_cache
                .loaded
                .values()
                .all(|loaded| loaded.review.is_none()),
            "historical diffs must not compute review snapshots"
        );
    });
}

#[gpui::test]
fn a_bookmark_diff_reverses_and_keeps_file_navigation(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    run_jj_in(&fixture.path, &["bookmark", "create", "feature"]);
    let (view, cx) = open_fixture(&fixture, cx);

    view.update_in(cx, |view, _, cx| {
        let head = view
            .view_model()
            .read(cx)
            .selected_change()
            .unwrap()
            .clone();
        let base = view
            .view_model()
            .read(cx)
            .graph
            .changes
            .iter()
            .position(|change| change.bookmarks.iter().any(|name| name == "main"))
            .expect("main change");
        view.select_change(base, cx);
        let action = view
            .build_change_menu(&head, cx)
            .into_iter()
            .find(|item| item.label == "Diff Bookmark")
            .expect("bookmark diff action")
            .action;
        view.dispatch_context_action(action, cx);
    });
    settle_visual(cx);

    for reversed in [true, false] {
        view.update_in(cx, |view, _, cx| {
            view.view_model()
                .update(cx, |vm, cx| vm.reverse_compare(cx));
        });
        settle_visual(cx);
        select_file(&view, "wip2.txt", cx);

        view.read_with(cx, |view, cx| {
            let vm = view.view_model().read(cx);
            let compare = vm.compare.as_ref().expect("bookmark comparison");
            let (from, to) = if reversed {
                ("feature", "main")
            } else {
                ("main", "feature")
            };
            assert_eq!(
                (compare.display.from.as_str(), compare.display.to.as_str()),
                (from, to)
            );
            assert_eq!(compare.display.title, "PR Diff");
            assert!(vm.error.is_none(), "reverse errored: {:?}", vm.error);
            assert_eq!(vm.shown.diff.as_ref().unwrap().path, "wip2.txt");
            let (old, new) = if reversed {
                ("wip 2\n", "")
            } else {
                ("", "wip 2\n")
            };
            assert_eq!(vm.shown.old_content.as_deref(), Some(old));
            assert_eq!(vm.shown.new_content.as_deref(), Some(new));
        });
    }
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
        vm.compare = Some(compare::CompareState {
            from_rev: "main".to_owned(),
            to_rev: "missing-change".to_owned(),
            source_change_id: None,
            target_change_id: Some("missing-change".to_owned()),
            display: compare::CompareDisplay {
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
        assert!(vm.shown.diff.is_none());
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

#[gpui::test]
fn code_line_spacing_change_keeps_the_diff_scrolled_to_the_same_line(cx: &mut TestAppContext) {
    use jayjay_gpui::app::config::AppConfig;
    use jayjay_gpui::app::theme::Theme;

    let fixture = LinearFixture::build();
    install_test_globals(cx);
    suppress_fs_watcher(cx);
    let view = cx.new(|cx| RepoWindow::new(fixture.path.clone(), cx));
    view.update(cx, |view, _| view.set_diff_scroll_offset_y(px(-180.)));

    cx.update(|cx| {
        let mut config = AppConfig::default();
        config.code_line_spacing = 2.;
        let spaced = cx.global::<Theme>().clone().with_typography(&config);
        cx.set_global(spaced);
    });
    view.read_with(cx, |view, _| {
        assert_eq!(
            view.diff_scroll_offset_y(),
            px(-360.),
            "18 px rows become 36 px"
        );
    });
}

#[gpui::test]
fn zooming_keeps_the_same_source_line_at_the_top_of_a_wrapped_diff(cx: &mut TestAppContext) {
    use gpui::size;
    use jayjay_gpui::app::actions::ZoomIn;

    let fixture = LinearFixture::build();
    let lines = 40;
    let content = format!("{}\n", "word ".repeat(40)).repeat(lines);
    fs::write(fixture.path.join("wide.txt"), content).unwrap();
    run_jj_in(&fixture.path, &["st"]);
    install_test_globals(cx);
    cx.update(menus::install);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    cx.simulate_resize(size(px(1000.), px(700.)));
    settle_visual(cx);
    select_file(&view, "wide.txt", cx);
    let rows_per_line =
        |view: &RepoWindow, cx: &gpui::App| view.diff_render_rows(cx).unwrap().rows.len() / lines;

    let before = view.read_with(cx, rows_per_line);
    assert!(before > 1, "lines must wrap");
    view.update(cx, |view, _| {
        view.set_diff_scroll_offset_y(px(-18. * (10 * before + 1) as f32))
    });
    cx.cx.update(|cx| {
        for _ in 0..6 {
            cx.dispatch_action(&ZoomIn);
        }
    });
    settle_visual(cx);

    let after = view.read_with(cx, rows_per_line);
    assert!(after > before, "a larger font wraps into more rows");
    let line_height = cx
        .cx
        .update(|cx| jayjay_gpui::app::config::current(cx).code_font_size())
        + 5.;
    let top_row = view.read_with(cx, |view, _| {
        -f32::from(view.diff_scroll_offset_y()) / line_height
    });
    assert_eq!(
        top_row as usize / after,
        10,
        "source line 10 stays at the top"
    );
}

#[gpui::test]
fn space_reveals_the_next_unreviewed_file_in_a_collapsed_offscreen_directory(
    cx: &mut TestAppContext,
) {
    let fixture = LinearFixture::build();
    run_jj_in(&fixture.path, &["new"]);
    for index in 0..40 {
        let dir = fixture.path.join(format!("dir-{index:02}"));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("file.txt"), "new file\n").unwrap();
    }
    run_jj_in(&fixture.path, &["st"]);
    install_test_globals(cx);
    cx.update(|cx| jayjay_gpui::app::config::update(cx, |c| c.diff.tree_file_list = true));
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    load_selected_change_files(&view, cx);
    settle_visual(cx);

    let target = "dir-39/file.txt";
    let (target_ix, target_row) = view.update_in(cx, |view, window, cx| {
        let vm = view.view_model().read(cx);
        let change_id = vm.selected_change().unwrap().change_id.id.clone();
        let files = vm.files.clone().unwrap();
        let first = files
            .iter()
            .position(|h| h.path == "dir-00/file.txt")
            .unwrap();
        let target_ix = files.iter().position(|h| h.path == target).unwrap();
        let paths = files.iter().map(|h| h.path.clone()).collect::<Vec<_>>();
        let tree = jayjay_core::file_tree::build_file_tree(&paths);
        let target_row = tree.iter().position(|entry| entry.path == target).unwrap();
        for (ix, hunk) in files.iter().enumerate() {
            if ix != first && ix != target_ix {
                view.toggle_reviewed(
                    change_id.clone(),
                    hunk.path.clone(),
                    hunk.review_identity.clone(),
                    cx,
                );
            }
        }
        view.select_file(first, cx);
        view.focus_handle(cx).focus(window, cx);
        (target_ix, target_row)
    });
    settle_visual(cx);
    let selector: &'static str = Box::leak(format!("tree-file-{target_row}").into_boxed_str());
    let viewport = cx.debug_bounds("files-tree").unwrap();
    let before = cx.debug_bounds(selector).unwrap();
    assert!(
        before.top() >= viewport.bottom(),
        "target starts below the viewport"
    );
    view.update_in(cx, |view, _, cx| view.toggle_dir("dir-39".to_owned(), cx));
    settle_visual(cx);
    assert!(
        cx.debug_bounds(selector).is_none(),
        "target directory is collapsed"
    );

    cx.simulate_keystrokes("space");
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        assert_eq!(view.view_model().read(cx).selected_file_ix, Some(target_ix));
    });
    let after = cx
        .debug_bounds(selector)
        .expect("selected file is expanded");
    let viewport = cx.debug_bounds("files-tree").unwrap();
    assert!(
        after.top() >= viewport.top() && after.bottom() <= viewport.bottom(),
        "selected file must be fully visible: {after:?} in {viewport:?}"
    );
}

#[gpui::test]
fn space_picks_the_next_unreviewed_file_in_tree_order(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    run_jj_in(&fixture.path, &["new"]);
    fs::write(fixture.path.join("a.txt"), "root\n").unwrap();
    fs::create_dir_all(fixture.path.join("b")).unwrap();
    fs::write(fixture.path.join("b/x.txt"), "x\n").unwrap();
    fs::write(fixture.path.join("b/y.txt"), "y\n").unwrap();
    run_jj_in(&fixture.path, &["st"]);
    install_test_globals(cx);
    cx.update(|cx| jayjay_gpui::app::config::update(cx, |c| c.diff.tree_file_list = true));
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    load_selected_change_files(&view, cx);
    settle_visual(cx);

    let ix = |view: &RepoWindow, cx: &gpui::App, path: &str| {
        let files = view.view_model().read(cx).files.clone().unwrap();
        files.iter().position(|h| h.path == path).unwrap()
    };
    view.update_in(cx, |view, window, cx| {
        let first = ix(view, cx, "b/x.txt");
        view.select_file(first, cx);
        view.focus_handle(cx).focus(window, cx);
    });
    cx.simulate_keystrokes("space");
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        assert_eq!(
            view.view_model().read(cx).selected_file_ix,
            Some(ix(view, cx, "b/y.txt"))
        );
    });
}
