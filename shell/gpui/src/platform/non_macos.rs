use gpui::{Context, ParentElement, Window};

use crate::app::theme::Theme;
use crate::repo::window::RepoWindow;

pub const MOD_KEY: &str = "ctrl";
/// Ctrl is already the platform Mod here, so a plain `ctrl-s` would collide with the file editor's save.
pub const SIDEBAR_TOGGLE_KEY: &str = "alt-ctrl-s";
pub const TOOLBAR_HEIGHT: f32 = 44.;
pub const TOOLBAR_LEADING_INSET: f32 = 12.;
pub const TOOLBAR_BUTTON_HEIGHT: f32 = 30.;
pub const TOOLBAR_ICON_SIZE: f32 = 16.;
pub const REPO_TRAFFIC_LIGHTS: (f32, f32) = (12., 14.);

pub fn append_menu_bar(
    root: gpui::Div,
    view: &RepoWindow,
    t: &Theme,
    window: &Window,
    cx: &mut Context<RepoWindow>,
) -> gpui::Div {
    root.child(crate::ui::app_menu::menu_bar(
        view.open_app_menu_name(),
        &view.app_menu_titles,
        t,
        window,
        cx,
    ))
}
