mod support;

use gpui::{Focusable, KeyBinding, Modifiers, TestAppContext, VisualTestContext};
use jayjay_gpui::app::actions::{CloseWindow, Dismiss};
use jayjay_gpui::app::config;
use jayjay_gpui::repo::{RepoWindow, open_repo_window};
use jayjay_gpui::windows::repo_list::RepoListWindow;
use jj_test::LinearFixture;
use support::{install_test_globals, settle, settle_visual};

#[gpui::test]
fn cmd_w_replaces_last_repo_window_with_repo_list(cx: &mut TestAppContext) {
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

    view.update_in(cx, |view, _, cx| view.open_find(cx));
    cx.simulate_keystrokes("escape");
    view.read_with(cx, |view, _| {
        assert!(view.find_query_text().is_none(), "escape should close find");
    });

    cx.simulate_keystrokes("cmd-w");

    assert_single_repo_list(&cx.cx);
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

    cx.simulate_keystrokes("cmd-w");
    view.read_with(cx, |view, _| {
        assert!(view.find_query_text().is_none(), "cmd-w should close find");
    });
    assert_eq!(
        cx.cx.windows().len(),
        1,
        "cmd-w with an overlay open must dismiss the overlay, not the window"
    );

    cx.simulate_keystrokes("cmd-w");

    assert_single_repo_list(&cx.cx);
}

#[gpui::test]
fn native_close_of_last_repo_window_opens_repo_list(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    install_test_globals(cx);
    cx.update(|cx| open_repo_window(fixture.path.clone(), cx));
    let repo_window = cx
        .windows()
        .into_iter()
        .find(|handle| handle.downcast::<RepoWindow>().is_some())
        .expect("repo window");
    let mut visual = VisualTestContext::from_window(repo_window, cx);

    assert!(visual.simulate_close(), "native close should be accepted");
    settle_visual(&mut visual);

    assert_eq!(repo_list_count(&visual.cx), 1);
}

#[gpui::test]
fn repo_list_opens_only_after_last_repo_window_closes(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    install_test_globals(cx);
    let first = cx.add_window(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let second = cx.add_window(|_, cx| RepoWindow::new(fixture.path.clone(), cx));

    first
        .update(cx, |_, window, cx| {
            RepoListWindow::open_if_last_repo_window(cx);
            window.remove_window();
        })
        .expect("close first repo window");
    settle(cx);
    assert_eq!(repo_list_count(cx), 0);

    second
        .update(cx, |_, window, cx| {
            RepoListWindow::open_if_last_repo_window(cx);
            window.remove_window();
        })
        .expect("close second repo window");
    settle(cx);

    assert_single_repo_list(cx);
}

#[gpui::test]
fn removing_a_recent_repo_does_not_open_it(cx: &mut TestAppContext) {
    install_test_globals(cx);
    cx.update(|cx| {
        config::update(cx, |cfg| {
            cfg.recent_repos = vec!["/tmp/recent-repo".to_owned()];
        });
        RepoListWindow::open(cx);
    });
    let window = cx.windows().last().copied().expect("repo list window");
    let mut visual = VisualTestContext::from_window(window, cx);
    settle_visual(&mut visual);
    let remove = visual
        .debug_bounds("repo-list-remove-0")
        .expect("remove recent repository button");

    visual.simulate_click(remove.center(), Modifiers::default());
    settle_visual(&mut visual);

    assert!(
        visual
            .cx
            .update(|cx| config::current(cx).recent_repos.is_empty())
    );
    assert_eq!(
        visual
            .cx
            .windows()
            .iter()
            .filter(|handle| handle.downcast::<RepoWindow>().is_some())
            .count(),
        0,
        "removing a recent repository must not trigger the row's open action"
    );
}

#[gpui::test]
fn opening_a_recent_repo_replaces_the_repo_list(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    install_test_globals(cx);
    cx.update(|cx| {
        config::update(cx, |cfg| {
            cfg.recent_repos = vec![fixture.path.to_string_lossy().into_owned()];
        });
        RepoListWindow::open(cx);
    });
    let window = cx.windows().last().copied().expect("repo list window");
    let mut visual = VisualTestContext::from_window(window, cx);
    settle_visual(&mut visual);
    let row = visual
        .debug_bounds("repo-list-row-0")
        .expect("recent repository row");

    visual.simulate_click(row.center(), Modifiers::default());
    settle(&mut visual.cx);

    assert_eq!(repo_list_count(&visual.cx), 0);
    assert_eq!(
        visual
            .cx
            .windows()
            .iter()
            .filter(|handle| handle.downcast::<RepoWindow>().is_some())
            .count(),
        1
    );
}

#[gpui::test]
fn opening_an_already_open_repo_activates_one_window(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    install_test_globals(cx);

    cx.update(|cx| {
        open_repo_window(fixture.path.clone(), cx);
        open_repo_window(fixture.path.clone(), cx);
    });
    settle(cx);

    assert_eq!(
        cx.windows()
            .iter()
            .filter(|handle| handle.downcast::<RepoWindow>().is_some())
            .count(),
        1
    );
}

fn assert_single_repo_list(cx: &TestAppContext) {
    assert_eq!(cx.windows().len(), 1);
    assert_eq!(repo_list_count(cx), 1);
}

fn repo_list_count(cx: &TestAppContext) -> usize {
    cx.windows()
        .iter()
        .filter(|handle| handle.downcast::<RepoListWindow>().is_some())
        .count()
}
