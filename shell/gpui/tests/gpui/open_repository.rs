use crate::harness::{install_test_globals, settle_visual};
use gpui::{TestAppContext, VisualTestContext};
use jayjay_gpui::repo::RepoWindow;
use jayjay_gpui::windows::open_repository::OpenRepositoryPathView;
use jj_test::LinearFixture;

#[gpui::test]
fn typed_path_fallback_opens_a_repository(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    install_test_globals(cx);
    cx.update(OpenRepositoryPathView::open);
    let window = cx.windows().last().copied().expect("path fallback window");
    let mut visual = VisualTestContext::from_window(window, cx);
    settle_visual(&mut visual);

    visual.simulate_input(fixture.path.to_str().expect("utf-8 fixture path"));
    visual.simulate_keystrokes("enter");
    visual.cx.run_until_parked();
    visual.cx.executor().run_until_parked();

    assert_eq!(
        visual
            .cx
            .windows()
            .iter()
            .filter(|window| window.downcast::<RepoWindow>().is_some())
            .count(),
        1
    );
}
