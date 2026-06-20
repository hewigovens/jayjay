use gpui::{
    AnyElement, App, ClickEvent, Div, InteractiveElement, IntoElement, ParentElement, SharedString,
    Stateful, StatefulInteractiveElement, Styled, UniformList, Window, div, px, rgb,
};

use crate::app::theme::Theme;
use crate::ui::icons;

/// A small labeled toggle button (icon + text) with active/inactive styling.
/// Shared by the diff view-mode toggle and the file-column tree/flat toggle.
pub fn toggle_button<F>(
    glyph_str: &'static str,
    tooltip: &'static str,
    id: &'static str,
    active: bool,
    t: &Theme,
    on_click: F,
) -> AnyElement
where
    F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
{
    let (bg, fg) = if active {
        (t.toggle_active_bg, t.toggle_active_fg)
    } else {
        (t.toggle_inactive_bg, t.toggle_inactive_fg)
    };
    div()
        .id(SharedString::from(format!("toggle-{id}")))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.))
        .px(px(8.))
        .py(px(3.))
        .rounded_sm()
        .bg(rgb(bg))
        .text_size(px(11.))
        .text_color(rgb(fg))
        .cursor_pointer()
        .on_click(on_click)
        .child(icons::icon(glyph_str, 14., fg))
        .child(tooltip)
        .into_any_element()
}

/// uniform_list reserves a 15px gutter for an OS scrollbar by default. We
/// don't render a scrollbar, so the gutter just leaves a thick gap on the
/// right edge — collapse it to 0.
pub fn no_scrollbar_gutter(mut list: UniformList) -> UniformList {
    list.style().scrollbar_width = Some(px(0.).into());
    list
}

pub fn capsule(
    label: impl Into<SharedString>,
    bg: u32,
    fg: u32,
    font_size: f32,
) -> impl IntoElement {
    div()
        .flex_none()
        .px(px(6.))
        .py(px(1.))
        .rounded_full()
        .bg(rgb(bg))
        .text_color(rgb(fg))
        .text_size(px(font_size))
        .child(label.into())
}

/// A rounded chip with a leading colored glyph (e.g. bookmark / tag pills).
/// Returns a `Div` so callers can chain `.id()` / `.on_mouse_down()`.
pub fn icon_chip(
    glyph_str: &'static str,
    label: impl Into<SharedString>,
    bg: u32,
    fg: u32,
    icon_color: u32,
    font_size: f32,
) -> Div {
    div()
        .flex_none()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(3.))
        .px(px(6.))
        .py(px(1.))
        .rounded_full()
        .bg(rgb(bg))
        .text_color(rgb(fg))
        .text_size(px(font_size))
        .child(icons::icon(glyph_str, font_size, icon_color))
        .child(label.into())
}

pub fn icon_label(
    glyph_str: &'static str,
    label: impl Into<SharedString>,
    icon_size: f32,
    icon_color: u32,
) -> Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.))
        .child(icons::icon(glyph_str, icon_size, icon_color))
        .child(label.into())
}

pub fn button(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    theme: &Theme,
    primary: bool,
) -> Stateful<Div> {
    let (bg, fg) = if primary {
        (theme.toggle_active_bg, theme.toggle_active_fg)
    } else {
        (theme.toggle_inactive_bg, theme.toggle_inactive_fg)
    };
    div()
        .id(id.into())
        .flex()
        .items_center()
        .justify_center()
        .px(px(10.))
        .h(px(28.))
        .rounded_sm()
        .bg(rgb(bg))
        .text_color(rgb(fg))
        .text_size(px(12.))
        .cursor_pointer()
        .hover(|s| s.bg(rgb(theme.row_alt_bg)))
        .child(label.into())
}

pub fn toolbar_button(id: impl Into<SharedString>, theme: &Theme) -> Stateful<Div> {
    div()
        .id(id.into())
        .flex()
        .items_center()
        .justify_center()
        .w(px(28.))
        .h(px(24.))
        .rounded_sm()
        .bg(rgb(theme.toolbar_icon_bg))
        .cursor_pointer()
        .hover(|s| s.bg(rgb(theme.row_alt_bg)))
        .active(|s| s.bg(rgb(theme.selected_bg)))
}

pub fn toolbar_icon_button(
    id: impl Into<SharedString>,
    glyph_str: &'static str,
    theme: &Theme,
) -> Stateful<Div> {
    toolbar_button(id, theme).child(icons::icon(glyph_str, 14., theme.fg_dim))
}

pub fn divider_h(theme: &Theme) -> impl IntoElement {
    div().h(px(1.)).w_full().bg(rgb(theme.border))
}

pub fn divider_v(theme: &Theme) -> impl IntoElement {
    div().w(px(1.)).h_full().bg(rgb(theme.border))
}
