use crate::harness::*;
use gpui::{Modifiers, MouseButton, TestAppContext, VisualContext, VisualTestContext, point, px};
use jayjay_core::bookmark_filter_revset;
use jayjay_gpui::repo::RepoWindow;
use jayjay_gpui::windows::settings::{SettingsSection, SettingsView};
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
    let panel = repo_cx
        .debug_bounds("repo-switcher-panel")
        .expect("repository picker panel");
    assert_eq!(
        panel.origin,
        point(title.left(), (title.bottom() + px(4.)).round())
    );

    for selector in [
        "repo-switcher-panel",
        "repo-switcher-filter",
        "repo-switcher-overview",
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
    assert_eq!(repo_window_count(repo_cx), 1);
    assert_eq!(current_workspace(&view, repo_cx), "default");
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
    repo_cx.simulate_keystrokes("escape");
    view.update_in(repo_cx, |view, _, cx| {
        view.view_model().update(cx, |vm, _| {
            let workspace = std::sync::Arc::make_mut(&mut vm.graph.workspaces)
                .iter_mut()
                .find(|workspace| workspace.name == "pinned-picker")
                .unwrap();
            workspace.pinnable_path = None;
        });
    });
    let title = repo_cx.debug_bounds("repo-switcher-button").unwrap();
    repo_cx.simulate_click(title.center(), Modifiers::default());
    settle_visual(repo_cx);
    let row = repo_cx
        .debug_bounds("repo-switcher-workspace-pinned-picker")
        .unwrap();
    repo_cx.simulate_mouse_down(row.center(), MouseButton::Right, Modifiers::default());
    settle_visual(repo_cx);
    assert!(repo_cx.debug_bounds("context-menu-Pin").is_none());
    assert!(repo_cx.debug_bounds("context-menu-Unpin").is_none());
    assert!(
        repo_cx
            .debug_bounds("context-menu-Copy Workspace Name")
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
        .debug_bounds("bookmarks-button-3")
        .expect("bookmark picker button");
    repo_cx.simulate_click(bookmarks.center(), Modifiers::default());
    settle_visual(repo_cx);
    let panel = repo_cx
        .debug_bounds("bookmark-picker-panel")
        .expect("bookmark picker panel");
    assert_eq!(
        panel.origin,
        point(bookmarks.left(), (bookmarks.bottom() + px(4.)).round())
    );

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

    let default_revset = view.read_with(repo_cx, |view, cx| {
        view.view_model().read(cx).revset().to_owned()
    });
    repo_cx.simulate_keystrokes("enter");
    settle_visual(repo_cx);
    assert!(repo_cx.debug_bounds("bookmark-picker-panel").is_none());
    view.read_with(repo_cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert_eq!(
            vm.revset(),
            default_revset,
            "a shown bookmark is selected, not filtered"
        );
        assert_eq!(
            vm.selected_change()
                .map(|change| change.commit_id.id.clone()),
            Some(bookmark_commit(&fixture, "tracked-picker"))
        );
    });

    let bookmarks = repo_cx
        .debug_bounds("bookmarks-button-3")
        .expect("bookmark picker button");
    repo_cx.simulate_click(bookmarks.center(), Modifiers::default());
    settle_visual(repo_cx);
    repo_cx.simulate_input("odd");
    settle_visual(repo_cx);
    filter_by_bookmark_row(repo_cx, "bookmark-picker-row-odd&name");
    view.read_with(repo_cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert_eq!(
            vm.revset(),
            bookmark_filter_revset("odd&name", None).as_str()
        );
        assert!(vm.error.is_none(), "{:?}", vm.error);
        assert_eq!(
            vm.graph.changes.len(),
            3,
            "the filter shows the bookmark's stack"
        );
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
        .debug_bounds("bookmarks-button-2")
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
    let filter = repo_cx
        .debug_bounds("context-menu-Filter by This Bookmark")
        .expect("filter menu item");
    repo_cx.simulate_click(filter.center(), Modifiers::default());
    settle_visual(repo_cx);
    view.read_with(repo_cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert_eq!(
            vm.revset(),
            bookmark_filter_revset("remote-picker", Some("origin")).as_str()
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
        .debug_bounds("bookmarks-button-2")
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
        .debug_bounds("context-menu-Filter by This Bookmark")
        .expect("filter menu item");
    repo_cx.simulate_click(filter.center(), Modifiers::default());
    settle_visual(repo_cx);
    view.read_with(repo_cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert_eq!(vm.revset(), bookmark_filter_revset("clash", None).as_str());
        assert!(vm.error.is_none(), "{:?}", vm.error);
        assert_eq!(
            vm.graph.changes.len(),
            4,
            "both conflicted targets are listed with their stacks"
        );
    });
}

#[gpui::test]
fn bookmark_picker_new_uses_the_existing_create_flow(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, repo_cx) = open_fixture(&fixture, cx);
    repo_cx.focus(&view);
    let bookmarks = repo_cx
        .debug_bounds("bookmarks-button-1")
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
        .debug_bounds("bookmarks-button-1")
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
        .debug_bounds("bookmarks-button-2")
        .expect("bookmark picker button");
    repo_cx.simulate_click(bookmarks.center(), Modifiers::default());
    settle_visual(repo_cx);
    assert!(repo_cx.debug_bounds("bookmark-picker-tracked").is_some());
    assert!(repo_cx.debug_bounds("bookmark-picker-local").is_some());

    repo_cx.simulate_input("main");
    repo_cx.simulate_keystrokes("enter");
    settle_visual(repo_cx);

    view.read_with(repo_cx, |view, cx| {
        assert_eq!(
            view.view_model()
                .read(cx)
                .selected_change()
                .map(|change| change.commit_id.id.clone()),
            Some(bookmark_commit(&fixture, "main"))
        );
    });
}

fn bookmark_commit(fixture: &LinearFixture, name: &str) -> String {
    let output = run_jj_in(
        &fixture.path,
        &["log", "--no-graph", "-r", name, "-T", "commit_id"],
    );
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

fn filter_by_bookmark_row(repo_cx: &mut VisualTestContext, row_selector: &'static str) {
    let row = repo_cx.debug_bounds(row_selector).expect("bookmark row");
    repo_cx.simulate_mouse_down(row.center(), MouseButton::Right, Modifiers::default());
    settle_visual(repo_cx);
    let filter = repo_cx
        .debug_bounds("context-menu-Filter by This Bookmark")
        .expect("filter menu item");
    repo_cx.simulate_click(filter.center(), Modifiers::default());
    settle_visual(repo_cx);
}

fn request_workspace_delete(repo_cx: &mut VisualTestContext, row_selector: &'static str) {
    let title = repo_cx
        .debug_bounds("repo-switcher-button")
        .expect("repository title picker button");
    repo_cx.simulate_click(title.center(), Modifiers::default());
    settle_visual(repo_cx);
    let row = repo_cx.debug_bounds(row_selector).expect("workspace row");
    repo_cx.simulate_mouse_down(row.center(), MouseButton::Right, Modifiers::default());
    settle_visual(repo_cx);
    let delete = repo_cx
        .debug_bounds("context-menu-Forget & Delete from Disk")
        .expect("delete menu item");
    repo_cx.simulate_click(delete.center(), Modifiers::default());
    settle_visual(repo_cx);
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
    let request_confirmation = |repo_cx: &mut VisualTestContext| {
        request_workspace_delete(repo_cx, "repo-switcher-workspace-doomed");
        assert!(repo_cx.debug_bounds("confirmation").is_some());
        assert!(
            repo_cx.debug_bounds("repo-switcher-panel").is_none(),
            "the confirmation takes over from the switcher"
        );
    };

    request_confirmation(repo_cx);
    repo_cx.simulate_keystrokes("escape");
    settle_visual(repo_cx);
    assert!(repo_cx.debug_bounds("confirmation").is_none());
    assert!(
        workspace_path.exists(),
        "cancelling must not touch the directory"
    );

    request_confirmation(repo_cx);
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
fn workspace_delete_confirmation_can_be_skipped_and_restored_in_settings(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let parent = fixture.path.parent().expect("fixture parent");
    for name in ["first", "second", "third"] {
        let path = parent.join(name);
        run_jj_in(
            &fixture.path,
            &[
                "workspace",
                "add",
                "--name",
                name,
                path.to_str().expect("workspace path UTF-8"),
            ],
        );
    }
    let (view, repo_cx) = open_fixture(&fixture, cx);
    repo_cx.focus(&view);
    request_workspace_delete(repo_cx, "repo-switcher-workspace-first");
    let dont_ask = repo_cx
        .debug_bounds("confirmation-dont-ask-again")
        .expect("don't ask again checkbox");
    repo_cx.simulate_click(dont_ask.center(), Modifiers::default());
    settle_visual(repo_cx);
    let confirm = repo_cx
        .debug_bounds("confirmation-submit")
        .expect("confirm button");
    repo_cx.simulate_click(confirm.center(), Modifiers::default());
    settle_visual(repo_cx);
    assert!(!parent.join("first").exists());

    request_workspace_delete(repo_cx, "repo-switcher-workspace-second");
    assert!(
        repo_cx.debug_bounds("confirmation").is_none(),
        "the checked flag skips the confirmation"
    );
    assert!(
        !parent.join("second").exists(),
        "the workspace is deleted without confirming"
    );

    repo_cx.update(|_, cx| SettingsView::open_section(SettingsSection::Workflow, cx));
    let settings_window = repo_cx
        .cx
        .windows()
        .last()
        .copied()
        .expect("settings window");
    let mut settings_cx = VisualTestContext::from_window(settings_window, &repo_cx.cx);
    settle_visual(&mut settings_cx);
    let toggle = settings_cx
        .debug_bounds("setting-workflow-confirm-workspace-delete")
        .expect("workspace delete setting");
    settings_cx.simulate_click(toggle.center(), Modifiers::default());
    settle_visual(&mut settings_cx);

    request_workspace_delete(repo_cx, "repo-switcher-workspace-third");
    assert!(repo_cx.debug_bounds("confirmation").is_some());
    repo_cx.simulate_keystrokes("escape");
    settle_visual(repo_cx);
    assert!(
        parent.join("third").exists(),
        "cancelling preserves the workspace"
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
        .debug_bounds("bookmarks-button-2")
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

#[gpui::test]
fn bookmark_picker_selects_the_divergent_version_its_bookmark_names(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
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
    for (name, subject) in [("alt", "add hello (alt)"), ("orig", "add hello (orig)")] {
        let revset = format!("subject(\"{subject}\")");
        run_jj_in(&fixture.path, &["bookmark", "create", name, "-r", &revset]);
    }
    let (view, repo_cx) = open_fixture(&fixture, cx);
    repo_cx.focus(&view);

    for name in ["orig", "alt"] {
        let bookmarks = repo_cx
            .debug_bounds("bookmarks-button-3")
            .expect("bookmark picker button");
        repo_cx.simulate_click(bookmarks.center(), Modifiers::default());
        settle_visual(repo_cx);
        let row = repo_cx
            .debug_bounds(match name {
                "orig" => "bookmark-picker-row-orig",
                _ => "bookmark-picker-row-alt",
            })
            .expect("bookmark row");
        repo_cx.simulate_click(row.center(), Modifiers::default());
        settle_visual(repo_cx);
        view.read_with(repo_cx, |view, cx| {
            let vm = view.view_model().read(cx);
            let selected = vm.selected_change().expect("a selected change");
            assert!(selected.is_divergent);
            assert_eq!(selected.commit_id.id, bookmark_commit(&fixture, name));
        });
    }
}

#[gpui::test]
fn picker_buttons_close_their_open_picker(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, repo_cx) = open_fixture(&fixture, cx);
    repo_cx.focus(&view);
    for (button, panel) in [
        ("bookmarks-button-1", "bookmark-picker-panel"),
        ("repo-switcher-button", "repo-switcher-panel"),
    ] {
        let bounds = repo_cx.debug_bounds(button).expect(button);
        repo_cx.simulate_click(bounds.center(), Modifiers::default());
        settle_visual(repo_cx);
        assert!(repo_cx.debug_bounds(panel).is_some(), "{button} opens");
        repo_cx.simulate_click(bounds.center(), Modifiers::default());
        settle_visual(repo_cx);
        assert!(repo_cx.debug_bounds(panel).is_none(), "{button} closes");
    }
}

#[gpui::test]
fn bookmark_picker_builds_only_visible_rows_and_reveals_the_keyboard_selection(
    cx: &mut TestAppContext,
) {
    const COUNT: usize = 60;
    let fixture = LinearFixture::build();
    let names: Vec<String> = (0..COUNT).map(|i| format!("bulk-{i:02}")).collect();
    let mut args = vec!["bookmark", "create", "-r", "@-"];
    args.extend(names.iter().map(String::as_str));
    run_jj_in(&fixture.path, &args);
    let (view, repo_cx) = open_fixture(&fixture, cx);
    repo_cx.focus(&view);

    let button = repo_cx
        .debug_bounds(selector(format!("bookmarks-button-{}", COUNT + 1)))
        .expect("bookmark picker button");
    repo_cx.simulate_click(button.center(), Modifiers::default());
    settle_visual(repo_cx);
    let rows: Vec<&'static str> = names
        .iter()
        .map(|name| selector(format!("bookmark-picker-row-{name}")))
        .collect();
    let built = rows
        .iter()
        .filter(|row| repo_cx.debug_bounds(row).is_some())
        .count();
    assert!(built > 0 && built < COUNT, "{built} of {COUNT} rows built");
    assert!(repo_cx.debug_bounds(rows[COUNT - 1]).is_none());

    for _ in 0..=COUNT {
        repo_cx.simulate_keystrokes("down");
    }
    settle_visual(repo_cx);
    assert!(
        repo_cx.debug_bounds(rows[COUNT - 1]).is_some(),
        "moving the selection to the end scrolls the last rows into view"
    );
}
