#[cfg(not(target_os = "macos"))]
use gpui::{AnyElement, IntoElement};
use gpui::{
    Div, ElementId, InteractiveElement, ParentElement, SharedString, Stateful, Styled, div, px,
    rgb, svg,
};

use crate::app::theme::{Theme, ui_font_size};
use crate::ui::icons;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CheckCircleState {
    Off,
    /// Filled with a dash: some of the selection is on.
    Partial,
    /// Outlined check: the parts seen so far are done, but not all of them.
    CheckOutline,
    /// Left half filled: done before, changed since.
    HalfFilled,
    On,
}

/// Circular check shared by the file list's review mark and diff edit's selection checkbox; callers attach their own click handler.
pub(crate) fn check_circle(
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
        CheckCircleState::CheckOutline => {
            circle = circle.child(check_mark(accent));
        }
        CheckCircleState::HalfFilled => {
            // The half disc fills its whole viewBox, so at the 12px inner size it meets the border with no gap.
            circle = circle.child(
                svg()
                    .path(icons::CIRCLE_HALF_SVG)
                    .w(px(12.))
                    .h(px(12.))
                    .text_color(rgb(accent)),
            );
        }
        CheckCircleState::On => {
            circle = circle.bg(rgb(accent)).child(check_mark(0xffffff));
        }
    }
    circle
}

// An SVG check centers geometrically; the lucide text glyph sits visibly off-center in a 14px circle.

fn check_mark(color: u32) -> gpui::Svg {
    svg()
        .path(icons::CHECK_SVG)
        .w(px(8.))
        .h(px(8.))
        .text_color(rgb(color))
}

pub(crate) fn checkbox_row(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    checked: bool,
    theme: &Theme,
) -> Stateful<Div> {
    let id = id.into();
    let debug_id = id.clone();
    let mut box_glyph = div()
        .flex_none()
        .w(px(14.))
        .h(px(14.))
        .rounded(px(3.))
        .border_1()
        .border_color(rgb(theme.border))
        .flex()
        .items_center()
        .justify_center();
    if checked {
        box_glyph = box_glyph.bg(rgb(theme.toggle_active_bg)).child(icons::icon(
            icons::glyph::CHECK,
            10.,
            theme.toggle_active_fg,
        ));
    }
    div()
        .id(id)
        .debug_selector(move || debug_id.to_string())
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.))
        .cursor_pointer()
        .child(box_glyph)
        .child(
            div()
                .text_size(ui_font_size(12.))
                .text_color(rgb(theme.fg))
                .child(label.into()),
        )
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn checked_menu_row(
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
        .text_size(ui_font_size(12.))
        .text_color(rgb(text_color))
        .child(menu_checkmark(checked, unchecked_glyph, checked_color))
        .child(div().flex_1().min_w_0().truncate().child(label.into()))
}

#[cfg(not(target_os = "macos"))]
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
