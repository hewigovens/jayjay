use crate::harness::*;
use gpui::{Entity, Modifiers, MouseButton, TestAppContext, VisualContext, VisualTestContext};
use jayjay_gpui::app::actions::{OpenBookmarkManager, OpenOperationLog, OpenOverview};
use jayjay_gpui::app::config;
use jayjay_gpui::repo::{RepoWindow, open_repo_window};
use jj_test::{LinearFixture, run_jj_in};
use std::path::Path;

fn open_switcher(cx: &mut VisualTestContext) {
    let title = cx
        .debug_bounds("repo-switcher-button")
        .expect("repository title picker button");
    cx.simulate_click(title.center(), Modifiers::default());
    settle_visual(cx);
}

fn pick_workspace(name: &str, modifiers: Modifiers, cx: &mut VisualTestContext) {
    open_switcher(cx);
    let row = cx
        .debug_bounds(selector(format!("repo-switcher-workspace-{name}")))
        .expect("workspace row");
    cx.simulate_click(row.center(), modifiers);
    settle_visual(cx);
}

fn summary(view: &Entity<RepoWindow>, cx: &VisualTestContext) -> String {
    view.read_with(cx, |view, cx| view.summary_input().read(cx).text())
}

fn type_summary(view: &Entity<RepoWindow>, text: &str, cx: &mut VisualTestContext) {
    let input = view.read_with(cx, |view, _| view.summary_input());
    input.update(cx, |input, cx| input.set_text(text.to_owned(), cx));
}

fn other_repo_window(view: &Entity<RepoWindow>, cx: &VisualTestContext) -> Entity<RepoWindow> {
    cx.cx
        .windows()
        .into_iter()
        .filter_map(|window| window.downcast::<RepoWindow>())
        .filter_map(|window| window.entity(&cx.cx).ok())
        .find(|entity| entity != view)
        .expect("second repo window")
}

fn recorded_as_recent(dir: &str, cx: &VisualTestContext) -> bool {
    cx.cx.read(|cx| {
        config::current(cx).recent_repos.iter().any(|recent| {
            Path::new(recent)
                .file_name()
                .is_some_and(|name| name == dir)
        })
    })
}

#[gpui::test]
fn switching_workspaces_reuses_the_window_and_keeps_each_draft(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    add_workspace(&fixture, "first");
    let (view, cx) = open_fixture(&fixture, cx);
    let source = current_workspace(&view, cx);
    type_summary(&view, "draft in source", cx);

    pick_workspace("first", Modifiers::default(), cx);

    assert_eq!(current_workspace(&view, cx), "first");
    assert_eq!(repo_window_count(cx), 1);
    assert_ne!(summary(&view, cx), "draft in source");
    type_summary(&view, "draft in first", cx);

    pick_workspace("default", Modifiers::default(), cx);

    assert_eq!(current_workspace(&view, cx), source);
    assert_eq!(summary(&view, cx), "draft in source");
    pick_workspace("first", Modifiers::default(), cx);
    assert_eq!(summary(&view, cx), "draft in first");
}

#[gpui::test]
fn open_workspace_is_activated_and_alt_click_opens_a_new_window(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let first = add_workspace(&fixture, "first");
    add_workspace(&fixture, "second");
    let (view, cx) = open_fixture(&fixture, cx);
    let source = current_workspace(&view, cx);
    cx.update(|_, cx| open_repo_window(first, cx));
    settle_visual(cx);
    assert_eq!(repo_window_count(cx), 2);

    pick_workspace("first", Modifiers::default(), cx);

    assert_eq!(current_workspace(&view, cx), source);
    assert_eq!(repo_window_count(cx), 2);

    let alt = Modifiers {
        alt: true,
        ..Modifiers::default()
    };
    pick_workspace("second", alt, cx);

    assert_eq!(current_workspace(&view, cx), source);
    assert_eq!(repo_window_count(cx), 3);
}

#[gpui::test]
fn failed_switch_keeps_the_source_workspace_and_its_draft(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let broken = add_workspace(&fixture, "broken");
    std::fs::remove_dir_all(broken.join(".jj")).expect("break the checkout");
    let (view, cx) = open_fixture(&fixture, cx);
    let source = current_workspace(&view, cx);
    type_summary(&view, "draft before failure", cx);
    open_child_windows(&view, cx);

    pick_workspace("broken", Modifiers::default(), cx);
    assert_eq!(child_window_count(cx), 3);

    assert_eq!(current_workspace(&view, cx), source);
    assert_eq!(repo_window_count(cx), 1);
    assert_eq!(summary(&view, cx), "draft before failure");
    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert!(vm.repo.is_some(), "source repo stays open");
        assert!(vm.error.is_some(), "the failed open is surfaced");
    });
}

#[gpui::test]
fn switching_back_to_the_current_workspace_cancels_a_pending_switch(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let first = add_workspace(&fixture, "first");
    let (view, cx) = open_fixture(&fixture, cx);
    let source = fixture.path.clone();

    view.update(cx, |view, cx| {
        view.switch_workspace(first.clone(), cx);
        view.switch_workspace(source, cx);
    });
    settle_visual(cx);
    assert_eq!(current_workspace(&view, cx), "default");
}

#[gpui::test]
fn switch_waits_for_a_running_operation(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let first = add_workspace(&fixture, "first");
    let (view, cx) = open_fixture(&fixture, cx);
    let vm = view.read_with(cx, |view, _| view.view_model());
    vm.update(cx, |vm, _| vm.loading.operations = 1);

    view.update(cx, |view, cx| view.switch_workspace(first.clone(), cx));
    settle_visual(cx);

    assert_eq!(current_workspace(&view, cx), "default");
    assert!(!recorded_as_recent("first", cx));
    assert!(
        view.read_with(cx, |view, _| view.toast())
            .is_some_and(|toast| toast.contains("running operation"))
    );

    vm.update(cx, |vm, _| vm.loading.operations = 0);
    view.update(cx, |view, cx| view.switch_workspace(first, cx));
    settle_visual(cx);
    assert_eq!(current_workspace(&view, cx), "first");
    assert!(recorded_as_recent("first", cx));
}

#[gpui::test]
fn removing_a_workspace_drops_drafts_parked_in_other_windows(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let first = add_workspace(&fixture, "first");
    let second = add_workspace(&fixture, "second");
    let (view, cx) = open_fixture(&fixture, cx);
    cx.update(|_, cx| open_repo_window(first.clone(), cx));
    settle_visual(cx);
    let other = other_repo_window(&view, cx);
    type_summary(&other, "stale draft", cx);
    other.update(cx, |other, cx| other.switch_workspace(second, cx));
    settle_visual(cx);
    assert_eq!(current_workspace(&other, cx), "second");

    open_switcher(cx);
    let row = cx
        .debug_bounds("repo-switcher-workspace-first")
        .expect("workspace row");
    cx.simulate_mouse_down(row.center(), MouseButton::Right, Modifiers::default());
    settle_visual(cx);
    let forget = cx.debug_bounds("context-menu-Forget").expect("forget item");
    cx.simulate_click(forget.center(), Modifiers::default());
    settle_visual(cx);

    std::fs::remove_dir_all(&first).expect("clear the old checkout");
    add_workspace(&fixture, "first");
    other.update(cx, |other, cx| other.switch_workspace(first, cx));
    settle_visual(cx);

    assert_eq!(current_workspace(&other, cx), "first");
    assert_ne!(summary(&other, cx), "stale draft");
}

#[gpui::test]
fn cleared_commit_message_survives_switching_away_and_back(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    add_workspace(&fixture, "first");
    run_jj_in(&fixture.path, &["describe", "-m", "original"]);
    let (view, cx) = open_fixture(&fixture, cx);
    assert_eq!(summary(&view, cx), "original");
    type_summary(&view, "", cx);

    pick_workspace("first", Modifiers::default(), cx);
    pick_workspace("default", Modifiers::default(), cx);
    let vm = view.read_with(cx, |view, _| view.view_model());
    vm.update(cx, |vm, cx| vm.refresh(false, cx));
    settle_visual(cx);

    assert_eq!(current_workspace(&view, cx), "default");
    assert_eq!(summary(&view, cx), "");
}

#[gpui::test]
fn concurrent_switches_to_one_workspace_leave_a_single_window_on_it(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let first = add_workspace(&fixture, "first");
    let second = add_workspace(&fixture, "second");
    let (view, cx) = open_fixture(&fixture, cx);
    cx.update(|_, cx| open_repo_window(second, cx));
    settle_visual(cx);
    let other = other_repo_window(&view, cx);

    view.update(cx, |view, cx| view.switch_workspace(first.clone(), cx));
    other.update(cx, |other, cx| other.switch_workspace(first, cx));
    settle_visual(cx);

    let on_first = [&view, &other]
        .into_iter()
        .filter(|window| current_workspace(window, cx) == "first")
        .count();
    assert_eq!(on_first, 1);
    assert_eq!(repo_window_count(cx), 2);
}

#[gpui::test]
fn workspace_opened_in_a_new_window_during_a_switch_stays_in_that_window(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let first = add_workspace(&fixture, "first");
    let (view, cx) = open_fixture(&fixture, cx);

    view.update(cx, |view, cx| view.switch_workspace(first.clone(), cx));
    cx.update(|_, cx| {
        cx.spawn(async move |cx| cx.update(|cx| open_repo_window(first, cx)))
            .detach();
    });
    settle_visual(cx);

    assert_eq!(current_workspace(&view, cx), "default");
    assert_eq!(repo_window_count(cx), 2);
    let opened = other_repo_window(&view, cx);
    assert_eq!(current_workspace(&opened, cx), "first");
}

fn open_child_windows(view: &Entity<RepoWindow>, cx: &mut VisualTestContext) {
    cx.focus(view);
    cx.dispatch_action(OpenOverview);
    cx.dispatch_action(OpenBookmarkManager);
    cx.dispatch_action(OpenOperationLog);
    settle_visual(cx);
}

fn child_window_count(cx: &VisualTestContext) -> usize {
    cx.cx
        .windows()
        .iter()
        .filter(|window| window.downcast::<RepoWindow>().is_none())
        .count()
}

#[gpui::test]
fn switching_workspaces_closes_the_windows_opened_for_the_old_one(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    add_workspace(&fixture, "first");
    let (view, cx) = open_fixture(&fixture, cx);
    open_child_windows(&view, cx);
    assert_eq!(child_window_count(cx), 3);
    let old_vm = view.read_with(cx, |view, _| view.view_model().downgrade());

    pick_workspace("first", Modifiers::default(), cx);
    settle_visual(cx);

    assert_eq!(child_window_count(cx), 0);
    assert_eq!(current_workspace(&view, cx), "first");
    assert!(old_vm.upgrade().is_none());
}
