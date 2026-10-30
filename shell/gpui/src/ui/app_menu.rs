use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gpui::{
    Action, Anchor, AnyElement, App, Bounds, Entity, InteractiveElement, IntoElement, MouseButton,
    MouseDownEvent, OwnedMenu, OwnedMenuItem, ParentElement, Pixels, Point, SharedString,
    StatefulInteractiveElement, Styled, Window, anchored, canvas, deferred, div,
    prelude::FluentBuilder as _, px, rgb,
};

use crate::app::key_caps::KeyCaps;
use crate::app::theme::{Theme, ui_font_size};
use crate::repo::window::RepoWindow;
use crate::repo::window::picker::{below, opener};
use crate::ui::icons::{self, glyph};
use crate::ui::logo::Logo;
use crate::ui::title_bar;

/// Where each menu bar title was last painted, so menus open under their title however they were opened.
pub(crate) type MenuTitleBounds = Rc<RefCell<HashMap<SharedString, Bounds<Pixels>>>>;

#[derive(Clone)]
pub struct AppMenuState {
    pub(crate) menu_name: SharedString,
    pub(crate) selected: Option<usize>,
}

/// One line of an open menu; submenus are flattened under their label.
pub(crate) enum MenuRow {
    Action {
        name: String,
        action: Box<dyn Action>,
        checked: bool,
        disabled: bool,
    },
    Label(SharedString),
    Separator,
}

impl MenuRow {
    pub(crate) fn for_menu(name: &SharedString, cx: &App) -> Vec<Self> {
        let mut rows = Vec::new();
        if let Some(menu) = cx
            .get_menus()
            .unwrap_or_default()
            .into_iter()
            .find(|menu| menu.name == *name)
        {
            Self::append(&menu, &mut rows);
        }
        rows
    }

    fn append(menu: &OwnedMenu, rows: &mut Vec<Self>) {
        for item in clean_items(&menu.items) {
            match item {
                OwnedMenuItem::Separator => rows.push(Self::Separator),
                OwnedMenuItem::Action {
                    name,
                    action,
                    checked,
                    disabled,
                    ..
                } => rows.push(Self::Action {
                    name,
                    action,
                    checked,
                    disabled,
                }),
                OwnedMenuItem::Submenu(submenu) => {
                    rows.push(Self::Label(submenu.name.clone()));
                    Self::append(&submenu, rows);
                }
                OwnedMenuItem::SystemMenu(_) => {}
            }
        }
    }

    pub(crate) fn is_selectable(&self) -> bool {
        matches!(
            self,
            Self::Action {
                disabled: false,
                ..
            }
        )
    }

    /// The next selectable row after `from` in the given direction, wrapping at the ends.
    pub(crate) fn step(rows: &[Self], from: Option<usize>, forward: bool) -> Option<usize> {
        let selectable: Vec<usize> = (0..rows.len())
            .filter(|&ix| rows[ix].is_selectable())
            .collect();
        let position = from.and_then(|from| selectable.iter().position(|&ix| ix == from));
        let next = match (position, forward) {
            (None, true) => 0,
            (None, false) => selectable.len().checked_sub(1)?,
            (Some(position), true) => (position + 1) % selectable.len(),
            (Some(position), false) => position.checked_sub(1).unwrap_or(selectable.len() - 1),
        };
        selectable.get(next).copied()
    }
}

pub(crate) fn render_app_menu(
    state: &AppMenuState,
    titles: &MenuTitleBounds,
    t: &Theme,
    view: &Entity<RepoWindow>,
    cx: &mut gpui::Context<RepoWindow>,
) -> AnyElement {
    let rows = MenuRow::for_menu(&state.menu_name, cx);
    // Read at render time so the menu follows its title when zoom resizes the bar.
    let anchor = titles
        .borrow()
        .get(&state.menu_name)
        .map_or_else(Point::default, |title| below(*title));
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
        .position(anchor)
        .snap_to_window_with_margin(px(6.))
        .child(menu_panel(rows, state.selected, t, view, cx));

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

pub(crate) fn menu_bar(
    open_menu: Option<&SharedString>,
    titles: &MenuTitleBounds,
    t: &Theme,
    window: &Window,
    cx: &mut gpui::Context<RepoWindow>,
) -> AnyElement {
    let menus = cx.get_menus().unwrap_or_default();
    let row = if title_bar::DRAWN {
        title_bar::drag_strip("app-menu-bar")
            .h(px(t.scaled_control_height(28., 12.).max(32.)))
            .child(title_bar::system_menu_icon(Logo::load(cx).image(16.)).mr(px(6.)))
    } else {
        div()
            .id("app-menu-bar")
            .flex()
            .flex_row()
            .items_center()
            .w_full()
            .h(px(t.scaled_control_height(28., 12.)))
            .pr(px(8.))
    };
    let mut row = row
        .debug_selector(|| "app-menu-bar".to_owned())
        .gap(px(2.))
        .pl(px(8.))
        .bg(rgb(t.header_bg))
        .border_b_1()
        .border_color(rgb(t.border));

    let mut items = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(2.))
        .min_w_0()
        .overflow_hidden();
    for menu in menus {
        let is_open = open_menu == Some(&menu.name);
        items = items.child(menu_bar_item(menu.name.clone(), is_open, titles, t, cx));
    }
    row = row.child(items);
    if title_bar::DRAWN {
        row = row
            .child(div().flex_1())
            .child(title_bar::caption_buttons(t, window));
    }

    row.into_any_element()
}

fn menu_bar_item(
    name: SharedString,
    is_open: bool,
    titles: &MenuTitleBounds,
    t: &Theme,
    cx: &mut gpui::Context<RepoWindow>,
) -> AnyElement {
    let id = SharedString::from(format!(
        "app-menu-bar-{}",
        name.as_ref().to_ascii_lowercase()
    ));
    let clicked = name.clone();
    let hovered = name.clone();
    let shown = name.clone();
    let painted = (name.clone(), titles.clone());
    let title = div()
        .id(id)
        .debug_selector({
            let name = name.clone();
            move || format!("app-menu-bar-{}", name.as_ref())
        })
        .relative()
        .flex()
        .items_center()
        .h(px(t.scaled_control_height(22., 12.)))
        .px(px(8.))
        .rounded_sm()
        .text_size(ui_font_size(12.))
        .text_color(rgb(if is_open { t.fg } else { t.fg_dim }))
        .when(is_open, |row| row.bg(rgb(t.row_alt_bg)))
        .cursor_pointer()
        .occlude()
        .hover(|s| s.bg(rgb(t.row_alt_bg)))
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |view, _: &MouseDownEvent, _w, cx| {
                view.open_named_app_menu(clicked.clone(), None, cx);
            }),
        )
        .on_hover(cx.listener(move |view, hovering: &bool, _w, cx| {
            if *hovering
                && view
                    .open_app_menu_name()
                    .is_some_and(|open| *open != hovered)
            {
                view.open_named_app_menu(hovered.clone(), None, cx);
            }
        }))
        .child(
            canvas(
                move |bounds, window, _| {
                    let (name, titles) = &painted;
                    if titles.borrow().get(name) != Some(&bounds) {
                        titles.borrow_mut().insert(name.clone(), bounds);
                        window.request_animation_frame();
                    }
                },
                |_, _, _, _| {},
            )
            .absolute()
            .inset_0(),
        )
        .child(name);
    opener(
        title,
        move |view| view.open_app_menu_name() == Some(&shown),
        RepoWindow::close_app_menu,
        cx,
    )
    .into_any_element()
}

fn menu_panel(
    rows: Vec<MenuRow>,
    selected: Option<usize>,
    t: &Theme,
    view: &Entity<RepoWindow>,
    cx: &App,
) -> AnyElement {
    let shows_checks = rows
        .iter()
        .any(|row| matches!(row, MenuRow::Action { checked: true, .. }));
    let mut col = div()
        .debug_selector(|| "app-menu-panel".to_owned())
        .occlude()
        .flex()
        .flex_col()
        .min_w(px(240.))
        .max_w(px(360.))
        .p(px(4.))
        .bg(rgb(t.detail_bg))
        .border_1()
        .border_color(rgb(t.border))
        .rounded_lg()
        .shadow_lg();

    for (ix, row) in rows.into_iter().enumerate() {
        col = col.child(match row {
            MenuRow::Separator => separator(t),
            MenuRow::Label(name) => label(name, t),
            MenuRow::Action {
                name,
                action,
                checked,
                disabled,
            } => {
                let shortcut = KeyCaps::label(action.as_ref(), cx);
                action_row(ActionRow {
                    ix,
                    name,
                    action,
                    shortcut,
                    checked: shows_checks.then_some(checked),
                    disabled,
                    selected: selected == Some(ix),
                    t,
                    view,
                })
            }
        });
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

struct ActionRow<'a> {
    ix: usize,
    name: String,
    action: Box<dyn Action>,
    shortcut: String,
    /// `None` when the menu has no checkable items, so no check column is drawn.
    checked: Option<bool>,
    disabled: bool,
    selected: bool,
    t: &'a Theme,
    view: &'a Entity<RepoWindow>,
}

fn action_row(row: ActionRow<'_>) -> AnyElement {
    let ActionRow {
        ix,
        name,
        action,
        shortcut,
        checked,
        disabled,
        selected,
        t,
        view,
    } = row;
    let element = div()
        .id(SharedString::from(format!("app-menu-item-{name}")))
        .debug_selector({
            let name = name.clone();
            move || format!("app-menu-item-{name}")
        })
        .flex()
        .items_center()
        .gap(px(10.))
        .h(px(t.scaled_control_height(28., 12.)))
        .px(px(10.))
        .rounded(px(5.))
        .text_size(ui_font_size(12.))
        .text_color(rgb(if disabled { t.fg_faint } else { t.fg }))
        .when(selected, |row| row.bg(rgb(t.selected_bg)))
        .when_some(checked, |row, checked| {
            row.child(div().flex_none().w(px(12.)).when(checked, |cell| {
                cell.child(icons::icon(glyph::CHECK, 12., t.fg_dim))
            }))
        })
        .child(div().flex_1().min_w_0().truncate().child(name))
        .when(!shortcut.is_empty(), |row| {
            row.child(
                div()
                    .flex_none()
                    .text_color(rgb(t.fg_faint))
                    .child(shortcut),
            )
        });

    if disabled {
        return element.into_any_element();
    }
    let hovered_view = view.clone();
    let clicked_view = view.clone();
    element
        .cursor_pointer()
        .on_hover(move |hovering, _, cx| {
            if *hovering {
                hovered_view.update(cx, |this, cx| this.select_app_menu_row(ix, cx));
            }
        })
        .on_mouse_down(MouseButton::Left, move |_: &MouseDownEvent, window, cx| {
            cx.stop_propagation();
            clicked_view.update(cx, |this, cx| this.close_app_menu(cx));
            // App-level dispatch re-enters this window's update from inside its own event handler and fails silently.
            window.dispatch_action(action.boxed_clone(), cx);
        })
        .into_any_element()
}

fn label(name: SharedString, t: &Theme) -> AnyElement {
    div()
        .px(px(10.))
        .pt(px(6.))
        .pb(px(2.))
        .text_size(ui_font_size(11.))
        .text_color(rgb(t.fg_faint))
        .child(name)
        .into_any_element()
}

fn separator(t: &Theme) -> AnyElement {
    div()
        .mx(px(6.))
        .my(px(4.))
        .h(px(1.))
        .bg(rgb(t.border))
        .into_any_element()
}

#[cfg(test)]
mod tests;
