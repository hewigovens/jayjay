use gpui::{
    Context, Entity, IntoElement, Modifiers, ParentElement, Render, Styled, VisualTestContext,
    Window, div, px, size,
};

use super::action_row;
use crate::app::actions::OpenSettings;
use crate::app::config::{AppConfig, AppConfigStore};
use crate::app::theme::Theme;
use crate::repo::window::RepoWindow;

struct MenuRowHost {
    repo: Entity<RepoWindow>,
}

impl Render for MenuRowHost {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(action_row(
            "Settings...".to_owned(),
            Box::new(OpenSettings),
            false,
            false,
            &Theme::light(),
            &self.repo,
        ))
    }
}

#[gpui::test]
fn menu_item_click_dispatches_its_action(cx: &mut gpui::TestAppContext) {
    cx.update(|cx| {
        cx.set_global(Theme::light());
        cx.set_global(AppConfigStore::new(AppConfig::default()));
        crate::app::menus::install(cx);
    });
    let dir = tempfile::tempdir().unwrap();
    let (repo, _) =
        cx.add_window_view(|_, cx| RepoWindow::new_with_onboarding(dir.path().to_path_buf(), cx));
    let (_, cx) = cx.add_window_view(|_, _| MenuRowHost { repo });
    let cx: &mut VisualTestContext = cx;
    cx.update(|window, _| window.activate_window());
    cx.simulate_resize(size(px(300.), px(40.)));
    cx.run_until_parked();
    let row = cx
        .debug_bounds("app-menu-item-Settings...")
        .expect("menu row bounds");
    let windows_before = cx.windows().len();

    cx.simulate_click(row.center(), Modifiers::default());
    cx.run_until_parked();

    assert_eq!(cx.windows().len(), windows_before + 1);
}
