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
            outside.right() + gpui::px(40.),
            outside.bottom() + gpui::px(40.),
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
