use gpui::{
    Action, Anchor, AnyElement, Entity, InteractiveElement, IntoElement, MouseButton,
    MouseDownEvent, OwnedMenu, OwnedMenuItem, ParentElement, Pixels, Point, SharedString, Styled,
    anchored, deferred, div, px, rgb,
};

use crate::app::theme::{Theme, ui_font_size};
use crate::repo::window::RepoWindow;
use crate::ui::primitives::checked_menu_row;

#[derive(Clone)]
pub struct AppMenuState {
    pub(crate) anchor: Point<Pixels>,
    pub(crate) menu_name: Option<SharedString>,
}

pub(crate) fn render_app_menu(
    state: &AppMenuState,
    t: &Theme,
    view: &Entity<RepoWindow>,
    cx: &mut gpui::Context<RepoWindow>,
) -> AnyElement {
    let mut menus = cx.get_menus().unwrap_or_default();
    if let Some(menu_name) = state.menu_name.as_ref() {
        menus.retain(|menu| menu.name == *menu_name);
    }
    let backdrop_left_view = view.clone();
    let backdrop_right_view = view.clone();
    let backdrop = div()
        .id("app-menu-backdrop")
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .on_mouse_down(MouseButton::Left, move |_: &MouseDownEvent, _, cx| {
            backdrop_left_view.update(cx, |this, cx| this.close_app_menu(cx));
        })
        .on_mouse_down(MouseButton::Right, move |_: &MouseDownEvent, _, cx| {
            backdrop_right_view.update(cx, |this, cx| this.close_app_menu(cx));
        });

    let menu = anchored()
        .anchor(Anchor::TopLeft)
        .position(state.anchor)
        .snap_to_window_with_margin(px(6.))
        .child(menu_panel(&menus, t, view));

    deferred(
        div()
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .child(backdrop)
            .child(menu),
    )
    .with_priority(2)
    .into_any_element()
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn menu_bar(t: &Theme, cx: &mut gpui::Context<RepoWindow>) -> AnyElement {
    let menus = cx.get_menus().unwrap_or_default();
    let mut row = div()
        .id("app-menu-bar")
        .debug_selector(|| "app-menu-bar".to_owned())
        .flex()
        .flex_row()
        .items_center()
        .gap(px(2.))
        .w_full()
        .h(px(t.scaled_control_height(28., 12.)))
        .px(px(8.))
        .bg(rgb(t.header_bg))
        .border_b_1()
        .border_color(rgb(t.border));

    for menu in menus {
        let name = menu.name.clone();
        row = row.child(menu_bar_item(name, t, cx));
    }

    row.into_any_element()
}

#[cfg(not(target_os = "macos"))]
fn menu_bar_item(name: SharedString, t: &Theme, cx: &mut gpui::Context<RepoWindow>) -> AnyElement {
    let id = SharedString::from(format!(
        "app-menu-bar-{}",
        name.as_ref().to_ascii_lowercase()
    ));
    let clicked_name = name.clone();
    div()
        .id(id)
        .debug_selector({
            let name = name.clone();
            move || format!("app-menu-bar-{}", name.as_ref())
        })
        .flex()
        .items_center()
        .h(px(t.scaled_control_height(22., 12.)))
        .px(px(8.))
        .rounded_sm()
        .text_size(ui_font_size(12.))
        .text_color(rgb(t.fg_dim))
        .cursor_pointer()
        .hover(|s| s.bg(rgb(t.row_alt_bg)))
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |view, ev: &MouseDownEvent, _w, cx| {
                view.open_named_app_menu(clicked_name.clone(), ev.position, cx);
            }),
        )
        .child(name)
        .into_any_element()
}

fn menu_panel(menus: &[OwnedMenu], t: &Theme, view: &Entity<RepoWindow>) -> AnyElement {
    let mut col = div()
        .debug_selector(|| "app-menu-panel".to_owned())
        .flex()
        .flex_col()
        .min_w(px(240.))
        .max_w(px(360.))
        .py(px(6.))
        .bg(rgb(t.detail_bg))
        .border_1()
        .border_color(rgb(t.border))
        .rounded_sm();

    for (ix, menu) in menus.iter().enumerate() {
        if ix > 0 {
            col = col.child(section_gap(t));
        }
        col = col.child(menu_header(menu, t));
        for item in clean_items(&menu.items) {
            col = append_item(col, item, t, view);
        }
    }

    col.into_any_element()
}

fn clean_items(items: &[OwnedMenuItem]) -> Vec<OwnedMenuItem> {
    let mut cleaned = Vec::new();
    let mut last_was_separator = true;
    for item in items {
        match item {
            OwnedMenuItem::Separator => {
                if !last_was_separator {
                    cleaned.push(item.clone());
                    last_was_separator = true;
                }
            }
            OwnedMenuItem::SystemMenu(_) => {}
            OwnedMenuItem::Submenu(menu) if menu.items.is_empty() => {}
            item => {
                cleaned.push(item.clone());
                last_was_separator = false;
            }
        }
    }
    if matches!(cleaned.last(), Some(OwnedMenuItem::Separator)) {
        cleaned.pop();
    }
    cleaned
}

fn append_item(
    col: gpui::Div,
    item: OwnedMenuItem,
    t: &Theme,
    view: &Entity<RepoWindow>,
) -> gpui::Div {
    match item {
        OwnedMenuItem::Separator => col.child(separator(t)),
        OwnedMenuItem::Action {
            name,
            action,
            checked,
            disabled,
            ..
        } => col.child(action_row(name, action, checked, disabled, t, view)),
        OwnedMenuItem::Submenu(menu) => append_submenu(col, menu, t, view),
        OwnedMenuItem::SystemMenu(_) => col,
    }
}

fn append_submenu(
    mut col: gpui::Div,
    menu: OwnedMenu,
    t: &Theme,
    view: &Entity<RepoWindow>,
) -> gpui::Div {
    col = col.child(submenu_header(menu.name, t));
    for item in clean_items(&menu.items) {
        col = append_item(col, item, t, view);
    }
    col
}

fn menu_header(menu: &OwnedMenu, t: &Theme) -> AnyElement {
    div()
        .px(px(10.))
        .pt(px(4.))
        .pb(px(3.))
        .text_size(ui_font_size(11.))
        .text_color(rgb(t.fg_faint))
        .child(menu.name.clone())
        .into_any_element()
}

fn submenu_header(name: SharedString, t: &Theme) -> AnyElement {
    div()
        .px(px(10.))
        .pt(px(6.))
        .pb(px(2.))
        .text_size(ui_font_size(11.))
        .text_color(rgb(t.fg_faint))
        .child(name)
        .into_any_element()
}

fn action_row(
    name: String,
    action: Box<dyn Action>,
    checked: bool,
    disabled: bool,
    t: &Theme,
    view: &Entity<RepoWindow>,
) -> AnyElement {
    let color = if disabled { t.fg_faint } else { t.fg };
    let icon_color = if disabled { t.fg_faint } else { t.fg_dim };
    let row = checked_menu_row(
        SharedString::from(format!("app-menu-item-{name}")),
        SharedString::from(name.clone()),
        checked,
        None,
        color,
        icon_color,
    )
    .debug_selector({
        let name = name.clone();
        move || format!("app-menu-item-{name}")
    });

    if disabled {
        row.into_any_element()
    } else {
        let view = view.clone();
        row.cursor_pointer()
            .hover(|s| s.bg(rgb(t.selected_bg)))
            .on_mouse_down(MouseButton::Left, move |_: &MouseDownEvent, window, cx| {
                cx.stop_propagation();
                view.update(cx, |this, cx| this.close_app_menu(cx));
                // App-level dispatch re-enters this window's update from inside its own event handler and fails silently.
                window.dispatch_action(action.boxed_clone(), cx);
            })
            .into_any_element()
    }
}

fn separator(t: &Theme) -> AnyElement {
    div()
        .mx(px(8.))
        .my(px(4.))
        .h(px(1.))
        .bg(rgb(t.border))
        .into_any_element()
}

fn section_gap(t: &Theme) -> AnyElement {
    div()
        .mx(px(8.))
        .mt(px(5.))
        .h(px(1.))
        .bg(rgb(t.border))
        .into_any_element()
}

#[cfg(test)]
mod tests;
