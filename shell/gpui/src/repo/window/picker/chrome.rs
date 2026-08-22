use gpui::{
    Anchor, AnyElement, App, Div, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    ParentElement, Pixels, Point, ScrollHandle, SharedString, Stateful, StatefulInteractiveElement,
    Styled, Window, anchored, deferred, div, px, rgb,
};

use super::query::PickerQuery;
use crate::app::theme::Theme;
use crate::ui::icons::{self, glyph};
use crate::ui::input::{LineInput, line_input_content};
use crate::ui::primitives::{button_container, icon_label};

pub(crate) fn overlay(
    backdrop_id: &'static str,
    anchor: Point<Pixels>,
    content: impl IntoElement,
    on_dismiss: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> AnyElement {
    let backdrop = div()
        .id(backdrop_id)
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .on_mouse_down(MouseButton::Left, on_dismiss);
    let menu = anchored()
        .anchor(Anchor::TopLeft)
        .position(anchor)
        .snap_to_window_with_margin(px(6.))
        .child(content);
    deferred(
        div()
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .child(backdrop)
            .child(menu),
    )
    .with_priority(3)
    .into_any_element()
}

pub(crate) fn panel(
    id: &'static str,
    width: f32,
    header: impl IntoElement,
    rows: Vec<AnyElement>,
    scroll: &ScrollHandle,
    t: &Theme,
) -> AnyElement {
    div()
        .debug_selector(move || id.to_owned())
        .flex()
        .flex_col()
        .w(px(width))
        .min_h(px(120.))
        .max_h(px(480.))
        .bg(rgb(t.detail_bg))
        .border_1()
        .border_color(rgb(t.border))
        .rounded_lg()
        .overflow_hidden()
        .occlude()
        .child(header)
        .child(
            div()
                .id("picker-scroll")
                .flex()
                .flex_col()
                .min_h_0()
                .overflow_y_scroll()
                .track_scroll(scroll)
                .py(px(4.))
                .children(rows),
        )
        .into_any_element()
}

pub(crate) fn header(
    filter_id: &'static str,
    query: &PickerQuery,
    buttons: impl IntoIterator<Item = AnyElement>,
    t: &Theme,
) -> Div {
    div()
        .flex()
        .flex_none()
        .items_center()
        .gap(px(8.))
        .h(px(45.))
        .px(px(12.))
        .border_b_1()
        .border_color(rgb(t.border))
        .child(search_box(filter_id, &query.input, t))
        .children(buttons)
}

pub(crate) fn header_button(
    id: &'static str,
    icon: &'static str,
    label: &'static str,
    t: &Theme,
    on_click: impl Fn(&mut Window, &mut App) + 'static,
) -> AnyElement {
    button_container(id, t, false)
        .debug_selector(move || id.to_owned())
        .on_mouse_down(MouseButton::Left, move |_: &MouseDownEvent, window, cx| {
            cx.stop_propagation();
            on_click(window, cx);
        })
        .child(icon_label(icon, label, 12., t.fg_dim))
        .into_any_element()
}

fn search_box(id: &'static str, query: &LineInput, t: &Theme) -> Stateful<Div> {
    div()
        .id(id)
        .debug_selector(move || id.to_owned())
        .flex()
        .items_center()
        .min_w_0()
        .flex_1()
        .gap(px(7.))
        .text_size(px(13.))
        .cursor_text()
        .child(icons::icon(glyph::SEARCH, 13., t.fg_dim))
        .child(line_input_content(
            query,
            "Filter",
            t,
            Some("picker-search-caret"),
        ))
}

pub(super) fn section_header(id: &'static str, label: &'static str, t: &Theme) -> AnyElement {
    div()
        .id(id)
        .debug_selector(move || id.to_owned())
        .px(px(14.))
        .pt(px(8.))
        .pb(px(3.))
        .text_size(px(11.))
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_color(rgb(t.fg_dim))
        .child(label)
        .into_any_element()
}

pub(crate) fn empty(label: impl Into<SharedString>, t: &Theme) -> AnyElement {
    div()
        .w_full()
        .py(px(18.))
        .text_size(px(12.))
        .text_color(rgb(t.fg_dim))
        .text_align(gpui::TextAlign::Center)
        .child(label.into())
        .into_any_element()
}

pub(crate) fn row(id: String, selected: bool, height: f32, t: &Theme) -> Stateful<Div> {
    let background = if selected { t.selected_bg } else { t.detail_bg };
    div()
        .id(SharedString::from(id.clone()))
        .debug_selector(move || id.clone())
        .flex()
        .items_center()
        .w_full()
        .h(px(height))
        .px(px(14.))
        .bg(rgb(background))
        .hover(|style| style.bg(rgb(t.row_alt_bg)))
}
