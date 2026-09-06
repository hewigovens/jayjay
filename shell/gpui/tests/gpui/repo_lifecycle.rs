use std::fs;

use crate::harness::*;
use gpui::{AppContext, Modifiers, TestAppContext, VisualTestContext};
use jayjay_gpui::app::config;
use jayjay_gpui::repo::RepoWindow;
use jayjay_gpui::repo::view_model::RepoViewModel;
use jayjay_gpui::windows::repo_list::RepoListWindow;
use jj_test::{LinearFixture, run_jj_in};

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
    cx.update(|cx| config::update(cx, |cfg| cfg.onboarding.completed = false));
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
    install_test_globals(cx);
    let fixture = LinearFixture::build();
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
    fixture.add_tracked_working_copy_edits();
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
    fixture.add_tracked_working_copy_edits();
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
    install_test_globals(cx);
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
fn fs_change_refreshes_while_reviewing_working_copy(cx: &mut TestAppContext) {
    install_test_globals(cx);
    let fixture = LinearFixture::build();
    let vm = cx.new(|_| RepoViewModel::new(fixture.path.clone()));
    vm.update(cx, |vm, cx| vm.boot(cx));
    settle(cx);
    fs::write(fixture.path.join("late-edit.txt"), "refresh me\n")
        .expect("write late working-copy edit");

    vm.update(cx, |vm, cx| {
        assert!(
            vm.selected_change().is_some_and(|c| c.is_working_copy),
            "boot should select the working copy"
        );
        vm.handle_fs_event(cx);
        assert!(
            vm.loading.refreshing,
            "working-copy review should no longer suppress auto-refresh"
        );
    });
    settle(cx);

    vm.read_with(cx, |vm, _| {
        assert!(
            vm.files
                .as_ref()
                .is_some_and(|files| files.iter().any(|file| file.path == "late-edit.txt")),
            "auto-refresh should show the external edit"
        );
    });
}

#[gpui::test]
fn fs_event_mid_refresh_is_not_dropped(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let vm = cx.new(|_| RepoViewModel::new(fixture.path.clone()));

    vm.update(cx, |vm, cx| {
        vm.handle_fs_event(cx);
        assert!(vm.loading.refreshing, "first event should start a refresh");
        vm.handle_fs_event(cx);
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
fn selecting_a_change_resets_pr_state(cx: &mut TestAppContext) {
    install_test_globals(cx);
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
        vm.last_internal_mutation_at = Some(std::time::Instant::now());
        vm.handle_fs_event(cx);
    });

    vm.read_with(cx, |vm, _| {
        assert!(
            !vm.loading.refreshing,
            "FS echo within the mutation window must not refresh"
        );
    });
}

#[gpui::test]
fn suspended_fs_event_is_remembered_and_runs_when_the_gate_clears(cx: &mut TestAppContext) {
    install_test_globals(cx);
    let fixture = LinearFixture::build();
    let vm = cx.new(|_| RepoViewModel::new(fixture.path.clone()));
    vm.update(cx, |vm, cx| vm.boot(cx));
    settle(cx);

    vm.update(cx, |vm, cx| {
        vm.set_refresh_suspended(true, cx);
        vm.handle_fs_event(cx);
        assert!(!vm.loading.refreshing, "a suspended event must not refresh");
        assert!(
            vm.loading.pending_auto_refresh,
            "a suspended event must be remembered"
        );
        vm.set_refresh_suspended(false, cx);
        assert!(
            vm.loading.refreshing,
            "clearing the gate must run the owed refresh"
        );
    });
    settle(cx);
}

#[gpui::test]
fn overlay_opening_mid_refresh_defers_the_apply(cx: &mut TestAppContext) {
    install_test_globals(cx);
    let fixture = LinearFixture::build();
    let vm = cx.new(|_| RepoViewModel::new(fixture.path.clone()));
    vm.update(cx, |vm, cx| vm.boot(cx));
    settle(cx);

    vm.update(cx, |vm, cx| {
        vm.refresh(true, cx);
        assert!(vm.loading.refreshing);
        vm.set_refresh_suspended(true, cx);
    });
    settle(cx);

    vm.read_with(cx, |vm, _| {
        assert!(!vm.loading.refreshing, "the in-flight refresh completes");
        assert!(
            vm.loading.pending_auto_refresh,
            "its result must be deferred, not applied under the overlay"
        );
    });

    vm.update(cx, |vm, cx| vm.set_refresh_suspended(false, cx));
    settle(cx);
    vm.read_with(cx, |vm, _| {
        assert!(!vm.loading.pending_auto_refresh);
        assert!(!vm.loading.refreshing);
    });
}

#[gpui::test]
fn auto_refresh_keeps_the_selected_file(cx: &mut TestAppContext) {
    install_test_globals(cx);
    let fixture = LinearFixture::build();
    fixture.add_tracked_working_copy_edits();
    let vm = cx.new(|_| RepoViewModel::new(fixture.path.clone()));
    vm.update(cx, |vm, cx| vm.boot(cx));
    settle(cx);

    let file_ix = vm.read_with(cx, |vm, _| {
        let files = vm.files.as_ref().expect("boot loads the WC file list");
        assert!(files.len() > 1, "fixture needs at least two changed files");
        files.len() - 1
    });
    vm.update(cx, |vm, _| vm.selected_file_ix = Some(file_ix));
    let selected_path = vm.read_with(cx, |vm, _| vm.files.as_ref().unwrap()[file_ix].path.clone());

    vm.update(cx, |vm, cx| vm.refresh(true, cx));
    settle(cx);

    vm.read_with(cx, |vm, _| {
        let files = vm.files.as_ref().expect("files after refresh");
        let ix = vm.selected_file_ix.expect("a file stays selected");
        assert_eq!(
            files[ix].path, selected_path,
            "a background refresh must keep the user's place in the file column"
        );
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

    vm.update(cx, |vm, cx| {
        vm.can_load_more = true;
        vm.load_more(cx);
    });

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
fn an_operation_refreshes_the_workspace_list_while_reviewing_working_copy(cx: &mut TestAppContext) {
    install_test_globals(cx);
    let fixture = LinearFixture::build();
    let vm = cx.new(|_| RepoViewModel::new(fixture.path.clone()));
    vm.update(cx, |vm, cx| vm.boot(cx));
    settle(cx);
    let sibling = fixture
        .path
        .parent()
        .expect("fixture parent")
        .join("added-by-cli");
    run_jj_in(
        &fixture.path,
        &[
            "workspace",
            "add",
            "--name",
            "added-by-cli",
            sibling.to_str().expect("utf-8 workspace path"),
        ],
    );

    vm.update(cx, |vm, cx| {
        assert!(vm.selected_change().is_some_and(|c| c.is_working_copy));
        vm.handle_fs_event(cx);
    });
    settle(cx);

    vm.read_with(cx, |vm, _| {
        assert!(
            vm.graph.workspaces.iter().any(|w| w.name == "added-by-cli"),
            "the full refresh should update the workspace list"
        );
        assert!(vm.selected_change().is_some_and(|c| c.is_working_copy));
    });
}

#[gpui::test]
fn pathless_first_launch_finishes_onboarding_in_repo_list(cx: &mut TestAppContext) {
    install_test_globals(cx);
    cx.update(|cx| {
        config::update(cx, |cfg| cfg.onboarding.completed = false);
        RepoListWindow::open(cx);
    });
    let window = cx.windows()[0];
    let mut visual = VisualTestContext::from_window(window, cx);
    settle_visual(&mut visual);
    assert!(visual.debug_bounds("onboarding-pane").is_some());
    for _ in 0..2 {
        let next = visual.debug_bounds("onboarding-next").expect("Next button");
        visual.simulate_click(next.center(), Modifiers::default());
        settle_visual(&mut visual);
    }
    let finish = visual
        .debug_bounds("onboarding-finish")
        .expect("Get Started button");
    visual.simulate_click(finish.center(), Modifiers::default());
    settle_visual(&mut visual);
    assert!(visual.debug_bounds("onboarding-pane").is_none());
    assert!(visual.debug_bounds("repo-list-window").is_some());
    visual.cx.update(|cx| {
        assert!(config::current(cx).onboarding.completed);
        assert!(config::current(cx).recent_repos.is_empty());
        assert!(
            cx.windows()
                .iter()
                .all(|handle| handle.downcast::<RepoWindow>().is_none())
        );
    });
    window
        .downcast::<RepoListWindow>()
        .unwrap()
        .update(&mut visual, |_, window, _| window.remove_window())
        .unwrap();
    visual.cx.update(RepoListWindow::open);
    let window = visual.cx.windows()[0];
    let mut reopened = VisualTestContext::from_window(window, &visual.cx);
    settle_visual(&mut reopened);
    assert!(reopened.debug_bounds("onboarding-pane").is_none());
    assert!(reopened.debug_bounds("repo-list-window").is_some());
}
