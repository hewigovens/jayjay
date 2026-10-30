use gpui::{
    Context, Entity, IntoElement, Modifiers, ParentElement, Render, Styled, VisualTestContext,
    Window, div, px, size,
};

use super::{ActionRow, MenuRow, action_row};
use crate::app::actions::OpenSettings;
use crate::app::config::{AppConfig, AppConfigStore};
use crate::app::theme::Theme;
use crate::repo::window::RepoWindow;

struct MenuRowHost {
    repo: Entity<RepoWindow>,
}

impl Render for MenuRowHost {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(action_row(ActionRow {
            ix: 0,
            name: "Settings...".to_owned(),
            action: Box::new(OpenSettings),
            shortcut: String::new(),
            checked: None,
            disabled: false,
            selected: false,
            t: &Theme::light(),
            view: &self.repo,
        }))
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

#[gpui::test]
fn menu_bar_menus_open_under_their_title(cx: &mut gpui::TestAppContext) {
    cx.update(|cx| {
        cx.set_global(Theme::light());
        cx.set_global(AppConfigStore::new(AppConfig::default()));
        crate::app::menus::install(cx);
    });
    let dir = tempfile::tempdir().unwrap();
    let (repo, cx) = cx.add_window_view(|_, cx| RepoWindow::new(dir.path().to_path_buf(), cx));
    let cx: &mut VisualTestContext = cx;
    cx.update(|window, cx| {
        let handle = repo.read(cx).focus_handle.clone();
        window.focus(&handle, cx);
    });
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

    cx.simulate_keystrokes("left");
    assert!(opens_under(panel_origin(cx), first));
    cx.simulate_keystrokes("right");
    assert!(opens_under(panel_origin(cx), second));

    cx.simulate_click(second.center(), Modifiers::default());
    cx.run_until_parked();
    assert_eq!(panel_origin(cx), None);
}

#[test]
fn menu_selection_skips_rows_that_cannot_run_and_wraps() {
    let action = |disabled| MenuRow::Action {
        name: "Settings...".to_owned(),
        action: Box::new(OpenSettings),
        checked: false,
        disabled,
    };
    let rows = [
        MenuRow::Label("Zoom".into()),
        action(true),
        MenuRow::Separator,
        action(false),
        action(false),
    ];

    assert_eq!(MenuRow::step(&rows, None, true), Some(3));
    assert_eq!(MenuRow::step(&rows, None, false), Some(4));
    assert_eq!(MenuRow::step(&rows, Some(4), true), Some(3));
    assert_eq!(MenuRow::step(&rows, Some(3), false), Some(4));
    assert_eq!(MenuRow::step(&rows[..3], None, true), None);
}
