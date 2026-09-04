#![allow(dead_code)]

use std::sync::{Arc, Mutex};

use gpui::{
    Entity, Global, Modifiers, MouseButton, Pixels, TestAppContext, VisualTestContext, point, px,
};
use jayjay_core::ChangeInfo;
use jayjay_gpui::app::actions::ZoomIn;
use jayjay_gpui::app::config::{AppConfig, AppConfigStore};
use jayjay_gpui::app::theme::Theme;
use jayjay_gpui::repo::RepoWindow;
use jj_test::{LinearFixture, run_git, run_jj_in};

pub(crate) fn settle(cx: &mut TestAppContext) {
    for _ in 0..8 {
        cx.run_until_parked();
        cx.executor().run_until_parked();
    }
}

pub(crate) fn settle_visual(cx: &mut VisualTestContext) {
    for _ in 0..8 {
        cx.run_until_parked();
        cx.cx.run_until_parked();
        cx.cx.executor().run_until_parked();
    }
}

pub(crate) fn drag_between(
    cx: &mut VisualTestContext,
    source_selector: &'static str,
    target_selector: &'static str,
) {
    let source = cx
        .debug_bounds(source_selector)
        .unwrap_or_else(|| panic!("missing drag source {source_selector}"));
    let target = cx
        .debug_bounds(target_selector)
        .unwrap_or_else(|| panic!("missing drop target {target_selector}"));
    cx.simulate_mouse_down(source.center(), MouseButton::Left, Modifiers::default());
    cx.simulate_mouse_move(target.center(), MouseButton::Left, Modifiers::default());
    cx.simulate_mouse_up(target.center(), MouseButton::Left, Modifiers::default());
    settle_visual(cx);
}

pub(crate) fn drag_handle(cx: &mut VisualTestContext, handle: &'static str, dx: f32) {
    let start = cx.debug_bounds(handle).expect(handle).center();
    let end = point(start.x + px(dx), start.y);
    cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::default());
    cx.simulate_mouse_move(end, MouseButton::Left, Modifiers::default());
    cx.simulate_mouse_up(end, MouseButton::Left, Modifiers::default());
    settle_visual(cx);
}

pub(crate) fn pane_width(cx: &mut VisualTestContext, selector: &'static str) -> f32 {
    f32::from(cx.debug_bounds(selector).expect(selector).size.width)
}

pub(crate) fn rendered_height(cx: &mut VisualTestContext, selector: &'static str) -> Pixels {
    cx.debug_bounds(selector)
        .unwrap_or_else(|| panic!("rendered {selector}"))
        .size
        .height
}

pub(crate) fn zoom_to_max(cx: &mut VisualTestContext) {
    cx.cx.update(|cx| {
        jayjay_gpui::app::menus::install(cx);
        for _ in 0..12 {
            cx.dispatch_action(&ZoomIn);
        }
    });
    settle_visual(cx);
}

pub(crate) fn create_tracked_bookmark(fixture: &LinearFixture, name: &str) -> tempfile::TempDir {
    let remote = tempfile::tempdir().expect("create remote directory");
    run_git(remote.path(), &["init", "--bare", "origin.git"]);
    let origin = remote.path().join("origin.git");
    run_git(
        &fixture.path,
        &[
            "remote",
            "add",
            "origin",
            origin.to_str().expect("origin path utf-8"),
        ],
    );
    run_jj_in(&fixture.path, &["bookmark", "create", name, "-r", "main"]);
    run_jj_in(
        &fixture.path,
        &["git", "push", "--bookmark", name, "--remote", "origin"],
    );
    remote
}

/// Two concurrent `bookmark set` operations onto sibling commits leave `name` with both targets (`name??`).
pub(crate) fn create_conflicted_bookmark(fixture: &LinearFixture, name: &str) {
    let path = &fixture.path;
    run_jj_in(
        path,
        &["new", "--no-edit", "-m", "side", "subject(\"initial\")"],
    );
    run_jj_in(
        path,
        &["bookmark", "create", name, "-r", "subject(\"initial\")"],
    );
    let base_op = run_jj_in(
        path,
        &["op", "log", "--no-graph", "--limit", "1", "-T", "id"],
    );
    let base_op = String::from_utf8(base_op.stdout).expect("utf-8 op id");
    run_jj_in(path, &["bookmark", "set", name, "-r", "subject(\"side\")"]);
    run_jj_in(
        path,
        &[
            "--at-op",
            base_op.trim(),
            "bookmark",
            "set",
            name,
            "-r",
            "main",
        ],
    );
    run_jj_in(path, &["st"]);
}

/// The last URL handed to the app's opener; links go through `app::links`, not GPUI's platform opener.
struct OpenedUrl(Arc<Mutex<Option<String>>>);

impl Global for OpenedUrl {}

pub(crate) fn opened_url(cx: &mut TestAppContext) -> Option<String> {
    cx.run_until_parked();
    cx.update(|cx| {
        cx.global::<OpenedUrl>()
            .0
            .lock()
            .expect("opened URL lock")
            .clone()
    })
}

pub(crate) fn install_test_globals(cx: &mut TestAppContext) {
    cx.update(|cx| {
        cx.bind_keys(jayjay_gpui::app::actions::app_key_bindings());
        cx.set_global(AppConfigStore::new_ephemeral(AppConfig::default()));
        cx.set_global(Theme::light());
        let opened = Arc::new(Mutex::new(None));
        cx.set_global(OpenedUrl(opened.clone()));
        jayjay_gpui::app::links::install_url_opener(cx, move |url| {
            *opened.lock().expect("opened URL lock") = Some(url.to_owned());
            true
        });
        jayjay_gpui::app::repositories::install_in_memory(cx);
        // Hermetic review store: no reads or writes of the real review_store.json.
        jayjay_gpui::repo::window::install_in_memory_review_store(cx);
    });
    suppress_fs_watcher(cx);
}

/// Keep the real `notify` FSEvents thread out of `RepoWindow`s; it would trip the deterministic GPUI test scheduler.
pub(crate) fn suppress_fs_watcher(cx: &mut TestAppContext) {
    cx.update(jayjay_gpui::app::fs_watcher::suppress_for_tests);
}

pub(crate) fn load_selected_change_files(view: &Entity<RepoWindow>, cx: &mut VisualTestContext) {
    view.update_in(cx, |view, _, cx| {
        let selected = view.view_model().read(cx).selected;
        if let Some(ix) = selected {
            view.view_model()
                .update(cx, |vm, cx| vm.select_change(ix, cx));
        }
    });
}

pub(crate) fn open_fixture<'a>(
    fixture: &LinearFixture,
    cx: &'a mut TestAppContext,
) -> (Entity<RepoWindow>, &'a mut VisualTestContext) {
    open_repo(fixture.path.clone(), cx)
}

pub(crate) fn open_repo(
    path: std::path::PathBuf,
    cx: &mut TestAppContext,
) -> (Entity<RepoWindow>, &mut VisualTestContext) {
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(path, cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);
    (view, cx)
}

pub(crate) fn change_with_subject(
    view: &Entity<RepoWindow>,
    cx: &VisualTestContext,
    subject: &str,
) -> ChangeInfo {
    view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .graph
            .changes
            .iter()
            .find(|change| change.description.trim() == subject)
            .unwrap_or_else(|| panic!("missing {subject} change"))
            .clone()
    })
}

pub(crate) fn bookmarks_on(
    view: &Entity<RepoWindow>,
    cx: &VisualTestContext,
    commit_id: &str,
) -> Vec<String> {
    view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .graph
            .changes
            .iter()
            .find(|change| change.commit_id.id == commit_id)
            .unwrap_or_else(|| panic!("missing change with commit id {commit_id}"))
            .bookmarks
            .clone()
    })
}

pub(crate) fn select_file(
    view: &Entity<RepoWindow>,
    path: &str,
    cx: &mut VisualTestContext,
) -> usize {
    let ix = view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .files
            .as_ref()
            .expect("files loaded")
            .iter()
            .position(|hunk| hunk.path == path)
            .unwrap_or_else(|| panic!("file '{path}' present"))
    });
    view.update_in(cx, |view, _, cx| view.select_file(ix, cx));
    settle_visual(cx);
    ix
}

pub(crate) fn selector(value: String) -> &'static str {
    Box::leak(value.into_boxed_str())
}
