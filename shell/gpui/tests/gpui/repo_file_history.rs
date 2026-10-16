use std::path::Path;

use crate::harness::{open_fixture, selector, settle_visual};
use gpui::{Focusable, Keystroke, Modifiers, TestAppContext, VisualTestContext, WindowHandle};
use jayjay_core::ChangeInfo;
use jayjay_gpui::ui::context_menu::ContextAction;
use jayjay_gpui::windows::file_history::FileHistoryView;
use jj_test::{LinearFixture, run_jj_in};

fn history_fixture() -> LinearFixture {
    let fixture = LinearFixture::build();
    run_jj_in(&fixture.path, &["new", "-m", "edit beta"]);
    std::fs::write(fixture.path.join("hello.txt"), "beta\n").expect("write beta");
    run_jj_in(&fixture.path, &["new", "-m", "edit gamma"]);
    std::fs::write(fixture.path.join("hello.txt"), "gamma\n").expect("write gamma");
    run_jj_in(&fixture.path, &["st"]);
    fixture
}

fn hello_history(path: &Path) -> Vec<ChangeInfo> {
    jayjay_core::Repo::open(path)
        .expect("open repo")
        .file_history("hello.txt")
        .expect("file history")
}

fn open_history(fixture: &LinearFixture, cx: &mut TestAppContext) -> VisualTestContext {
    let (view, repo_cx) = open_fixture(fixture, cx);
    view.update_in(repo_cx, |view, _, cx| {
        view.dispatch_context_action(ContextAction::OpenFileHistoryFor("hello.txt".into()), cx);
    });
    settle_visual(repo_cx);
    let window = repo_cx
        .cx
        .windows()
        .into_iter()
        .find(|window| window.downcast::<FileHistoryView>().is_some())
        .expect("file history window");
    let mut history_cx = VisualTestContext::from_window(window, &repo_cx.cx);
    settle_visual(&mut history_cx);
    let history = history_window(&history_cx);
    history
        .update(&mut history_cx.cx, |view, window, cx| {
            window.focus(&view.focus_handle(cx), cx);
        })
        .expect("focus file history");
    history_cx
}

fn history_window(cx: &VisualTestContext) -> WindowHandle<FileHistoryView> {
    cx.cx
        .windows()
        .into_iter()
        .find_map(|window| window.downcast::<FileHistoryView>())
        .expect("file history window")
}

fn click(cx: &mut VisualTestContext, name: &'static str) {
    let bounds = cx
        .debug_bounds(name)
        .unwrap_or_else(|| panic!("missing {name}"));
    cx.simulate_click(bounds.center(), Modifiers::default());
    settle_visual(cx);
}

fn endpoint(role: &str, change: &ChangeInfo) -> &'static str {
    selector(format!(
        "file-history-{role}-{}-{}",
        change.change_id.id, change.commit_id.id
    ))
}

#[gpui::test]
fn file_history_pins_moves_swaps_and_clears(cx: &mut TestAppContext) {
    let fixture = history_fixture();
    let history = hello_history(&fixture.path);
    let descriptions: Vec<_> = history
        .iter()
        .map(|change| change.description.trim().to_owned())
        .collect();
    assert_eq!(
        descriptions,
        vec!["edit gamma", "edit beta", "add hello"],
        "history order"
    );
    let gamma = &history[0];
    let hello = &history[2];

    let mut history_cx = open_history(&fixture, cx);
    let view = history_window(&history_cx);
    let endpoints = |view: &WindowHandle<FileHistoryView>, cx: &VisualTestContext| {
        view.read_with(&cx.cx, |view, _| {
            (view.comparison_endpoints(), view.loaded_file_contents())
        })
        .expect("read history")
    };

    let (ends, contents) = endpoints(&view, &history_cx);
    assert_eq!(ends, (None, Some(gamma.commit_id.id.clone())));
    assert_eq!(
        contents,
        Some((Some("beta\n".into()), Some("gamma\n".into())))
    );
    assert!(history_cx.debug_bounds(endpoint("target", gamma)).is_some());
    assert!(history_cx.debug_bounds("file-history-swap").is_none());
    assert!(history_cx.debug_bounds("file-history-diff").is_some());

    click(&mut history_cx, "file-history-pin");
    let (ends, contents) = endpoints(&view, &history_cx);
    assert_eq!(
        ends,
        (
            Some(gamma.commit_id.id.clone()),
            Some(gamma.commit_id.id.clone())
        )
    );
    assert_eq!(contents, None);
    assert!(
        history_cx
            .debug_bounds("file-history-same-revision")
            .is_some()
    );
    assert!(history_cx.debug_bounds(endpoint("base", gamma)).is_some());
    assert!(history_cx.debug_bounds("file-history-swap").is_some());
    assert!(history_cx.debug_bounds("file-history-diff").is_none());

    history_cx.simulate_keystrokes("down down");
    let (ends, contents) = endpoints(&view, &history_cx);
    assert_eq!(
        ends,
        (
            Some(gamma.commit_id.id.clone()),
            Some(hello.commit_id.id.clone())
        )
    );
    assert_eq!(
        contents,
        Some((Some("gamma\n".into()), Some("hello\n".into())))
    );
    assert!(history_cx.debug_bounds(endpoint("target", hello)).is_some());
    assert!(
        history_cx
            .debug_bounds("file-history-same-revision")
            .is_none()
    );
    assert!(history_cx.debug_bounds("file-history-diff").is_some());

    click(&mut history_cx, "file-history-swap");
    let (ends, contents) = endpoints(&view, &history_cx);
    assert_eq!(
        ends,
        (
            Some(hello.commit_id.id.clone()),
            Some(gamma.commit_id.id.clone())
        )
    );
    assert_eq!(
        contents,
        Some((Some("hello\n".into()), Some("gamma\n".into())))
    );
    assert!(history_cx.debug_bounds(endpoint("base", hello)).is_some());
    assert!(history_cx.debug_bounds(endpoint("target", gamma)).is_some());

    click(&mut history_cx, "file-history-clear");
    let (ends, contents) = endpoints(&view, &history_cx);
    assert_eq!(ends, (None, Some(gamma.commit_id.id.clone())));
    assert_eq!(
        contents,
        Some((Some("beta\n".into()), Some("gamma\n".into())))
    );
    assert!(history_cx.debug_bounds("file-history-swap").is_none());
    assert!(history_cx.debug_bounds(endpoint("base", hello)).is_none());

    click(&mut history_cx, "file-history-pin");
    history_cx.simulate_keystrokes("escape");
    let (ends, _) = endpoints(&view, &history_cx);
    assert_eq!(ends.0, None, "escape clears the pinned base before closing");
    assert!(
        history_cx
            .cx
            .windows()
            .into_iter()
            .any(|window| window.downcast::<FileHistoryView>().is_some()),
        "escape should leave file history open after clearing the base"
    );

    history_cx.simulate_keystrokes("escape");
    assert!(
        history_cx
            .cx
            .windows()
            .into_iter()
            .all(|window| window.downcast::<FileHistoryView>().is_none()),
        "a second escape should close file history"
    );
}

#[gpui::test]
fn file_history_drops_a_stale_comparison(cx: &mut TestAppContext) {
    let fixture = history_fixture();
    let history = hello_history(&fixture.path);
    let gamma = history[0].commit_id.id.clone();
    let hello = history[2].commit_id.id.clone();

    let mut history_cx = open_history(&fixture, cx);
    click(&mut history_cx, "file-history-pin");
    let view = history_window(&history_cx);
    for key in ["down", "down"] {
        let keystroke = Keystroke::parse(key).expect("key");
        history_cx.update(|window, cx| window.dispatch_keystroke(keystroke, cx));
    }

    let mut painted_beta_under_hello = false;
    for _ in 0..64 {
        if !history_cx.background_executor.tick() {
            break;
        }
        let (target, contents) = view
            .read_with(&history_cx.cx, |view, _| {
                (view.comparison_endpoints().1, view.loaded_file_contents())
            })
            .expect("read history");
        if target.as_deref() == Some(hello.as_str())
            && contents.as_ref().and_then(|(_, new)| new.as_deref()) == Some("beta\n")
        {
            painted_beta_under_hello = true;
        }
    }
    history_cx.run_until_parked();
    let (ends, contents) = view
        .read_with(&history_cx.cx, |view, _| {
            (view.comparison_endpoints(), view.loaded_file_contents())
        })
        .expect("read settled history");

    assert!(
        !painted_beta_under_hello,
        "an older gamma→beta diff painted under the hello target"
    );
    assert_eq!(ends, (Some(gamma), Some(hello)));
    assert_eq!(
        contents,
        Some((Some("gamma\n".into()), Some("hello\n".into())))
    );
}
