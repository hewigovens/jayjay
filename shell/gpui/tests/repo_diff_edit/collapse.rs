use std::fs;

use gpui::TestAppContext;
use jj_test::{FormatFixture, LinearFixture, run_jj_in};

use super::fixtures::*;
use super::harness::*;

fn big_file_contents(lines: usize) -> String {
    (0..lines).map(|i| format!("line {i}\n")).collect()
}

fn one_big_working_copy_fixture(lines: usize, extra_small_files: usize) -> LinearFixture {
    let fixture = LinearFixture::build();
    run_jj_in(&fixture.path, &["new"]);
    fs::write(fixture.path.join("big.txt"), big_file_contents(lines)).unwrap();
    for ix in 0..extra_small_files {
        fs::write(fixture.path.join(format!("small{ix}.txt")), "a\nb\n").unwrap();
    }
    run_jj_in(&fixture.path, &["st"]);
    fixture
}

#[gpui::test]
fn toggling_collapse_hides_and_restores_line_rows_preserving_selection(cx: &mut TestAppContext) {
    let fixture = two_file_edits_fixture();
    let (view, cx) = open_fixture(&fixture, cx);
    select_file(&view, "edit.txt", cx);
    view.update_in(cx, |view, _, cx| view.enter_diff_edit(cx));
    settle_visual(cx);

    let (rows_before, selected) = view.update_in(cx, |view, _, cx| {
        view.toggle_diff_edit_file("edit.txt", cx);
        (
            view.diff_edit_line_rows("edit.txt", cx),
            view.diff_edit_selected("edit.txt"),
        )
    });
    assert!(rows_before > 0, "an expanded file lists its line rows");
    assert!(!selected.is_empty(), "the whole-file selection is recorded");

    view.update_in(cx, |view, _, cx| {
        view.toggle_diff_edit_collapse("edit.txt", cx)
    });
    view.update_in(cx, |view, _, cx| {
        assert_eq!(
            view.diff_edit_line_rows("edit.txt", cx),
            0,
            "a collapsed file drops its line rows"
        );
        assert_eq!(
            view.diff_edit_selected("edit.txt"),
            selected,
            "collapse leaves the selection intact"
        );
    });

    view.update_in(cx, |view, _, cx| {
        view.toggle_diff_edit_collapse("edit.txt", cx)
    });
    view.update_in(cx, |view, _, cx| {
        assert_eq!(
            view.diff_edit_line_rows("edit.txt", cx),
            rows_before,
            "expanding restores every line row"
        );
        assert_eq!(
            view.diff_edit_selected("edit.txt"),
            selected,
            "expand leaves the selection intact"
        );
    });
}

#[gpui::test]
fn collapse_all_and_expand_all_toggle_every_file(cx: &mut TestAppContext) {
    let fixture = two_file_edits_fixture();
    let (view, cx) = open_fixture(&fixture, cx);
    view.update_in(cx, |view, _, cx| view.enter_diff_edit(cx));
    settle_visual(cx);

    let expanded = view.update_in(cx, |view, _, cx| {
        (
            view.diff_edit_line_rows("edit.txt", cx),
            view.diff_edit_line_rows("untouched.txt", cx),
        )
    });
    assert!(
        expanded.0 > 0 && expanded.1 > 0,
        "both files start expanded"
    );

    view.update_in(cx, |view, _, cx| view.collapse_all_diff_edit(cx));
    view.update_in(cx, |view, _, cx| {
        assert_eq!(view.diff_edit_line_rows("edit.txt", cx), 0);
        assert_eq!(view.diff_edit_line_rows("untouched.txt", cx), 0);
        assert!(view.diff_edit_collapsed("edit.txt"));
        assert!(view.diff_edit_collapsed("untouched.txt"));
    });

    view.update_in(cx, |view, _, cx| view.expand_all_diff_edit(cx));
    view.update_in(cx, |view, _, cx| {
        assert_eq!(view.diff_edit_line_rows("edit.txt", cx), expanded.0);
        assert_eq!(view.diff_edit_line_rows("untouched.txt", cx), expanded.1);
        assert!(!view.diff_edit_collapsed("edit.txt"));
        assert!(!view.diff_edit_collapsed("untouched.txt"));
    });
}

#[gpui::test]
fn collapse_all_keystroke_collapses_every_file(cx: &mut TestAppContext) {
    let fixture = two_file_edits_fixture();
    let (view, cx) = open_fixture(&fixture, cx);
    view.update_in(cx, |view, _, cx| view.enter_diff_edit(cx));
    settle_visual(cx);

    cx.simulate_keystrokes(&format!("{}-alt-c", jayjay_gpui::platform::MOD_KEY));
    settle_visual(cx);
    assert!(view.read_with(cx, |view, _| view.diff_edit_collapsed("edit.txt")));

    cx.simulate_keystrokes(&format!("{}-alt-e", jayjay_gpui::platform::MOD_KEY));
    settle_visual(cx);
    assert!(view.read_with(cx, |view, _| !view.diff_edit_collapsed("edit.txt")));
}

#[gpui::test]
fn auto_collapse_hides_oversized_file_and_leaves_small_ones(cx: &mut TestAppContext) {
    let fixture = one_big_working_copy_fixture(400, 30);
    let (view, cx) = open_fixture(&fixture, cx);
    view.update_in(cx, |view, _, cx| view.enter_diff_edit(cx));
    settle_visual(cx);

    view.update_in(cx, |view, _, _| {
        assert!(view.diff_edit_stats_ready(), "per-file stats arrive");
        assert!(
            view.diff_edit_collapsed("big.txt"),
            "the oversized file starts collapsed"
        );
        assert!(!view.diff_edit_collapsed("small0.txt"));
        assert!(!view.diff_edit_collapsed("small1.txt"));
    });
}

#[gpui::test]
fn auto_collapse_skips_diffs_at_the_file_cap(cx: &mut TestAppContext) {
    let fixture = one_big_working_copy_fixture(1100, 28);
    let (view, cx) = open_fixture(&fixture, cx);
    view.update_in(cx, |view, _, cx| view.enter_diff_edit(cx));
    settle_visual(cx);

    view.update_in(cx, |view, _, _| {
        assert!(view.diff_edit_stats_ready());
        assert!(
            !view.diff_edit_collapsed("big.txt"),
            "at or under the file cap nothing auto-collapses even when the total is large"
        );
        assert!(!view.diff_edit_collapsed("small0.txt"));
    });
}

#[gpui::test]
fn large_diff_seeds_collapse_before_per_file_stats_arrive(cx: &mut TestAppContext) {
    let fixture = one_big_working_copy_fixture(1100, 30);
    let (view, cx) = open_fixture(&fixture, cx);
    settle_visual(cx);

    view.update_in(cx, |view, _, cx| {
        view.enter_diff_edit(cx);
        assert!(
            !view.diff_edit_stats_ready(),
            "per-file stats cannot have arrived inside the entry update"
        );
        assert!(
            view.diff_edit_collapsed("big.txt"),
            "the whole-change stats seed collapse at entry"
        );
        assert!(view.diff_edit_collapsed("small0.txt"));
        assert!(view.diff_edit_collapsed("small1.txt"));
    });

    settle_visual(cx);
    view.update_in(cx, |view, _, _| {
        assert!(view.diff_edit_stats_ready());
        assert!(
            view.diff_edit_collapsed("big.txt") && view.diff_edit_collapsed("small0.txt"),
            "the per-file refinement never expands seeded folds"
        );
    });
}

#[gpui::test]
fn cached_rich_preview_does_not_replace_the_raw_card(cx: &mut TestAppContext) {
    let fixture = FormatFixture::build();
    let (view, cx) = open_repo(fixture.path.clone(), cx);
    select_file(&view, FormatFixture::NOTEBOOK, cx);
    view.update_in(cx, |view, _, cx| view.toggle_projection_rich_preview(cx));
    settle_visual(cx);

    view.update_in(cx, |view, _, cx| view.enter_diff_edit(cx));
    settle_visual(cx);

    let (notebook_lines, plist_lines) = view.update_in(cx, |view, _, cx| {
        (
            view.diff_edit_preview_line_count(FormatFixture::NOTEBOOK, cx),
            view.diff_edit_preview_line_count(FormatFixture::PLIST, cx),
        )
    });
    let raw_lines = fs::read_to_string(fixture.path.join(FormatFixture::NOTEBOOK))
        .unwrap()
        .lines()
        .count();
    assert_eq!(
        notebook_lines, raw_lines,
        "the card shows the raw source rows its stats describe, not the cached processed preview"
    );
    let xml_lines = jayjay_core::Repo::open(&fixture.path)
        .expect("open repo")
        .show_file("@", FormatFixture::PLIST)
        .expect("projected plist")
        .new
        .content
        .expect("projected content")
        .lines()
        .count();
    assert_eq!(
        plist_lines, xml_lines,
        "the auto-open plist card resolves its processed rows; the cache stores them under the virtual path a path scan can never match"
    );
}

#[gpui::test]
fn whitespace_toggle_reloads_diff_edit_files(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    fs::write(fixture.path.join("ws.txt"), "a b\nkeep\n").unwrap();
    run_jj_in(&fixture.path, &["describe", "-m", "base"]);
    run_jj_in(&fixture.path, &["new"]);
    fs::write(fixture.path.join("ws.txt"), "a  b\nkeep\n").unwrap();
    run_jj_in(&fixture.path, &["st"]);

    let (view, cx) = open_fixture(&fixture, cx);
    view.update_in(cx, |view, _, cx| view.enter_diff_edit(cx));
    settle_visual(cx);

    view.update_in(cx, |view, _, cx| {
        view.toggle_diff_edit_file("ws.txt", cx);
        assert_eq!(
            view.diff_edit_file_state("ws.txt"),
            jayjay_gpui::repo::window::DiffEditCheckboxState::All,
            "the whitespace-only edit is selectable while the mode is exact"
        );
        view.view_model()
            .update(cx, |vm, cx| vm.toggle_ignore_whitespace(cx));
    });
    settle_visual(cx);

    view.update_in(cx, |view, _, cx| {
        assert_eq!(
            view.diff_edit_file_state("ws.txt"),
            jayjay_gpui::repo::window::DiffEditCheckboxState::None,
            "the mode change reloads the card, whose diff now has no changed lines"
        );
        assert!(
            view.diff_edit_selected("ws.txt").is_empty(),
            "old-mode row indices are cleared with the loaded diffs"
        );
        assert!(
            view.diff_edit_line_rows("ws.txt", cx) > 0,
            "the reload preloads every card instead of leaving it loading"
        );
    });
}

#[gpui::test]
fn whitespace_toggle_replaces_stale_folds_with_fresh_stats(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let base: String = (0..400).map(|i| format!("line {i} x\n")).collect();
    fs::write(fixture.path.join("big.txt"), base).unwrap();
    run_jj_in(&fixture.path, &["describe", "-m", "base"]);
    run_jj_in(&fixture.path, &["new"]);
    let edited: String = (0..400).map(|i| format!("line {i}  x\n")).collect();
    fs::write(fixture.path.join("big.txt"), edited).unwrap();
    for ix in 0..30 {
        fs::write(fixture.path.join(format!("small{ix}.txt")), "a\nb\n").unwrap();
    }
    run_jj_in(&fixture.path, &["st"]);

    let (view, cx) = open_fixture(&fixture, cx);
    view.update_in(cx, |view, _, cx| view.enter_diff_edit(cx));
    settle_visual(cx);

    view.update_in(cx, |view, _, cx| {
        assert!(
            view.diff_edit_collapsed("big.txt"),
            "exact mode folds the 400-line whitespace churn"
        );
        view.view_model()
            .update(cx, |vm, cx| vm.toggle_ignore_whitespace(cx));
    });
    settle_visual(cx);

    view.update_in(cx, |view, _, _| {
        assert!(
            view.diff_edit_stats_ready(),
            "the reset respawns per-file stats"
        );
        assert!(
            !view.diff_edit_collapsed("big.txt"),
            "fresh ignore-whitespace stats replace the stale exact-mode folds instead of unioning"
        );
    });
}

#[gpui::test]
fn stats_pending_entry_collapses_provisionally_then_refines(cx: &mut TestAppContext) {
    let fixture = one_big_working_copy_fixture(400, 30);
    let (view, cx) = open_fixture(&fixture, cx);
    settle_visual(cx);

    view.update_in(cx, |view, _, cx| {
        view.view_model().update(cx, |vm, _| vm.change_stats = None);
        view.enter_diff_edit(cx);
        assert!(
            view.diff_edit_collapsed("big.txt") && view.diff_edit_collapsed("small0.txt"),
            "without whole-change stats a multi-file diff starts provisionally collapsed"
        );
    });

    settle_visual(cx);
    view.update_in(cx, |view, _, _| {
        assert!(view.diff_edit_stats_ready());
        assert!(
            view.diff_edit_collapsed("big.txt"),
            "the per-file policy keeps the oversized file folded"
        );
        assert!(
            !view.diff_edit_collapsed("small0.txt"),
            "the provisional seed is replaced, so small files expand"
        );
    });
}

#[gpui::test]
fn queued_stats_read_the_captured_commit_not_an_amended_replacement(cx: &mut TestAppContext) {
    let fixture = one_big_working_copy_fixture(400, 30);
    let (view, cx) = open_fixture(&fixture, cx);
    view.update_in(cx, |view, _, cx| view.enter_diff_edit(cx));

    // Amend through the shared core handle while the stats task is still queued and the vm lags; the query must stay pinned to the entry commit.
    fs::write(fixture.path.join("big.txt"), "tiny\n").unwrap();
    view.update_in(cx, |view, _, cx| {
        let repo = view.view_model().read(cx).repo.clone().expect("repo");
        repo.refresh_working_copy().expect("refresh");
    });
    settle_visual(cx);

    view.update_in(cx, |view, _, _| {
        assert!(view.diff_edit_stats_ready());
        assert!(
            view.diff_edit_collapsed("big.txt"),
            "stats describe the on-screen commit's 400-line file, not the amended replacement"
        );
    });
}

#[gpui::test]
fn refined_stats_reopen_folds_seeded_by_an_inflated_aggregate(cx: &mut TestAppContext) {
    let fixture = one_big_working_copy_fixture(2, 30);
    let (view, cx) = open_fixture(&fixture, cx);
    settle_visual(cx);

    view.update_in(cx, |view, _, cx| {
        // Models a >32 MiB text file: jj's aggregate counts its real lines while the displayed placeholder counts zero.
        view.view_model().update(cx, |vm, _| {
            vm.change_stats = Some(jayjay_core::DiffStats {
                files_changed: 31,
                insertions: 5000,
                deletions: 5000,
            });
        });
        view.enter_diff_edit(cx);
        assert!(
            view.diff_edit_collapsed("big.txt") && view.diff_edit_collapsed("small0.txt"),
            "the inflated aggregate seeds every card collapsed"
        );
    });

    settle_visual(cx);
    view.update_in(cx, |view, _, _| {
        assert!(view.diff_edit_stats_ready());
        assert!(
            !view.diff_edit_collapsed("big.txt") && !view.diff_edit_collapsed("small0.txt"),
            "precise per-file stats replace the misleading aggregate seed and reopen the cards"
        );
    });
}

#[gpui::test]
fn manual_toggle_before_stats_suppresses_auto_collapse(cx: &mut TestAppContext) {
    let fixture = one_big_working_copy_fixture(400, 30);
    let (view, cx) = open_fixture(&fixture, cx);
    view.update_in(cx, |view, _, cx| {
        view.enter_diff_edit(cx);
        view.toggle_diff_edit_collapse("small0.txt", cx);
    });
    settle_visual(cx);

    view.update_in(cx, |view, _, _| {
        assert!(
            view.diff_edit_stats_ready(),
            "stats are still stored for badges"
        );
        assert!(
            !view.diff_edit_collapsed("big.txt"),
            "a manual toggle before stats arrive blocks auto-collapse"
        );
        assert!(
            view.diff_edit_collapsed("small0.txt"),
            "the user's manual collapse is preserved"
        );
    });
}
