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
        cx.set_global(AppConfigStore::new(AppConfig::default()));
        cx.set_global(Theme::light());
    });
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
