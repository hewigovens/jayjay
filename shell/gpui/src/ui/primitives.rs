use gpui::{
    AnyElement, AnyView, App, AppContext, BoxShadow, ClickEvent, ClipboardItem, Context, Div,
    ElementId, InteractiveElement, IntoElement, ParentElement, Render, Role, SharedString,
    Stateful, StatefulInteractiveElement, Styled, Toggled, UniformList, Window, div, hsla, px, rgb,
    svg,
};

use crate::app::theme::{Theme, theme};
use crate::ui::icons;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CheckCircleState {
    Off,
    Partial,
    On,
}

/// Circular check shared by the file list's review mark and diff edit's selection checkbox; callers attach their own click handler.
pub fn check_circle(
    id: impl Into<ElementId>,
    state: CheckCircleState,
    accent: u32,
    t: &Theme,
) -> Stateful<Div> {
    let mut circle = div()
        .id(id)
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .w(px(14.))
        .h(px(14.))
        .rounded_full()
        .border_1()
        .border_color(rgb(if state == CheckCircleState::Off {
            t.fg_faint
        } else {
            accent
        }))
        .cursor_pointer();
    match state {
        CheckCircleState::Off => {}
        CheckCircleState::Partial => {
            circle = circle
                .bg(rgb(accent))
                .child(div().w(px(6.)).h(px(2.)).rounded_full().bg(rgb(0xffffff)));
        }
        CheckCircleState::On => {
            // An SVG check centers geometrically; the lucide text glyph sits visibly off-center in a 14px circle.
            circle = circle.bg(rgb(accent)).child(
                svg()
                    .path(icons::CHECK_SVG)
                    .w(px(8.))
                    .h(px(8.))
                    .text_color(rgb(0xffffff)),
            );
        }
    }
    circle
}

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
        .debug_selector(move || format!("toggle-{id}"))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.))
        .px(px(8.))
        .py(px(3.))
        .rounded_md()
        .bg(rgb(bg))
        .text_size(px(11.))
        .text_color(rgb(fg))
        .cursor_pointer()
        .on_click(on_click)
        .child(icons::icon(glyph_str, 14., fg))
        .child(tooltip)
        .into_any_element()
}

/// uniform_list reserves a 15px gutter for an OS scrollbar by default; we don't render one, so collapse it to 0.
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

/// Returns `Div`, not `impl IntoElement`, so callers can chain `.id()` / `.on_mouse_down()`.
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
        .rounded_md()
        .bg(rgb(bg))
        .text_color(rgb(fg))
        .text_size(px(12.))
        .cursor_pointer()
        .hover(|s| s.bg(rgb(theme.row_alt_bg)))
        .child(label.into())
}

/// `icon_button` without the pointer/hover chrome, for dimmed non-interactive states.
pub fn inert_icon_button(
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

pub fn icon_button(
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

pub fn copy_icon_button(
    id: impl Into<SharedString>,
    value: impl Into<String>,
    icon_size: f32,
    width: f32,
    height: f32,
    color: u32,
    theme: &Theme,
) -> Stateful<Div> {
    let value = value.into();
    icon_button(
        id,
        icons::glyph::COPY,
        icon_size,
        width,
        height,
        color,
        theme,
    )
    .on_click(move |_, _, cx| {
        cx.write_to_clipboard(ClipboardItem::new_string(value.clone()));
    })
}

pub fn checked_menu_row(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    checked: bool,
    unchecked_glyph: Option<(&'static str, u32)>,
    text_color: u32,
    checked_color: u32,
) -> Stateful<Div> {
    div()
        .id(id.into())
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.))
        .px(px(10.))
        .py(px(5.))
        .text_size(px(12.))
        .text_color(rgb(text_color))
        .child(menu_checkmark(checked, unchecked_glyph, checked_color))
        .child(div().flex_1().min_w_0().truncate().child(label.into()))
}

fn menu_checkmark(
    checked: bool,
    unchecked_glyph: Option<(&'static str, u32)>,
    checked_color: u32,
) -> AnyElement {
    let marker = if checked {
        icons::icon(icons::glyph::CHECK, 12., checked_color).into_any_element()
    } else if let Some((glyph_str, color)) = unchecked_glyph {
        icons::icon(glyph_str, 12., color).into_any_element()
    } else {
        div().into_any_element()
    };

    div()
        .flex_none()
        .w(px(14.))
        .child(marker)
        .into_any_element()
}

pub fn boolean_toggle_button<F>(
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

pub fn text_tooltip(label: impl Into<SharedString>) -> impl Fn(&mut Window, &mut App) -> AnyView {
    let label = label.into();
    move |_, cx| {
        cx.new(|_| TextTooltip {
            label: label.clone(),
        })
        .into()
    }
}

pub const TOOLBAR_BUTTON_HEIGHT: f32 = 30.;
pub const TOOLBAR_BUTTON_WIDTH: f32 = 38.;
pub const TOOLBAR_ICON_SIZE: f32 = 16.;

struct TextTooltip {
    label: SharedString,
}

impl Render for TextTooltip {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = theme(cx);
        div().pl(px(6.)).pt(px(8.)).child(
            div()
                .px(px(8.))
                .py(px(4.))
                .rounded_sm()
                .bg(rgb(theme.detail_bg))
                .border_1()
                .border_color(rgb(theme.border))
                .shadow(tooltip_shadow(theme))
                .text_size(px(11.))
                .text_color(rgb(theme.fg))
                .child(self.label.clone()),
        )
    }
}

fn tooltip_shadow(theme: &Theme) -> Vec<BoxShadow> {
    let (wide, tight) = if theme.is_dark {
        (0.26, 0.2)
    } else {
        (0.12, 0.06)
    };
    vec![
        BoxShadow::new(px(0.), px(6.), hsla(0., 0., 0., wide)).blur_radius(px(18.)),
        BoxShadow::new(px(0.), px(1.), hsla(0., 0., 0., tight)),
    ]
}

pub fn divider_h(theme: &Theme) -> impl IntoElement {
    div().h(px(1.)).w_full().bg(rgb(theme.border))
}

pub fn divider_v(theme: &Theme) -> impl IntoElement {
    div().w(px(1.)).h_full().bg(rgb(theme.border))
}
