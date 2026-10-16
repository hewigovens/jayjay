use std::path::Path;

use crate::harness::{install_test_globals, repo_window_count, settle_visual};
use gpui::{Modifiers, TestAppContext, VisualTestContext};
use jayjay_gpui::app::config;
use jayjay_gpui::windows::clone_repository::CloneRepositoryView;
use jj_test::LinearFixture;

fn open_clone_window(recent_folder: &Path, cx: &mut TestAppContext) -> VisualTestContext {
    install_test_globals(cx);
    cx.update(|cx| {
        config::update(cx, |cfg| {
            cfg.recent_repos = vec![recent_folder.join("recent").display().to_string()];
        });
    });
    cx.update(CloneRepositoryView::open);
    let window = cx.windows().last().copied().expect("clone window");
    let mut visual = VisualTestContext::from_window(window, cx);
    settle_visual(&mut visual);
    visual
}

fn clone_window_is_open(visual: &VisualTestContext) -> bool {
    visual
        .cx
        .windows()
        .iter()
        .any(|window| window.downcast::<CloneRepositoryView>().is_some())
}

fn submit(visual: &mut VisualTestContext) {
    visual.simulate_keystrokes("enter");
    visual.cx.run_until_parked();
    visual.cx.executor().run_until_parked();
}

#[gpui::test]
fn cloning_beside_the_recent_repository_opens_the_clone(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let location = tempfile::tempdir().expect("clone location");
    let mut visual = open_clone_window(location.path(), cx);

    visual.simulate_input(fixture.path.to_str().expect("utf-8 fixture path"));
    submit(&mut visual);

    // The fixture lives at `<tmp>/repo`, so the name follows the URL to `repo`.
    assert!(location.path().join("repo/.jj").is_dir());
    assert_eq!(repo_window_count(&visual), 1);
    assert!(
        !clone_window_is_open(&visual),
        "the clone window stayed open"
    );
}

#[gpui::test]
fn a_typed_location_is_where_the_clone_lands(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let location = tempfile::tempdir().expect("clone location");
    let mut visual = open_clone_window(location.path(), cx);
    visual.simulate_input(fixture.path.to_str().expect("utf-8 fixture path"));
    settle_visual(&mut visual);

    let field = visual
        .debug_bounds("clone-repository-location")
        .expect("location field");
    visual.simulate_click(field.center(), Modifiers::default());
    visual.simulate_keystrokes(&format!("{}-a", jayjay_gpui::platform::MOD_KEY));
    visual.simulate_input(
        location
            .path()
            .join("renamed")
            .to_str()
            .expect("utf-8 path"),
    );
    submit(&mut visual);

    assert!(location.path().join("renamed/.jj").is_dir());
    assert!(!location.path().join("repo").exists());
}
