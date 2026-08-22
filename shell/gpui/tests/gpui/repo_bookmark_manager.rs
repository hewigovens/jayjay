use crate::harness::*;
use gpui::{Entity, Modifiers, MouseButton, TestAppContext, VisualContext, VisualTestContext};
use jayjay_core::{RemoteSyncStatus, Repo};
use jayjay_gpui::app::actions::OpenBookmarkManager;
use jayjay_gpui::repo::RepoWindow;
use jayjay_gpui::windows::bookmark_manager::BookmarkManagerView;
use jj_test::{LinearFixture, run_jj_in};

fn open_manager(view: &Entity<RepoWindow>, repo_cx: &mut VisualTestContext) -> VisualTestContext {
    repo_cx.focus(view);
    repo_cx.dispatch_action(OpenBookmarkManager);
    settle_visual(repo_cx);
    let manager_window = repo_cx
        .cx
        .windows()
        .into_iter()
        .find(|window| window.downcast::<BookmarkManagerView>().is_some())
        .expect("bookmark manager window");
    let mut manager_cx = VisualTestContext::from_window(manager_window, &repo_cx.cx);
    settle_visual(&mut manager_cx);
    manager_cx
}

#[gpui::test]
fn bookmark_count_and_manager_ignore_deleted_until_requested(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let _remote = create_tracked_bookmark(&fixture, "stale");
    run_jj_in(&fixture.path, &["bookmark", "delete", "stale"]);
    let (view, repo_cx) = open_fixture(&fixture, cx);

    assert!(repo_cx.debug_bounds("toolbar-bookmarks-1").is_some());
    let mut manager_cx = open_manager(&view, repo_cx);

    assert!(manager_cx.debug_bounds("bookmark-stat-active-1").is_some());
    assert!(manager_cx.debug_bounds("bookmark-stat-deleted-1").is_some());
    assert!(manager_cx.debug_bounds("bookmark-row-main").is_some());
    assert!(manager_cx.debug_bounds("bookmark-row-stale").is_none());

    let show_deleted = manager_cx
        .debug_bounds("bookmark-show-deleted")
        .expect("show deleted checkbox");
    manager_cx.simulate_click(show_deleted.center(), Modifiers::default());
    settle_visual(&mut manager_cx);

    assert!(manager_cx.debug_bounds("bookmark-row-stale").is_some());
    let filter = manager_cx
        .debug_bounds("bookmark-filter")
        .expect("bookmark filter");
    manager_cx.simulate_click(filter.center(), Modifiers::default());
    manager_cx.simulate_input("stale");
    settle_visual(&mut manager_cx);
    assert!(manager_cx.debug_bounds("bookmark-row-main").is_none());
    assert!(manager_cx.debug_bounds("bookmark-row-stale").is_some());
}

#[gpui::test]
fn bookmark_manager_push_goes_through_the_repo_window(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let _remote = create_tracked_bookmark(&fixture, "tracked");
    run_jj_in(&fixture.path, &["describe", "-m", "wip"]);
    run_jj_in(&fixture.path, &["bookmark", "set", "tracked", "-r", "@"]);
    let remote_status = |fixture: &LinearFixture| {
        Repo::open(&fixture.path)
            .expect("open fixture")
            .list_bookmarks()
            .expect("list bookmarks")
            .into_iter()
            .find(|bookmark| bookmark.name == "tracked")
            .and_then(|bookmark| bookmark.remote_targets.first().map(|target| target.status))
    };
    assert_eq!(remote_status(&fixture), Some(RemoteSyncStatus::Ahead));
    let (view, repo_cx) = open_fixture(&fixture, cx);
    let mut manager_cx = open_manager(&view, repo_cx);

    let row = manager_cx
        .debug_bounds("bookmark-row-tracked")
        .expect("tracked bookmark row");
    manager_cx.simulate_mouse_down(row.center(), MouseButton::Right, Modifiers::default());
    settle_visual(&mut manager_cx);
    let push = manager_cx
        .debug_bounds("bookmark-context-Push")
        .expect("push menu item");
    manager_cx.simulate_click(push.center(), Modifiers::default());
    settle_visual(&mut manager_cx);

    assert_eq!(remote_status(&fixture), Some(RemoteSyncStatus::Synced));
    view.read_with(&manager_cx, |view, cx| {
        assert!(view.view_model().read(cx).error.is_none());
        assert!(
            view.toast().is_some(),
            "the repo window's push pipeline reports the result"
        );
    });
}
