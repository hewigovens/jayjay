use crate::harness::*;
use gpui::{Modifiers, MouseButton, TestAppContext, VisualContext};
use jayjay_gpui::repo::RepoWindow;
use jj_test::{LinearFixture, run_git, run_jj_in};

#[gpui::test]
fn repository_title_picker_combines_workspaces_repositories_and_actions(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let workspace_path = fixture
        .path
        .parent()
        .expect("fixture parent")
        .join("feature-picker");
    run_jj_in(
        &fixture.path,
        &[
            "workspace",
            "add",
            "--name",
            "feature-picker",
            workspace_path.to_str().expect("workspace path UTF-8"),
        ],
    );
    let (view, repo_cx) = open_repo(workspace_path, cx);
    repo_cx.focus(&view);

    assert!(
        repo_cx.debug_bounds("repo-title-repository-repo").is_some(),
        "secondary workspace title should use the primary repository name"
    );
    assert!(
        repo_cx
            .debug_bounds("repo-title-workspace-feature-picker")
            .is_some(),
        "secondary workspace name should follow the repository title"
    );

    let title = repo_cx
        .debug_bounds("repo-switcher-button")
        .expect("repository title picker button");
    repo_cx.simulate_click(title.center(), Modifiers::default());
    settle_visual(repo_cx);

    for selector in [
        "repo-switcher-panel",
        "repo-switcher-filter",
        "repo-switcher-refresh",
        "repo-switcher-new",
        "repo-switcher-workspaces",
        "repo-switcher-workspace-feature-picker",
        "repo-switcher-workspace-default",
        "repo-switcher-repositories",
        "repo-switcher-open-0",
        "repo-switcher-list",
        "repo-switcher-open-repository",
    ] {
        assert!(
            repo_cx.debug_bounds(selector).is_some(),
            "missing picker element {selector}"
        );
    }

    repo_cx.simulate_input("default");
    settle_visual(repo_cx);
    assert!(
        repo_cx
            .debug_bounds("repo-switcher-workspace-feature-picker")
            .is_none()
    );
    assert!(
        repo_cx
            .debug_bounds("repo-switcher-workspace-default")
            .is_some()
    );

    repo_cx.simulate_keystrokes("enter");
    settle_visual(repo_cx);
    assert!(repo_cx.debug_bounds("repo-switcher-panel").is_none());
    assert_eq!(
        repo_cx
            .cx
            .windows()
            .iter()
            .filter(|window| window.downcast::<RepoWindow>().is_some())
            .count(),
        2,
        "the selected workspace should open in its own repository window"
    );
}

#[gpui::test]
fn repository_picker_pins_rows_without_opening_them(cx: &mut TestAppContext) {
    use jayjay_core::repositories::normalize_repository_path;
    use jayjay_gpui::app::repositories;

    let fixture = LinearFixture::build();
    let workspace_path = fixture.path.parent().unwrap().join("pinned-picker");
    run_jj_in(
        &fixture.path,
        &[
            "workspace",
            "add",
            "--name",
            "pinned-picker",
            workspace_path.to_str().unwrap(),
        ],
    );
    let (view, repo_cx) = open_fixture(&fixture, cx);
    repo_cx.focus(&view);

    for (row_id, menu_id, expected_path) in [
        (
            "repo-switcher-open-0",
            "context-menu-Pin",
            Some(&fixture.path),
        ),
        (
            "repo-switcher-workspace-default",
            "context-menu-Unpin",
            None,
        ),
        (
            "repo-switcher-workspace-pinned-picker",
            "context-menu-Pin",
            Some(&workspace_path),
        ),
        ("repo-switcher-pinned-0", "context-menu-Unpin", None),
    ] {
        let title = repo_cx.debug_bounds("repo-switcher-button").unwrap();
        repo_cx.simulate_click(title.center(), Modifiers::default());
        settle_visual(repo_cx);
        let row = repo_cx.debug_bounds(row_id).expect(row_id);
        repo_cx.simulate_mouse_down(row.center(), MouseButton::Right, Modifiers::default());
        settle_visual(repo_cx);
        let item = repo_cx.debug_bounds(menu_id).expect(menu_id);
        repo_cx.simulate_click(item.center(), Modifiers::default());
        settle_visual(repo_cx);

        let expected: Vec<String> = expected_path
            .map(|path| {
                normalize_repository_path(path)
                    .to_string_lossy()
                    .into_owned()
            })
            .into_iter()
            .collect();
        assert_eq!(repo_cx.cx.update(repositories::current), expected);
        assert!(repo_cx.debug_bounds("repo-switcher-panel").is_none());
        assert_eq!(
            repo_cx.cx.windows().len(),
            1,
            "pinning must not open a window"
        );
    }

    let title = repo_cx.debug_bounds("repo-switcher-button").unwrap();
    repo_cx.simulate_click(title.center(), Modifiers::default());
    settle_visual(repo_cx);
    assert!(repo_cx.debug_bounds("repo-switcher-pinned-0").is_none());
    assert!(
        repo_cx
            .debug_bounds("repo-switcher-workspace-pinned-picker")
            .is_some()
    );
}

#[gpui::test]
fn bookmark_picker_groups_filters_and_applies_bookmark_revsets(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let _remote = create_tracked_bookmark(&fixture, "tracked-picker");
    run_git(&fixture.path, &["branch", "odd&name", "HEAD"]);
    run_jj_in(&fixture.path, &["st"]);
    let (view, repo_cx) = open_fixture(&fixture, cx);
    repo_cx.focus(&view);

    let bookmarks = repo_cx
        .debug_bounds("toolbar-bookmarks-3")
        .expect("bookmark picker button");
    repo_cx.simulate_click(bookmarks.center(), Modifiers::default());
    settle_visual(repo_cx);

    for selector in [
        "bookmark-picker-panel",
        "bookmark-picker-filter",
        "bookmark-picker-new",
        "bookmark-picker-tracked",
        "bookmark-picker-row-tracked-picker",
        "bookmark-picker-row-main",
    ] {
        assert!(
            repo_cx.debug_bounds(selector).is_some(),
            "missing picker element {selector}"
        );
    }

    repo_cx.simulate_input("tracked");
    settle_visual(repo_cx);
    assert!(
        repo_cx
            .debug_bounds("bookmark-picker-row-tracked-picker")
            .is_some()
    );
    assert!(repo_cx.debug_bounds("bookmark-picker-row-main").is_none());

    repo_cx.simulate_keystrokes("enter");
    settle_visual(repo_cx);
    assert!(repo_cx.debug_bounds("bookmark-picker-panel").is_none());
    view.read_with(repo_cx, |view, cx| {
        assert_eq!(
            view.view_model().read(cx).revset.as_ref(),
            "\"tracked-picker\""
        );
    });

    let bookmarks = repo_cx
        .debug_bounds("toolbar-bookmarks-3")
        .expect("bookmark picker button");
    repo_cx.simulate_click(bookmarks.center(), Modifiers::default());
    settle_visual(repo_cx);
    repo_cx.simulate_input("odd");
    repo_cx.simulate_keystrokes("enter");
    settle_visual(repo_cx);
    view.read_with(repo_cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert_eq!(vm.revset.as_ref(), "\"odd&name\"");
        assert!(vm.error.is_none(), "{:?}", vm.error);
        assert_eq!(vm.graph.changes.len(), 1, "the literal selects one change");
    });
}

#[gpui::test]
fn bookmark_picker_browses_remote_history_without_tracking(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let _remote = create_tracked_bookmark(&fixture, "remote-picker");
    run_jj_in(
        &fixture.path,
        &["bookmark", "untrack", "remote-picker@origin"],
    );
    run_jj_in(&fixture.path, &["bookmark", "delete", "remote-picker"]);
    let (view, repo_cx) = open_fixture(&fixture, cx);
    repo_cx.focus(&view);
    let before = run_jj_in(
        &fixture.path,
        &["log", "--no-graph", "-r", "mutable()", "-T", "commit_id"],
    );

    let button = repo_cx
        .debug_bounds("toolbar-bookmarks-2")
        .expect("bookmark picker");
    repo_cx.simulate_click(button.center(), Modifiers::default());
    settle_visual(repo_cx);
    assert!(repo_cx.debug_bounds("bookmark-picker-remote").is_some());
    let row = repo_cx
        .debug_bounds("bookmark-picker-remote-row-13:remote-pickerorigin")
        .expect("remote row");
    repo_cx.simulate_mouse_down(row.center(), MouseButton::Right, Modifiers::default());
    settle_visual(repo_cx);
    assert!(
        repo_cx
            .debug_bounds("context-menu-Track remote-picker@origin")
            .is_some()
    );
    assert!(repo_cx.debug_bounds("context-menu-Push").is_none());
    assert!(repo_cx.debug_bounds("context-menu-Move to @-").is_none());
    repo_cx.simulate_keystrokes("escape");
    repo_cx.simulate_input("remote-picker@origin");
    repo_cx.simulate_keystrokes("enter");
    settle_visual(repo_cx);
    view.read_with(repo_cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert_eq!(
            vm.revset.as_ref(),
            "ancestors(remote_bookmarks(exact:\"remote-picker\", exact:\"origin\"), 20)"
        );
        assert!(vm.error.is_none(), "{:?}", vm.error);
        assert_eq!(vm.graph.changes.len(), 3);
    });
    let bookmarks = jayjay_core::Repo::open(&fixture.path)
        .unwrap()
        .list_bookmarks()
        .unwrap();
    let bookmark = bookmarks
        .iter()
        .find(|b| b.name == "remote-picker")
        .unwrap();
    assert!(!bookmark.has_local_target && !bookmark.is_tracking_remote);
    assert_eq!(
        run_jj_in(
            &fixture.path,
            &["log", "--no-graph", "-r", "mutable()", "-T", "commit_id"]
        ),
        before
    );

    repo_cx.simulate_click(button.center(), Modifiers::default());
    settle_visual(repo_cx);
    let row = repo_cx
        .debug_bounds("bookmark-picker-remote-row-13:remote-pickerorigin")
        .unwrap();
    repo_cx.simulate_mouse_down(row.center(), MouseButton::Right, Modifiers::default());
    settle_visual(repo_cx);
    let track = repo_cx
        .debug_bounds("context-menu-Track remote-picker@origin")
        .unwrap();
    repo_cx.simulate_click(track.center(), Modifiers::default());
    settle_visual(repo_cx);
    assert!(repo_cx.debug_bounds("bookmark-picker-panel").is_none());
    assert!(
        jayjay_core::Repo::open(&fixture.path)
            .unwrap()
            .list_bookmarks()
            .unwrap()
            .iter()
            .any(|b| b.name == "remote-picker" && b.has_local_target && b.is_tracking_remote)
    );
}

#[gpui::test]
fn bookmark_picker_menu_offers_no_removal_for_a_conflicted_bookmark(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    create_conflicted_bookmark(&fixture, "clash");
    let (view, repo_cx) = open_fixture(&fixture, cx);
    repo_cx.focus(&view);
    let bookmarks = repo_cx
        .debug_bounds("toolbar-bookmarks-2")
        .expect("bookmark picker button");
    repo_cx.simulate_click(bookmarks.center(), Modifiers::default());
    settle_visual(repo_cx);

    let row = repo_cx
        .debug_bounds("bookmark-picker-row-clash")
        .expect("conflicted bookmark row");
    repo_cx.simulate_mouse_down(row.center(), MouseButton::Right, Modifiers::default());
    settle_visual(repo_cx);

    assert!(
        repo_cx
            .debug_bounds("context-menu-Resolve conflict (set to @)")
            .is_some()
    );
    assert!(
        repo_cx.debug_bounds("bookmark-picker-panel").is_some(),
        "a right-click menu opens over the picker, not instead of it"
    );
    repo_cx.simulate_keystrokes("enter");
    settle_visual(repo_cx);
    assert!(
        repo_cx.debug_bounds("bookmark-picker-panel").is_some()
            && repo_cx
                .debug_bounds("context-menu-Resolve conflict (set to @)")
                .is_some(),
        "keys are not routed to the picker while its menu is open"
    );
    assert!(
        repo_cx
            .debug_bounds("context-menu-Remove from This Change")
            .is_none(),
        "the picker has no change to remove the bookmark from"
    );

    let filter = repo_cx
        .debug_bounds("context-menu-Filter by this bookmark")
        .expect("filter menu item");
    repo_cx.simulate_click(filter.center(), Modifiers::default());
    settle_visual(repo_cx);
    view.read_with(repo_cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert_eq!(vm.revset.as_ref(), "bookmarks(exact:\"clash\")");
        assert!(vm.error.is_none(), "{:?}", vm.error);
        assert_eq!(
            vm.graph.changes.len(),
            2,
            "both conflicted targets are listed"
        );
    });
}

#[gpui::test]
fn bookmark_picker_new_uses_the_existing_create_flow(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, repo_cx) = open_fixture(&fixture, cx);
    repo_cx.focus(&view);
    let bookmarks = repo_cx
        .debug_bounds("toolbar-bookmarks-1")
        .expect("bookmark picker button");
    repo_cx.simulate_click(bookmarks.center(), Modifiers::default());
    settle_visual(repo_cx);

    let new = repo_cx
        .debug_bounds("bookmark-picker-new")
        .expect("new bookmark button");
    repo_cx.simulate_click(new.center(), Modifiers::default());
    settle_visual(repo_cx);

    assert!(repo_cx.debug_bounds("bookmark-picker-panel").is_none());
    view.read_with(repo_cx, |view, _| {
        assert!(view.has_text_modal(), "New should open Create Bookmark");
    });
}

#[gpui::test]
fn clicks_inside_a_picker_do_not_dismiss_it(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, repo_cx) = open_fixture(&fixture, cx);
    repo_cx.focus(&view);
    let bookmarks = repo_cx
        .debug_bounds("toolbar-bookmarks-1")
        .expect("bookmark picker button");
    repo_cx.simulate_click(bookmarks.center(), Modifiers::default());
    settle_visual(repo_cx);

    let section_header = ["bookmark-picker-tracked", "bookmark-picker-local"]
        .into_iter()
        .find(|selector| repo_cx.debug_bounds(selector).is_some())
        .expect("a bookmark section header");
    for selector in ["bookmark-picker-filter", section_header] {
        let target = repo_cx
            .debug_bounds(selector)
            .unwrap_or_else(|| panic!("missing {selector}"));
        repo_cx.simulate_click(target.center(), Modifiers::default());
        settle_visual(repo_cx);
        assert!(
            repo_cx.debug_bounds("bookmark-picker-panel").is_some(),
            "clicking {selector} dismissed the picker"
        );
    }

    let panel = repo_cx
        .debug_bounds("bookmark-picker-panel")
        .expect("picker panel");
    repo_cx.simulate_click(
        gpui::point(
            panel.right() + gpui::px(40.),
            panel.bottom() + gpui::px(40.),
        ),
        Modifiers::default(),
    );
    settle_visual(repo_cx);
    assert!(repo_cx.debug_bounds("bookmark-picker-panel").is_none());
}

#[gpui::test]
fn bookmark_picker_updates_an_open_revset_panel(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, repo_cx) = open_fixture(&fixture, cx);
    repo_cx.focus(&view);
    let filter = repo_cx
        .debug_bounds("toolbar-revset-filter")
        .expect("revset filter toggle");
    repo_cx.simulate_click(filter.center(), Modifiers::default());
    settle_visual(repo_cx);
    assert!(repo_cx.debug_bounds("revset-filter-input").is_some());

    let bookmarks = repo_cx
        .debug_bounds("toolbar-bookmarks-1")
        .expect("bookmark picker button");
    repo_cx.simulate_click(bookmarks.center(), Modifiers::default());
    settle_visual(repo_cx);
    let row = repo_cx
        .debug_bounds("bookmark-picker-row-main")
        .expect("main bookmark row");
    repo_cx.simulate_click(row.center(), Modifiers::default());
    settle_visual(repo_cx);

    view.read_with(repo_cx, |view, cx| {
        assert_eq!(view.view_model().read(cx).revset.as_ref(), "\"main\"");
        assert_eq!(view.revset_filter_text().as_deref(), Some("\"main\""));
    });
}

#[gpui::test]
fn workspace_rows_keep_the_switcher_open_for_their_menu(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let workspace_path = fixture
        .path
        .parent()
        .expect("fixture parent")
        .join("feature-menu");
    run_jj_in(
        &fixture.path,
        &[
            "workspace",
            "add",
            "--name",
            "feature-menu",
            workspace_path.to_str().expect("workspace path UTF-8"),
        ],
    );
    let (view, repo_cx) = open_fixture(&fixture, cx);
    repo_cx.focus(&view);
    let title = repo_cx
        .debug_bounds("repo-switcher-button")
        .expect("repository title picker button");
    repo_cx.simulate_click(title.center(), Modifiers::default());
    settle_visual(repo_cx);

    let row = repo_cx
        .debug_bounds("repo-switcher-workspace-feature-menu")
        .expect("workspace row");
    repo_cx.simulate_mouse_down(row.center(), MouseButton::Right, Modifiers::default());
    settle_visual(repo_cx);
    assert!(repo_cx.debug_bounds("context-menu-Forget").is_some());
    assert!(repo_cx.debug_bounds("repo-switcher-panel").is_some());

    let outside = repo_cx
        .debug_bounds("repo-switcher-panel")
        .expect("switcher panel");
    repo_cx.simulate_click(
        gpui::point(
            outside.origin.x - gpui::px(10.),
            outside.origin.y - gpui::px(10.),
        ),
        Modifiers::default(),
    );
    settle_visual(repo_cx);
    assert!(
        repo_cx.debug_bounds("context-menu-Forget").is_none(),
        "clicking outside dismisses the menu"
    );
    assert!(
        repo_cx.debug_bounds("repo-switcher-panel").is_some(),
        "dismissing the menu does not click through to the switcher backdrop"
    );
}

#[gpui::test]
fn bookmark_picker_enter_activates_the_best_match_across_sections(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let _remote = create_tracked_bookmark(&fixture, "domain");
    let (view, repo_cx) = open_fixture(&fixture, cx);
    repo_cx.focus(&view);
    let bookmarks = repo_cx
        .debug_bounds("toolbar-bookmarks-2")
        .expect("bookmark picker button");
    repo_cx.simulate_click(bookmarks.center(), Modifiers::default());
    settle_visual(repo_cx);
    assert!(repo_cx.debug_bounds("bookmark-picker-tracked").is_some());
    assert!(repo_cx.debug_bounds("bookmark-picker-local").is_some());

    repo_cx.simulate_input("main");
    repo_cx.simulate_keystrokes("enter");
    settle_visual(repo_cx);

    view.read_with(repo_cx, |view, cx| {
        assert_eq!(view.view_model().read(cx).revset.as_ref(), "\"main\"");
    });
}

#[gpui::test]
fn forget_and_delete_confirms_then_removes_the_workspace_directory(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let workspace_path = fixture
        .path
        .parent()
        .expect("fixture parent")
        .join("doomed");
    run_jj_in(
        &fixture.path,
        &[
            "workspace",
            "add",
            "--name",
            "doomed",
            workspace_path.to_str().expect("workspace path UTF-8"),
        ],
    );
    let (view, repo_cx) = open_fixture(&fixture, cx);
    repo_cx.focus(&view);
    let recorded = jayjay_core::repositories::normalize_repository_path(&workspace_path)
        .to_string_lossy()
        .into_owned();
    repo_cx.update(|_, cx| {
        jayjay_gpui::app::config::update(cx, |config| config.recent_repos.push(recorded.clone()));
        jayjay_gpui::app::repositories::set_pinned(cx, &workspace_path, true);
    });
    let open_switcher = |repo_cx: &mut gpui::VisualTestContext| {
        let title = repo_cx
            .debug_bounds("repo-switcher-button")
            .expect("repository title picker button");
        repo_cx.simulate_click(title.center(), Modifiers::default());
        settle_visual(repo_cx);
        let row = repo_cx
            .debug_bounds("repo-switcher-workspace-doomed")
            .expect("workspace row");
        repo_cx.simulate_mouse_down(row.center(), MouseButton::Right, Modifiers::default());
        settle_visual(repo_cx);
        let delete = repo_cx
            .debug_bounds("context-menu-Forget & Delete from Disk")
            .expect("delete menu item");
        repo_cx.simulate_click(delete.center(), Modifiers::default());
        settle_visual(repo_cx);
        assert!(repo_cx.debug_bounds("confirmation").is_some());
        assert!(
            repo_cx.debug_bounds("repo-switcher-panel").is_none(),
            "the confirmation takes over from the switcher"
        );
    };

    open_switcher(repo_cx);
    repo_cx.simulate_keystrokes("escape");
    settle_visual(repo_cx);
    assert!(repo_cx.debug_bounds("confirmation").is_none());
    assert!(
        workspace_path.exists(),
        "cancelling must not touch the directory"
    );

    open_switcher(repo_cx);
    let confirm = repo_cx
        .debug_bounds("confirmation-submit")
        .expect("confirm button");
    repo_cx.simulate_click(confirm.center(), Modifiers::default());
    settle_visual(repo_cx);

    assert!(
        !workspace_path.exists(),
        "the workspace directory is deleted"
    );
    view.read_with(repo_cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert!(vm.error.is_none(), "{:?}", vm.error);
        assert!(!vm.graph.workspaces.iter().any(|w| w.name == "doomed"));
        assert_eq!(view.toast().as_deref(), Some("Deleted workspace doomed"));
    });
    assert!(
        !repo_cx
            .update(|_, cx| jayjay_gpui::app::config::current(cx).recent_repos.clone())
            .iter()
            .any(|path| path == &recorded)
    );
    assert!(
        !repo_cx
            .update(|_, cx| jayjay_gpui::app::repositories::current(cx))
            .iter()
            .any(|path| path == &recorded),
        "a deleted workspace must not stay pinned"
    );
}

#[gpui::test]
fn the_primary_root_cannot_be_deleted_from_a_secondary_workspace(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let secondary = fixture
        .path
        .parent()
        .expect("fixture parent")
        .join("secondary");
    run_jj_in(
        &fixture.path,
        &[
            "workspace",
            "add",
            "--name",
            "secondary",
            secondary.to_str().expect("workspace path UTF-8"),
        ],
    );
    let (view, repo_cx) = open_repo(secondary, cx);
    repo_cx.focus(&view);
    let title = repo_cx
        .debug_bounds("repo-switcher-button")
        .expect("repository title picker button");
    repo_cx.simulate_click(title.center(), Modifiers::default());
    settle_visual(repo_cx);
    let row = repo_cx
        .debug_bounds("repo-switcher-workspace-default")
        .expect("primary workspace row");
    repo_cx.simulate_mouse_down(row.center(), MouseButton::Right, Modifiers::default());
    settle_visual(repo_cx);

    assert!(repo_cx.debug_bounds("context-menu-Forget").is_some());
    assert!(
        repo_cx
            .debug_bounds("context-menu-Forget & Delete from Disk")
            .is_none(),
        "deleting the directory that owns .jj/repo is never offered"
    );
}

#[gpui::test]
fn forgetting_a_workspace_closes_its_window(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let secondary = fixture
        .path
        .parent()
        .expect("fixture parent")
        .join("forgotten");
    run_jj_in(
        &fixture.path,
        &[
            "workspace",
            "add",
            "--name",
            "forgotten",
            secondary.to_str().expect("workspace path UTF-8"),
        ],
    );
    let (view, repo_cx) = open_fixture(&fixture, cx);
    repo_cx.update(|_, cx| jayjay_gpui::repo::open_repo_window(secondary.clone(), cx));
    settle_visual(repo_cx);
    let repo_windows = |repo_cx: &gpui::VisualTestContext| {
        repo_cx
            .cx
            .windows()
            .iter()
            .filter(|window| window.downcast::<RepoWindow>().is_some())
            .count()
    };
    assert_eq!(repo_windows(repo_cx), 2);

    repo_cx.focus(&view);
    let title = repo_cx
        .debug_bounds("repo-switcher-button")
        .expect("repository title picker button");
    repo_cx.simulate_click(title.center(), Modifiers::default());
    settle_visual(repo_cx);
    let row = repo_cx
        .debug_bounds("repo-switcher-workspace-forgotten")
        .expect("workspace row");
    repo_cx.simulate_mouse_down(row.center(), MouseButton::Right, Modifiers::default());
    settle_visual(repo_cx);
    let forget = repo_cx
        .debug_bounds("context-menu-Forget")
        .expect("forget menu item");
    repo_cx.simulate_click(forget.center(), Modifiers::default());
    settle_visual(repo_cx);

    assert_eq!(
        repo_windows(repo_cx),
        1,
        "the forgotten workspace's window closes"
    );
    assert!(repo_cx.debug_bounds("repo-switcher-panel").is_none());
    assert!(
        secondary.exists(),
        "plain Forget leaves the directory alone"
    );
    view.read_with(repo_cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert!(!vm.graph.workspaces.iter().any(|w| w.name == "forgotten"));
    });
}

#[gpui::test]
fn bookmark_picker_deletes_a_bookmark_on_a_divergent_change(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    run_jj_in(
        &fixture.path,
        &[
            "bookmark",
            "create",
            "doomed",
            "-r",
            "subject(\"add hello\")",
        ],
    );
    let base_op = run_jj_in(
        &fixture.path,
        &["op", "log", "--no-graph", "--limit", "1", "-T", "id"],
    );
    let base_op = String::from_utf8(base_op.stdout).expect("utf-8 op id");
    run_jj_in(
        &fixture.path,
        &[
            "describe",
            "-r",
            "subject(\"add hello\")",
            "-m",
            "add hello (alt)",
        ],
    );
    run_jj_in(
        &fixture.path,
        &[
            "--at-op",
            base_op.trim(),
            "describe",
            "-r",
            "subject(\"add hello\")",
            "-m",
            "add hello (orig)",
        ],
    );
    run_jj_in(
        &fixture.path,
        &[
            "bookmark",
            "set",
            "doomed",
            "--allow-backwards",
            "-r",
            "subject(\"add hello (alt)\")",
        ],
    );
    let (view, repo_cx) = open_fixture(&fixture, cx);
    repo_cx.focus(&view);
    view.read_with(repo_cx, |view, cx| {
        let vm = view.view_model().read(cx);
        let target = vm
            .graph
            .changes
            .iter()
            .find(|change| change.bookmarks.iter().any(|b| b == "doomed"))
            .expect("doomed bookmark target");
        assert!(
            target.is_divergent,
            "fixture must bookmark a divergent change"
        );
    });

    let bookmarks = repo_cx
        .debug_bounds("toolbar-bookmarks-2")
        .expect("bookmark picker button");
    repo_cx.simulate_click(bookmarks.center(), Modifiers::default());
    settle_visual(repo_cx);
    let row = repo_cx
        .debug_bounds("bookmark-picker-row-doomed")
        .expect("doomed bookmark row");
    repo_cx.simulate_mouse_down(row.center(), MouseButton::Right, Modifiers::default());
    settle_visual(repo_cx);
    let delete = repo_cx
        .debug_bounds("context-menu-Delete Bookmark")
        .expect("delete menu item");
    repo_cx.simulate_click(delete.center(), Modifiers::default());
    settle_visual(repo_cx);

    view.read_with(repo_cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert!(vm.error.is_none(), "{:?}", vm.error);
        assert!(
            !vm.graph
                .changes
                .iter()
                .any(|change| change.bookmarks.iter().any(|b| b == "doomed"))
        );
        assert_eq!(view.toast().as_deref(), Some("Deleted bookmark doomed"));
    });
}
