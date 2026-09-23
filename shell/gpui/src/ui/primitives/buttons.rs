use gpui::{
    AnyElement, App, ClickEvent, ClipboardItem, Div, InteractiveElement, IntoElement,
    ParentElement, Role, SharedString, Stateful, StatefulInteractiveElement, Styled, Toggled,
    Window, div, px, rgb,
};

use crate::app::theme::{Theme, ui_font_size};
use crate::ui::icons;

pub(crate) fn button(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    theme: &Theme,
    primary: bool,
) -> Stateful<Div> {
    button_container(id, theme, primary).child(label.into())
}

pub(crate) fn button_container(
    id: impl Into<SharedString>,
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
        .h(px(theme.scaled_control_height(28., 12.)))
        .rounded_md()
        .bg(rgb(bg))
        .text_color(rgb(fg))
        .text_size(ui_font_size(12.))
        .cursor_pointer()
        .hover(|s| s.bg(rgb(theme.row_alt_bg)))
}

/// `icon_button` without the pointer/hover chrome, for dimmed non-interactive states.
pub(crate) fn inert_icon_button(
    id: impl Into<SharedString>,
    glyph_str: &'static str,
    icon_size: f32,
    width: f32,
    height: f32,
    color: u32,
) -> Stateful<Div> {
    div()
        .id(id.into())
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .w(px(width))
        .h(px(height))
        .rounded_md()
        .text_color(rgb(color))
        .child(icons::icon(glyph_str, icon_size, color))
}

pub(crate) fn icon_button(
    id: impl Into<SharedString>,
    glyph_str: &'static str,
    icon_size: f32,
    width: f32,
    height: f32,
    color: u32,
    theme: &Theme,
) -> Stateful<Div> {
    inert_icon_button(id, glyph_str, icon_size, width, height, color)
        .cursor_pointer()
        .hover(|s| s.bg(rgb(theme.row_alt_bg)))
}

pub(crate) fn copy_icon_button(
    id: impl Into<SharedString>,
    value: impl Into<String>,
    icon_size: f32,
    width: f32,
    height: f32,
    color: u32,
    theme: &Theme,
) -> Stateful<Div> {
    copy_action(
        icon_button(
            id,
            icons::glyph::COPY,
            icon_size,
            width,
            height,
            color,
            theme,
        ),
        value.into(),
    )
}

fn copy_action(element: Stateful<Div>, value: String) -> Stateful<Div> {
    element.on_click(move |_, _, cx| {
        cx.write_to_clipboard(ClipboardItem::new_string(value.clone()));
    })
}

pub(crate) fn toggle_button<F>(
    glyph_str: &'static str,
    tooltip: &'static str,
    id: &'static str,
    active: bool,
    t: &Theme,
    on_click: F,
) -> Stateful<Div>
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
        .debug_selector(move || format!("toggle-{id}"))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(5.))
        .px(px(8.))
        .py(px(3.))
        .rounded_md()
        .bg(rgb(bg))
        .text_size(ui_font_size(11.))
        .line_height(px(t.scaled_font_size(14.)))
        .text_color(rgb(fg))
        .cursor_pointer()
        .on_click(on_click)
        .child(icons::icon(glyph_str, 11., fg))
        .child(tooltip)
}

pub(crate) fn boolean_toggle_button<F>(
    id: impl Into<SharedString>,
    active: bool,
    theme: &Theme,
    on_click: F,
) -> AnyElement
where
    F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
{
    let (track_bg, border_color, thumb_color) = if active {
        (theme.toggle_active_bg, theme.toggle_active_fg, theme.fg)
    } else {
        (theme.toggle_inactive_bg, theme.border, theme.fg_dim)
    };
    let toggled = if active {
        Toggled::True
    } else {
        Toggled::False
    };
    let id = id.into();

    let thumb = div()
        .flex_none()
        .w(px(18.))
        .h(px(18.))
        .rounded_full()
        .bg(rgb(thumb_color));

    let mut track = div()
        .flex()
        .flex_row()
        .items_center()
        .w(px(44.))
        .h(px(24.))
        .px(px(2.))
        .rounded_full()
        .bg(rgb(track_bg))
        .border_1()
        .border_color(rgb(border_color));
    track = (if active {
        track.justify_end()
    } else {
        track.justify_start()
    })
    .child(thumb);

    div()
        .id(id.clone())
        .debug_selector(move || id.to_string())
        .flex()
        .items_center()
        .justify_center()
        .role(Role::Switch)
        .aria_toggled(toggled)
        .w(px(48.))
        .h(px(28.))
        .rounded_full()
        .cursor_pointer()
        .on_click(on_click)
        .child(track)
        .into_any_element()
}

pub(crate) const TOOLBAR_BUTTON_HEIGHT: f32 = 30.;

pub(crate) const TOOLBAR_BUTTON_WIDTH: f32 = 38.;

pub(crate) const TOOLBAR_ICON_SIZE: f32 = 16.;
