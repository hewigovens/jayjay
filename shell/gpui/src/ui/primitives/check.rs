use gpui::{
    Div, ElementId, InteractiveElement, ParentElement, SharedString, Stateful, Styled, div, rgb,
    svg,
};

use crate::app::theme::{Theme, ui_size};
use crate::ui::icons;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CheckCircleState {
    Off,
    /// Filled with a dash: some of the selection is on.
    Partial,
    On,
}

/// Circular check for diff edit's selection checkbox; callers attach their own click handler.
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
        .w(ui_size(14.))
        .h(ui_size(14.))
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
            circle = circle.bg(rgb(accent)).child(
                div()
                    .w(ui_size(6.))
                    .h(ui_size(2.))
                    .rounded_full()
                    .bg(rgb(0xffffff)),
            );
        }
        CheckCircleState::On => {
            circle = circle
                .bg(rgb(accent))
                .child(mark_svg(icons::CHECK_SVG, 0xffffff));
        }
    }
    circle
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CheckSquareState {
    Off,
    /// Outlined dash: some of it is done.
    Dash,
    /// Outlined dot: done before, changed since.
    Dot,
    On,
}

/// Outlined square review mark.
pub(crate) fn check_square(
    id: impl Into<ElementId>,
    state: CheckSquareState,
    accent: u32,
    t: &Theme,
) -> Stateful<Div> {
    let square = div()
        .id(id)
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .w(ui_size(14.))
        .h(ui_size(14.))
        .rounded(ui_size(3.))
        .border_1()
        .border_color(rgb(if state == CheckSquareState::Off {
            t.fg_faint
        } else {
            accent
        }))
        .cursor_pointer();
    match state {
        CheckSquareState::Off => square,
        CheckSquareState::Dash => square.child(mark_svg(icons::MINUS_SVG, accent)),
        CheckSquareState::Dot => square.child(
            div()
                .w(ui_size(4.))
                .h(ui_size(4.))
                .rounded_full()
                .bg(rgb(accent)),
        ),
        CheckSquareState::On => square.child(mark_svg(icons::CHECK_SVG, accent)),
    }
}

// An SVG mark centers geometrically; an icon-font glyph sits visibly off-center in a 14px box.
fn mark_svg(path: &'static str, color: u32) -> gpui::Svg {
    svg()
        .path(path)
        .w(ui_size(8.))
        .h(ui_size(8.))
        .text_color(rgb(color))
}

pub(crate) fn checkbox_glyph(checked: bool, theme: &Theme) -> Div {
    let glyph = div()
        .flex_none()
        .w(ui_size(14.))
        .h(ui_size(14.))
        .rounded(ui_size(3.))
        .border_1()
        .border_color(rgb(theme.border))
        .flex()
        .items_center()
        .justify_center();
    if checked {
        glyph.bg(rgb(theme.toggle_active_bg)).child(icons::icon(
            icons::glyph::CHECK,
            10.,
            theme.toggle_active_fg,
        ))
    } else {
        glyph
    }
}

pub(crate) fn checkbox_row(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    checked: bool,
    theme: &Theme,
) -> Stateful<Div> {
    let id = id.into();
    let debug_id = id.clone();
    div()
        .id(id)
        .debug_selector(move || debug_id.to_string())
        .flex()
        .flex_row()
        .items_center()
        .gap(ui_size(6.))
        .cursor_pointer()
        .child(checkbox_glyph(checked, theme))
        .child(
            div()
                .text_size(ui_size(12.))
                .text_color(rgb(theme.fg))
                .child(label.into()),
        )
}
