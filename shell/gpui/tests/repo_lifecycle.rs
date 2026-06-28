mod support;

use std::fs;

use gpui::{AppContext, Focusable, KeyBinding, Modifiers, TestAppContext, VisualTestContext};
use jayjay_gpui::app::actions::{CloseWindow, Dismiss};
use jayjay_gpui::app::config;
use jayjay_gpui::repo::RepoWindow;
use jayjay_gpui::repo::view_model::RepoViewModel;
use jj_test::LinearFixture;
use support::*;

#[gpui::test]
fn opens_linear_fixture_with_working_copy_selected(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let vm = cx.new(|_| RepoViewModel::new(fixture.path.clone()));

    vm.read_with(cx, |vm, _| {
        assert!(vm.error.is_none(), "open errored: {:?}", vm.error);
        assert!(vm.repo.is_some(), "repo handle should be populated");
        assert!(
            vm.graph.entries.len() >= 4,
            "linear fixture should expose at least 4 changes (initial, hello, feature, wc), got {}",
            vm.graph.entries.len()
        );
        let selected_ix = vm.selected.expect("working copy should be selected");
        let selected = &vm.graph.entries[selected_ix].change;
        assert!(selected.is_working_copy);
    });
}

#[gpui::test]
fn invalid_repo_can_be_initialized(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let repo_path = fixture.path.parent().unwrap().join("empty-repo");
    fs::create_dir(&repo_path).expect("create empty repo dir");

    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(repo_path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert!(vm.repo.is_none());
        assert!(vm.error.is_some());
        assert!(
            !view.fs_watcher_armed(),
            "a non-repo directory should not arm the FS watcher"
        );
    });
    view.update_in(cx, |view, _, cx| {
        view.view_model()
            .update(cx, |vm, cx| vm.initialize_repo(cx))
            .detach();
    });
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert!(vm.repo.is_some(), "repo should open after jj git init");
        assert!(vm.error.is_none(), "init/open errored: {:?}", vm.error);
        assert!(
            view.fs_watcher_armed(),
            "the FS auto-refresh watcher must arm after in-app jj git init"
        );
    });
    assert!(repo_path.join(".jj").exists());
}

#[gpui::test]
fn startup_onboarding_delays_repo_open_until_finished(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();

    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new_with_onboarding(fixture.path, cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);

    assert!(cx.debug_bounds("onboarding-next").is_some());
    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert!(vm.repo.is_none(), "onboarding should delay repo open");
        assert!(vm.error.is_none());
    });

    let next = cx.debug_bounds("onboarding-next").expect("Next button");
    cx.simulate_click(next.center(), Modifiers::default());
    settle_visual(cx);
    let next = cx.debug_bounds("onboarding-next").expect("Next button");
    cx.simulate_click(next.center(), Modifiers::default());
    settle_visual(cx);
    let finish = cx
        .debug_bounds("onboarding-finish")
        .expect("Get Started button");
    cx.simulate_click(finish.center(), Modifiers::default());
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert!(vm.repo.is_some(), "repo should open after onboarding");
        assert!(vm.error.is_none(), "open errored: {:?}", vm.error);
    });
    let completed = cx.cx.update(|cx| config::current(cx).onboarding.completed);
    assert!(completed);
}

#[gpui::test]
fn repo_opens_off_the_main_thread(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    // `opening` returns immediately with no repo so window-open never blocks on Repo::open.
    let vm = cx.new(|cx| {
        let mut vm = RepoViewModel::opening(fixture.path.clone());
        vm.open_async(cx);
        vm
    });

    vm.read_with(cx, |vm, _| {
        assert!(
            vm.repo.is_none(),
            "open must not block: repo is not loaded yet"
        );
        assert!(vm.error.is_none(), "no error while opening");
        assert!(
            vm.loading.refreshing,
            "the loading state drives the opening pane"
        );
    });

    settle(cx);

    vm.read_with(cx, |vm, _| {
        assert!(
            vm.repo.is_some(),
            "repo should be loaded after the async open settles"
        );
        assert!(vm.error.is_none(), "open errored: {:?}", vm.error);
        assert!(
            !vm.loading.refreshing,
            "loading clears once open + boot finish"
        );
        assert!(
            vm.selected_change().is_some_and(|c| c.is_working_copy),
            "open selects the working copy like the synchronous constructor did"
        );
        assert!(
            vm.graph.entries.len() >= 4,
            "the initial graph loads with the repo"
        );
    });
}

#[gpui::test]
fn manual_refresh_snapshots_working_copy(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let vm = cx.new(|_| RepoViewModel::new(fixture.path.clone()));

    fs::write(
        fixture.path.join("wip1.txt"),
        "wip 1\nchanged after gpui refresh\n",
    )
    .expect("edit working copy file");

    vm.update(cx, |vm, cx| vm.refresh(false, cx));
    settle(cx);

    vm.read_with(cx, |vm, _| {
        assert!(vm.error.is_none(), "refresh errored: {:?}", vm.error);
        let hunk = vm
            .files
            .as_ref()
            .expect("refreshed working copy files")
            .iter()
            .find(|hunk| hunk.path == "wip1.txt")
            .expect("refreshed wip1 hunk");
        assert!(
            !hunk.review_identity.is_empty(),
            "manual refresh should snapshot working copy edits"
        );
    });
}

#[gpui::test]
fn refresh_updates_status_bar_snapshot(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    add_tracked_working_copy_edits(&fixture);
    let vm = cx.new(|_| RepoViewModel::new(fixture.path.clone()));

    vm.update(cx, |vm, cx| vm.refresh(false, cx));
    settle(cx);

    vm.read_with(cx, |vm, _| {
        let stats = vm
            .working_copy_stats
            .as_ref()
            .expect("working-copy stats should load during refresh");
        assert!(stats.files_changed > 0, "working copy should be dirty");
        assert!(
            !vm.current_operation_description.trim().is_empty(),
            "status bar should have the current operation description"
        );
    });
}

#[gpui::test]
fn status_bar_renders_swiftui_style_items(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    add_tracked_working_copy_edits(&fixture);
    install_test_globals(cx);
    let (_view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);

    assert!(cx.debug_bounds("status-path").is_some());
    assert!(cx.debug_bounds("status-wc-stat").is_some());
    assert!(cx.debug_bounds("status-last-op").is_some());
    assert!(cx.debug_bounds("status-changes").is_some());
}

#[gpui::test]
fn boot_snapshots_small_working_copy(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let vm = cx.new(|_| RepoViewModel::new(fixture.path.clone()));

    // Edit made before "open" — the FS watcher would miss it, so boot must snapshot.
    fs::write(fixture.path.join("wip1.txt"), "wip 1\nedited before boot\n")
        .expect("edit working copy file");

    vm.update(cx, |vm, cx| vm.boot(cx));
    settle(cx);

    vm.read_with(cx, |vm, _| {
        assert!(vm.error.is_none(), "boot errored: {:?}", vm.error);
        let hunk = vm
            .files
            .as_ref()
            .expect("working copy files after boot")
            .iter()
            .find(|hunk| hunk.path == "wip1.txt")
            .expect("wip1 hunk after boot");
        assert!(
            !hunk.review_identity.is_empty(),
            "boot should snapshot pre-open working copy edits on a small repo"
        );
    });
}

#[gpui::test]
fn fs_change_badges_while_reviewing_working_copy(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let vm = cx.new(|_| RepoViewModel::new(fixture.path.clone()));
    vm.update(cx, |vm, cx| vm.boot(cx));
    settle(cx);

    // Reviewing the WC in an active window → badge, don't reload the diff.
    vm.update(cx, |vm, cx| {
        vm.is_repo_window_active = true;
        assert!(
            vm.selected_change().is_some_and(|c| c.is_working_copy),
            "boot should select the working copy"
        );
        vm.handle_working_copy_change(cx);
    });

    vm.read_with(cx, |vm, _| {
        assert!(vm.loading.wc_changes, "reviewing the WC should badge");
        assert!(
            !vm.loading.refreshing,
            "badge path must not start a refresh"
        );
    });
}

#[gpui::test]
fn fs_event_mid_refresh_is_not_dropped(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let vm = cx.new(|_| RepoViewModel::new(fixture.path.clone()));

    // Deselect so the refresh path (not the badge path) handles the event.
    vm.update(cx, |vm, cx| {
        vm.selected = None;
        // First FS event starts a refresh; its snapshot is in flight.
        vm.handle_working_copy_change(cx);
        assert!(vm.loading.refreshing, "first event should start a refresh");
        // A second FS event arrives before the snapshot completes.
        vm.handle_working_copy_change(cx);
    });

    vm.read_with(cx, |vm, _| {
        assert!(
            vm.loading.pending_auto_refresh,
            "an event arriving mid-refresh must be recorded, not dropped"
        );
    });

    settle(cx);

    vm.read_with(cx, |vm, _| {
        assert!(!vm.loading.refreshing, "refresh should finish");
        assert!(
            !vm.loading.pending_auto_refresh,
            "the recorded event must be consumed by a re-run"
        );
        assert!(vm.error.is_none(), "re-run errored: {:?}", vm.error);
    });
}

#[gpui::test]
fn badge_set_mid_refresh_survives_completion(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let vm = cx.new(|_| RepoViewModel::new(fixture.path.clone()));
    vm.update(cx, |vm, cx| vm.boot(cx));
    settle(cx);

    vm.update(cx, |vm, cx| {
        vm.is_repo_window_active = true;
        assert!(
            vm.selected_change().is_some_and(|c| c.is_working_copy),
            "boot should select the working copy"
        );
        // A refresh is in flight (e.g. a manual refresh) when the user saves again.
        vm.refresh(false, cx);
        assert!(vm.loading.refreshing, "manual refresh should be in flight");
        vm.handle_working_copy_change(cx);
        assert!(vm.loading.wc_changes, "reviewing the WC should badge");
    });

    settle(cx);

    vm.read_with(cx, |vm, _| {
        assert!(
            vm.loading.wc_changes,
            "a badge set mid-refresh must survive the in-flight completion"
        );
        assert!(!vm.loading.refreshing, "refresh should finish");
        assert!(!vm.loading.pending_auto_refresh);
    });
}

#[gpui::test]
fn selecting_another_change_keeps_the_staleness_badge(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let vm = cx.new(|_| RepoViewModel::new(fixture.path.clone()));
    vm.update(cx, |vm, cx| vm.boot(cx));
    settle(cx);

    // Reviewing the WC: an on-disk edit badges instead of reloading.
    vm.update(cx, |vm, cx| {
        vm.is_repo_window_active = true;
        vm.handle_working_copy_change(cx);
    });
    let other = vm.read_with(cx, |vm, _| {
        assert!(vm.loading.wc_changes, "WC edit should badge");
        vm.graph
            .changes
            .iter()
            .position(|c| !c.is_working_copy)
            .expect("fixture has a non-WC change")
    });

    // Selecting a different change must not silently clear the staleness badge.
    vm.update(cx, |vm, cx| vm.select_change(other, cx));
    settle(cx);

    vm.read_with(cx, |vm, _| {
        assert!(
            vm.loading.wc_changes,
            "selecting another change must keep the WC staleness badge (no re-snapshot happened)"
        );
    });
}

#[gpui::test]
fn selecting_badged_working_copy_refreshes_instead_of_showing_stale(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let vm = cx.new(|_| RepoViewModel::new(fixture.path.clone()));
    vm.update(cx, |vm, cx| vm.boot(cx));
    settle(cx);

    let wc_ix = vm.read_with(cx, |vm, _| {
        vm.graph
            .changes
            .iter()
            .position(|c| c.is_working_copy)
            .expect("fixture has a working copy")
    });

    // Edit on disk, then badge while reviewing the WC.
    fs::write(fixture.path.join("wip1.txt"), "wip 1\nstale-on-disk\n")
        .expect("edit working copy file");
    vm.update(cx, |vm, cx| {
        vm.is_repo_window_active = true;
        vm.handle_working_copy_change(cx);
        assert!(vm.loading.wc_changes, "WC edit should badge");
    });

    // Re-selecting the badged WC row must re-snapshot rather than render the stale snapshot.
    vm.update(cx, |vm, cx| vm.select_change(wc_ix, cx));
    vm.read_with(cx, |vm, _| {
        assert!(
            vm.loading.refreshing,
            "selecting the badged WC must trigger a refresh"
        );
    });
    settle(cx);

    vm.read_with(cx, |vm, _| {
        assert!(
            !vm.loading.wc_changes,
            "the refresh completion should clear the badge"
        );
        let hunk = vm
            .files
            .as_ref()
            .expect("files after refresh")
            .iter()
            .find(|h| h.path == "wip1.txt")
            .expect("wip1 hunk after re-snapshot");
        assert!(
            !hunk.review_identity.is_empty(),
            "the WC edit must be snapshotted, not hidden behind a cleared badge"
        );
    });
}

#[gpui::test]
fn selecting_a_change_resets_pr_state(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let vm = cx.new(|_| RepoViewModel::new(fixture.path.clone()));
    vm.update(cx, |vm, cx| vm.boot(cx));
    settle(cx);

    let (gen_before, target) = vm.read_with(cx, |vm, _| {
        let target = vm
            .graph
            .changes
            .iter()
            .position(|c| !c.is_working_copy)
            .expect("fixture has a non-WC change");
        (vm.loading.pr_gen, target)
    });

    // Selecting a change clears stale PR info and bumps the generation so a late fetch is dropped.
    vm.update(cx, |vm, cx| vm.select_change(target, cx));
    vm.read_with(cx, |vm, _| {
        assert!(vm.pr_info.is_none(), "selection should reset pr_info");
        assert!(
            vm.loading.pr_gen > gen_before,
            "selection must bump pr_gen to invalidate the prior selection's in-flight fetch"
        );
    });
}

#[gpui::test]
fn fs_change_after_own_mutation_is_ignored(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let vm = cx.new(|_| RepoViewModel::new(fixture.path.clone()));

    vm.update(cx, |vm, cx| {
        // Deselect so only the mutation-echo guard, not the badge path, can suppress the refresh.
        vm.last_internal_mutation_at = Some(std::time::Instant::now());
        vm.selected = None;
        vm.handle_working_copy_change(cx);
    });

    vm.read_with(cx, |vm, _| {
        assert!(
            !vm.loading.refreshing,
            "FS echo within the mutation window must not refresh"
        );
        assert!(!vm.loading.wc_changes);
    });
}

#[gpui::test]
fn overlapping_refreshes_keep_the_gate_until_all_finish(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let vm = cx.new(|_| RepoViewModel::new(fixture.path.clone()));

    // Two manual refreshes overlap (manual refreshes don't bail on the re-entry gate).
    vm.update(cx, |vm, cx| {
        vm.refresh(false, cx);
        vm.refresh(false, cx);
        assert!(vm.loading.refreshing, "refresh in flight");
        assert_eq!(vm.loading.in_flight, 2, "both refreshes are counted");
    });

    settle(cx);

    vm.read_with(cx, |vm, _| {
        assert_eq!(
            vm.loading.in_flight, 0,
            "every overlapping refresh must decrement the gate"
        );
        assert!(
            !vm.loading.refreshing,
            "the gate clears only after all overlapping refreshes finish"
        );
        assert!(vm.error.is_none(), "refresh errored: {:?}", vm.error);
    });
}

#[gpui::test]
fn load_more_shows_refresh_indicator(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let vm = cx.new(|_| RepoViewModel::new(fixture.path.clone()));

    vm.update(cx, |vm, cx| vm.load_more(cx));

    vm.read_with(cx, |vm, _| {
        assert!(vm.loading.more);
        assert!(vm.loading.refreshing);
        assert!(vm.loading.refresh_indicator);
    });

    settle(cx);

    vm.read_with(cx, |vm, _| {
        assert!(!vm.loading.more);
        assert!(vm.error.is_none(), "load more errored: {:?}", vm.error);
    });
}

#[gpui::test]
fn cmd_w_closes_repo_window_when_no_overlay_is_open(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    install_test_globals(cx);
    cx.update(|cx| {
        cx.bind_keys([
            KeyBinding::new("cmd-w", CloseWindow, None),
            KeyBinding::new("escape", Dismiss, None),
        ]);
    });
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);
    view.update_in(cx, |view, window, cx| {
        view.focus_handle(cx).focus(window, cx);
    });

    let windows_before = cx.cx.windows().len();
    assert_eq!(windows_before, 1, "the repo window should be open");

    // Open the find overlay, then escape: dismisses the overlay, not the window.
    view.update_in(cx, |view, _, cx| view.open_find(cx));
    cx.simulate_keystrokes("escape");
    view.read_with(cx, |view, _| {
        assert!(view.find_query_text().is_none(), "escape should close find");
    });
    assert_eq!(
        cx.cx.windows().len(),
        1,
        "escape must not close the repo window"
    );

    // The close-window action closes the window when no overlay is open.
    cx.simulate_keystrokes("cmd-w");
    assert_eq!(
        cx.cx.windows().len(),
        0,
        "cmd-w must close the repo window when no overlay is open"
    );
}

#[gpui::test]
fn cmd_w_dismisses_open_overlay_before_closing_repo_window(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    install_test_globals(cx);
    cx.update(|cx| {
        cx.bind_keys([KeyBinding::new("cmd-w", CloseWindow, None)]);
    });
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);
    view.update_in(cx, |view, window, cx| {
        view.focus_handle(cx).focus(window, cx);
        view.open_find(cx);
    });

    // First close-window action closes the find overlay; the window stays open.
    cx.simulate_keystrokes("cmd-w");
    view.read_with(cx, |view, _| {
        assert!(view.find_query_text().is_none(), "cmd-w should close find");
    });
    assert_eq!(
        cx.cx.windows().len(),
        1,
        "cmd-w with an overlay open must dismiss the overlay, not the window"
    );

    // Second close-window action now closes the window.
    cx.simulate_keystrokes("cmd-w");
    assert_eq!(cx.cx.windows().len(), 0, "second cmd-w closes the window");
}
