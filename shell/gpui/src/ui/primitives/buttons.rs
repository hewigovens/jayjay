use gpui::{
    AnyElement, App, ClickEvent, ClipboardItem, Div, InteractiveElement, IntoElement,
    ParentElement, Role, SharedString, Stateful, StatefulInteractiveElement, Styled, Toggled,
    Window, div, px, rgb, svg,
};

use crate::app::theme::{Theme, ui_size};
use crate::ui::icons;

pub(crate) fn button(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    theme: &Theme,
    primary: bool,
) -> Stateful<Div> {
    button_container(id, theme, primary).child(label.into())
}

pub(crate) fn small_button(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    theme: &Theme,
) -> Stateful<Div> {
    button_container(id, theme, false)
        .h(px(theme.scaled_control_height(22., 11.)))
        .px(ui_size(8.))
        .text_size(ui_size(11.))
        .child(label.into())
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
        .px(ui_size(10.))
        .h(px(theme.scaled_control_height(28., 12.)))
        .rounded_md()
        .bg(rgb(bg))
        .text_color(rgb(fg))
        .text_size(ui_size(12.))
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
    let chrome = div().child(icons::icon(glyph_str, icon_size, color));
    icon_button_frame(id, width, height, chrome).text_color(rgb(color))
}

/// Takes clicks at `hit_target` size while `chrome` keeps the `width`×`height` look.
fn icon_button_frame(
    id: impl Into<SharedString>,
    width: f32,
    height: f32,
    chrome: Div,
) -> Stateful<Div> {
    div()
        .id(id.into())
        .group(ICON_BUTTON_GROUP)
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .hit_target(width, height)
        .child(
            chrome
                .flex()
                .flex_none()
                .items_center()
                .justify_center()
                .w(ui_size(width))
                .h(ui_size(height))
                .rounded_md(),
        )
}

const ICON_BUTTON_GROUP: &str = "icon-button";

pub(crate) const MIN_HIT_TARGET: f32 = 24.;

pub(crate) trait HitTarget: Styled + Sized {
    /// Sizes the element to at least `MIN_HIT_TARGET` square, centred on a `width`×`height` slot, which is all the layout reserves for it.
    fn hit_target(self, width: f32, height: f32) -> Self {
        let (target_width, target_height) = (width.max(MIN_HIT_TARGET), height.max(MIN_HIT_TARGET));
        self.w(ui_size(target_width))
            .h(ui_size(target_height))
            .mx(ui_size((width - target_width) / 2.))
            .my(ui_size((height - target_height) / 2.))
    }
}

impl<E: Styled> HitTarget for E {}

pub(crate) fn icon_button(
    id: impl Into<SharedString>,
    glyph_str: &'static str,
    icon_size: f32,
    width: f32,
    height: f32,
    color: u32,
    theme: &Theme,
) -> Stateful<Div> {
    hovering_icon_button(
        id,
        icons::icon(glyph_str, icon_size, color),
        width,
        height,
        theme.row_alt_bg,
    )
    .text_color(rgb(color))
}

/// An icon button whose `fill` shows an on or off state.
pub(crate) fn toggle_icon_button(
    id: impl Into<SharedString>,
    glyph_str: &'static str,
    (fill, color): (u32, u32),
    theme: &Theme,
) -> Stateful<Div> {
    let chrome = div()
        .bg(rgb(fill))
        .group_hover(ICON_BUTTON_GROUP, |s| s.bg(rgb(theme.row_alt_bg)))
        .child(icons::icon(glyph_str, 14., color));
    icon_button_frame(id, 24., 22., chrome)
        .text_color(rgb(color))
        .cursor_pointer()
}

pub(crate) fn svg_icon_button(
    id: impl Into<SharedString>,
    path: &'static str,
    icon_size: f32,
    width: f32,
    height: f32,
    color: u32,
    theme: &Theme,
) -> Stateful<Div> {
    let icon = svg()
        .path(path)
        .size(ui_size(icon_size))
        .text_color(rgb(color));
    hovering_icon_button(id, icon, width, height, theme.row_alt_bg)
}

/// Copies `value`; `hover_fill` must stand out from the surface the button sits on.
pub(crate) fn copy_icon_button(
    id: impl Into<SharedString>,
    value: impl Into<String>,
    icon_size: f32,
    width: f32,
    height: f32,
    color: u32,
    hover_fill: u32,
) -> Stateful<Div> {
    let icon = icons::icon(icons::glyph::COPY, icon_size, color);
    copy_action(
        hovering_icon_button(id, icon, width, height, hover_fill).text_color(rgb(color)),
        value.into(),
    )
}

fn hovering_icon_button(
    id: impl Into<SharedString>,
    icon: impl IntoElement,
    width: f32,
    height: f32,
    hover_fill: u32,
) -> Stateful<Div> {
    let chrome = div()
        .group_hover(ICON_BUTTON_GROUP, |s| s.bg(rgb(hover_fill)))
        .child(icon);
    icon_button_frame(id, width, height, chrome).cursor_pointer()
}

fn copy_action(element: Stateful<Div>, value: String) -> Stateful<Div> {
    element.on_click(move |_, _, cx| {
        cx.write_to_clipboard(ClipboardItem::new_string(value.clone()));
    })
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
    let track_bg = if active {
        theme.switch_active_bg
    } else {
        theme.switch_inactive_bg
    };
    let toggled = if active {
        Toggled::True
    } else {
        Toggled::False
    };
    let id = id.into();

    let thumb = div()
        .flex_none()
        .w(ui_size(18.))
        .h(ui_size(18.))
        .rounded_full()
        .bg(rgb(theme.switch_thumb));

    let mut track = div()
        .flex()
        .flex_row()
        .items_center()
        .w(ui_size(44.))
        .h(ui_size(24.))
        .px(ui_size(2.))
        .rounded_full()
        .bg(rgb(track_bg))
        .border_1()
        .border_color(rgb(track_bg));
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
        .w(ui_size(48.))
        .h(ui_size(28.))
        .rounded_full()
        .cursor_pointer()
        .on_click(on_click)
        .child(track)
        .into_any_element()
}

pub(crate) use crate::platform::{TOOLBAR_BUTTON_HEIGHT, TOOLBAR_ICON_SIZE};

pub(crate) const TOOLBAR_BUTTON_WIDTH: f32 = 38.;
