#![allow(dead_code)]

use std::fs;

use gpui::{Entity, TestAppContext, VisualTestContext};
use jayjay_gpui::app::config::{AppConfig, AppConfigStore};
use jayjay_gpui::app::theme::Theme;
use jayjay_gpui::repo::RepoWindow;
use jj_test::{LinearFixture, run_jj_in};

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

pub(crate) fn add_tracked_working_copy_edits(fixture: &LinearFixture) {
    fs::write(
        fixture.path.join("README.md"),
        "# Sample project\nEdited in GPUI test\n",
    )
    .expect("write README.md");
    fs::write(
        fixture.path.join("feature.txt"),
        "feature\nEdited in GPUI test\n",
    )
    .expect("write feature.txt");
    run_jj_in(&fixture.path, &["st"]);
}

pub(crate) fn add_multiline_working_copy_edit(fixture: &LinearFixture) {
    fs::write(
        fixture.path.join("feature.txt"),
        "second\nthird\nfourth\nfeature\n",
    )
    .expect("write feature.txt");
    run_jj_in(&fixture.path, &["st"]);
}

pub(crate) fn add_conflict_marker_working_copy_edit(fixture: &LinearFixture) {
    fs::write(
        fixture.path.join("feature.txt"),
        "<<<<<<< Conflict\none line\n>>>>>>> Conflict ends\nfeature\n",
    )
    .expect("write feature.txt");
    run_jj_in(&fixture.path, &["st"]);
}

pub(crate) fn remove_tracked_working_copy_file(fixture: &LinearFixture, path: &str) {
    fs::remove_file(fixture.path.join(path)).expect("remove tracked file");
    run_jj_in(&fixture.path, &["st"]);
}
