use gpui::{
    Anchor, AnyElement, BoxShadow, Context, InteractiveElement, IntoElement, MouseButton,
    MouseDownEvent, ParentElement, Pixels, Point, SharedString, Styled, anchored, deferred, div,
    hsla, px, rgb,
};

use super::SettingsView;
use crate::app::config::{current as current_cfg, update as update_cfg};
use crate::app::theme::Theme;
use crate::app::tools::{EDITOR_OPTIONS, TERMINAL_OPTIONS};
use crate::ui::icons::{self, glyph};

#[derive(Clone)]
pub(super) struct OpenDropdown {
    pub field_id: SharedString,
    pub anchor: Point<Pixels>,
}

pub(super) fn dropdown_overlay(
    state: OpenDropdown,
    t: &Theme,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
    let options: &[(&str, &str)] = match state.field_id.as_ref() {
        "editor" => EDITOR_OPTIONS,
        "terminal" => TERMINAL_OPTIONS,
        _ => return div().into_any_element(),
    };
    let cfg = current_cfg(cx);
    let current = match state.field_id.as_ref() {
        "editor" => cfg.tools.external_editor.clone(),
        "terminal" => cfg.tools.terminal.clone(),
        _ => String::new(),
    };
    let field_id = state.field_id.clone();

    let backdrop = div()
        .id(SharedString::from("dropdown-backdrop"))
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(|view, _: &MouseDownEvent, _w, cx| view.close_dropdown(cx)),
        );

    let mut panel = div()
        .flex()
        .flex_col()
        .gap(px(1.))
        .p(px(8.))
        .bg(rgb(t.detail_bg))
        .border_1()
        .border_color(rgb(t.border))
        .rounded(px(14.))
        .shadow(popup_shadow(t));

    for (id, label) in options {
        panel = panel.child(dropdown_row(&field_id, id, label, current == *id, t, cx));
    }

    let menu = anchored()
        .anchor(Anchor::TopLeft)
        .position(state.anchor)
        .snap_to_window_with_margin(px(6.))
        .child(panel);

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

fn dropdown_row(
    field_id: &SharedString,
    id: &'static str,
    label: &'static str,
    is_selected: bool,
    t: &Theme,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
    let field_for_click = field_id.clone();
    let row_bg = if is_selected {
        t.selected_accent
    } else {
        t.detail_bg
    };
    let hover_bg = if is_selected {
        t.selected_accent
    } else {
        t.selected_bg
    };
    let text_color = if is_selected { 0xffffff } else { t.fg };
    div()
        .id(SharedString::from(format!("dd-{field_id}-{id}")))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.))
        .h(px(34.))
        .px(px(10.))
        .rounded_lg()
        .bg(rgb(row_bg))
        .text_size(px(14.))
        .text_color(rgb(text_color))
        .cursor_pointer()
        .hover(move |s| s.bg(rgb(hover_bg)))
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |view, _: &MouseDownEvent, _w, cx| {
                cx.stop_propagation();
                let value = id.to_owned();
                let field = field_for_click.clone();
                update_cfg(cx, move |c| match field.as_ref() {
                    "editor" => c.tools.external_editor = value.clone(),
                    "terminal" => c.tools.terminal = value.clone(),
                    _ => {}
                });
                view.close_dropdown(cx);
            }),
        )
        .child(selection_checkmark(is_selected))
        .child(
            div()
                .flex_none()
                .truncate()
                .child(SharedString::from(label)),
        )
        .into_any_element()
}

fn selection_checkmark(is_selected: bool) -> AnyElement {
    let checkmark = if is_selected {
        icons::icon(glyph::CHECK, 15., 0xffffff).into_any_element()
    } else {
        div().into_any_element()
    };
    div()
        .flex_none()
        .w(px(20.))
        .flex()
        .items_center()
        .justify_center()
        .child(checkmark)
        .into_any_element()
}

fn popup_shadow(t: &Theme) -> Vec<BoxShadow> {
    let (wide, tight) = if t.is_dark {
        (0.32, 0.26)
    } else {
        (0.14, 0.08)
    };
    vec![
        BoxShadow::new(px(0.), px(10.), hsla(0., 0., 0., wide)).blur_radius(px(28.)),
        BoxShadow::new(px(0.), px(1.), hsla(0., 0., 0., tight)),
    ]
}
