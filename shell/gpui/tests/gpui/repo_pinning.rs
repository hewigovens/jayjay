use crate::harness::{install_test_globals, settle_visual};
use gpui::{Modifiers, TestAppContext, VisualTestContext};
use jayjay_core::repositories::normalize_repository_path;
use jayjay_gpui::app::{config, repositories};
use jayjay_gpui::repo::{RepoWindow, open_repo_window};
use jayjay_gpui::windows::repo_list::RepoListWindow;
use jj_test::LinearFixture;

fn normalized_path(directory: &tempfile::TempDir) -> String {
    normalize_repository_path(directory.path())
        .to_string_lossy()
        .into_owned()
}

#[gpui::test]
fn pinning_a_recent_repo_moves_it_without_opening_it(cx: &mut TestAppContext) {
    let repository = tempfile::tempdir().expect("recent repository directory");
    let repository_path = normalized_path(&repository);
    install_test_globals(cx);
    cx.update(|cx| {
        config::update(cx, |cfg| {
            cfg.recent_repos = vec![repository_path.clone()];
        });
        RepoListWindow::open(cx);
    });
    let window = cx.windows().last().copied().expect("repo list window");
    let mut visual = VisualTestContext::from_window(window, cx);
    settle_visual(&mut visual);

    let pin = visual
        .debug_bounds("repo-list-pin-0")
        .expect("pin recent repository button");
    visual.simulate_click(pin.center(), Modifiers::default());
    settle_visual(&mut visual);

    assert_eq!(
        visual.cx.update(repositories::current),
        vec![repository_path]
    );
    assert!(visual.debug_bounds("repo-list-pinned-row-0").is_some());
    assert_eq!(
        visual
            .cx
            .windows()
            .iter()
            .filter(|handle| handle.downcast::<RepoWindow>().is_some())
            .count(),
        0,
        "pinning must not open a repository window"
    );

    let unpin = visual
        .debug_bounds("repo-list-pinned-pin-0")
        .expect("unpin repository button");
    visual.simulate_click(unpin.center(), Modifiers::default());
    settle_visual(&mut visual);
    assert!(visual.cx.update(repositories::current).is_empty());
    assert!(visual.debug_bounds("repo-list-row-0").is_some());
}

#[gpui::test]
fn clearing_recent_repositories_preserves_pins(cx: &mut TestAppContext) {
    let pinned_repository = tempfile::tempdir().expect("pinned repository directory");
    let pinned_path = normalized_path(&pinned_repository);
    let recent_repository = tempfile::tempdir().expect("recent repository directory");
    let recent_path = recent_repository.path().to_string_lossy().into_owned();
    install_test_globals(cx);
    cx.update(|cx| {
        config::update(cx, |cfg| {
            cfg.recent_repos = vec![pinned_path.clone(), recent_path];
        });
        repositories::set_pinned(cx, pinned_repository.path(), true);
        RepoListWindow::open(cx);
    });
    let window = cx.windows().last().copied().expect("repo list window");
    let mut visual = VisualTestContext::from_window(window, cx);
    settle_visual(&mut visual);

    let clear = visual
        .debug_bounds("repo-list-clear")
        .expect("clear recent repositories button");
    visual.simulate_click(clear.center(), Modifiers::default());
    settle_visual(&mut visual);

    assert!(
        visual
            .cx
            .update(|cx| config::current(cx).recent_repos.is_empty())
    );
    assert_eq!(visual.cx.update(repositories::current), vec![pinned_path]);
    assert!(visual.debug_bounds("repo-list-pinned-row-0").is_some());
}

#[gpui::test]
fn repo_title_switcher_lists_open_windows_and_closed_pins(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let pinned = tempfile::tempdir().expect("pinned repository directory");
    install_test_globals(cx);
    cx.update(|cx| {
        repositories::set_pinned(cx, pinned.path(), true);
        open_repo_window(fixture.path.clone(), cx);
    });
    let window = cx
        .windows()
        .iter()
        .find(|handle| handle.downcast::<RepoWindow>().is_some())
        .copied()
        .expect("repo window");
    let mut visual = VisualTestContext::from_window(window, cx);
    settle_visual(&mut visual);

    let title = visual
        .debug_bounds("repo-switcher-button")
        .expect("repository title switcher");
    let sync = visual
        .debug_bounds("toolbar-sync-cluster")
        .expect("toolbar sync controls");
    assert!(
        title.size.height >= gpui::px(30.),
        "repository title switcher click target should match toolbar controls"
    );
    assert!(
        sync.origin.x + sync.size.width <= title.origin.x,
        "repository title switcher should follow the leading toolbar controls"
    );
    visual.simulate_click(title.center(), Modifiers::default());
    settle_visual(&mut visual);

    assert!(visual.debug_bounds("repo-switcher-panel").is_some());
    assert!(visual.debug_bounds("repo-switcher-open-0").is_some());
    assert!(visual.debug_bounds("repo-switcher-pinned-0").is_some());
}
