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
    let (repo, _) = cx.add_window_view(|_, cx| RepoWindow::new(dir.path().to_path_buf(), cx));
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

#[cfg(not(target_os = "macos"))]
#[gpui::test]
fn menu_bar_menus_open_under_their_title(cx: &mut gpui::TestAppContext) {
    cx.update(|cx| {
        cx.set_global(Theme::light());
        cx.set_global(AppConfigStore::new(AppConfig::default()));
        crate::app::menus::install(cx);
    });
    let dir = tempfile::tempdir().unwrap();
    let (_, cx) = cx.add_window_view(|_, cx| RepoWindow::new(dir.path().to_path_buf(), cx));
    let cx: &mut VisualTestContext = cx;
    cx.simulate_resize(size(px(900.), px(600.)));
    cx.run_until_parked();
    let title = |cx: &mut VisualTestContext, selector: &'static str| {
        cx.debug_bounds(selector).expect("menu title bounds")
    };
    let panel_origin = |cx: &mut VisualTestContext| {
        cx.debug_bounds("app-menu-panel")
            .map(|bounds| bounds.origin)
    };
    // The panel snaps to whole pixels.
    let opens_under = |origin: Option<gpui::Point<gpui::Pixels>>,
                       title: gpui::Bounds<gpui::Pixels>| {
        origin.is_some_and(|origin| {
            (origin.x - title.left()).abs() < px(1.)
                && (origin.y - title.bottom() - px(4.)).abs() < px(1.)
        })
    };

    let first = title(cx, "app-menu-bar-Edit");
    cx.simulate_click(
        first.bottom_right() - gpui::point(px(2.), px(2.)),
        Modifiers::default(),
    );
    cx.run_until_parked();
    assert!(opens_under(panel_origin(cx), first));

    let second = title(cx, "app-menu-bar-View");
    cx.simulate_mouse_move(second.center(), None, Modifiers::default());
    cx.run_until_parked();
    assert!(opens_under(panel_origin(cx), second));

    cx.simulate_click(second.center(), Modifiers::default());
    cx.run_until_parked();
    assert_eq!(panel_origin(cx), None);
}
