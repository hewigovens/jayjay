#![allow(dead_code)]

use gpui::{Entity, TestAppContext, VisualTestContext};
use jayjay_gpui::app::config::{AppConfig, AppConfigStore};
use jayjay_gpui::app::theme::Theme;
use jayjay_gpui::repo::RepoWindow;
use jj_test::LinearFixture;

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

pub(crate) fn install_test_globals(cx: &mut TestAppContext) {
    cx.update(|cx| {
        cx.bind_keys(jayjay_gpui::app::actions::app_key_bindings());
        cx.set_global(AppConfigStore::new_ephemeral(AppConfig::default()));
        cx.set_global(Theme::light());
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
