use gpui::{
    Anchor, AnyElement, App, Bounds, Context, Div, InteractiveElement, IntoElement,
    ListSizingBehavior, MouseButton, MouseDownEvent, ParentElement, Pixels, Point, SharedString,
    Stateful, Styled, Window, anchored, canvas, deferred, div, list, point, px, rgb,
};

use super::super::{PanelBoundsSlot, RepoWindow};
use super::query::PickerQuery;
use super::sections::PickerItem;
use crate::app::theme::{Theme, ui_size};
use crate::ui::icons::{self, glyph};
use crate::ui::input::{LineInput, line_input_content};
use crate::ui::primitives::{button_container, icon_label};

/// While its picker is open, the opener's click closes it before the backdrop can dismiss it and the opener reopen it.
pub(crate) fn opener<E: InteractiveElement>(
    element: E,
    is_open: impl Fn(&RepoWindow) -> bool + 'static,
    close: impl Fn(&mut RepoWindow, &mut Context<RepoWindow>) + 'static,
    cx: &mut Context<RepoWindow>,
) -> E {
    element.capture_any_mouse_down(cx.listener(move |view, ev: &MouseDownEvent, _, cx| {
        if ev.button == MouseButton::Left && view.context_menu.is_none() && is_open(view) {
            cx.stop_propagation();
            close(view, cx);
        }
    }))
}

/// Like `bounds_capture`, but pinned to the padding box so the popup aligns with the button's edge rather than its content.
pub(crate) fn opener_bounds(slot: PanelBoundsSlot) -> impl IntoElement {
    canvas(
        move |bounds, window, _| {
            if slot.get() != Some(bounds) {
                slot.set(Some(bounds));
                window.request_animation_frame();
            }
        },
        |_, _, _, _| {},
    )
    .absolute()
    .inset_0()
}

pub(crate) fn below(trigger: Bounds<Pixels>) -> Point<Pixels> {
    point(trigger.origin.x, trigger.bottom() + px(4.))
}

pub(crate) fn overlay(
    backdrop_id: &'static str,
    trigger: Bounds<Pixels>,
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
        .position(below(trigger))
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

pub(crate) fn panel<R: Clone + 'static>(
    id: &'static str,
    width: f32,
    header: impl IntoElement,
    items: Vec<PickerItem<R>>,
    query: &PickerQuery,
    t: &Theme,
    render_row: impl Fn(R, bool, &Theme) -> AnyElement + 'static,
) -> AnyElement {
    if query.list.item_count() != items.len() {
        query.list.reset(items.len());
    }
    let theme = t.clone();
    let body = list(query.list.clone(), move |ix, _, _| match &items[ix] {
        PickerItem::Header { id, title } => section_header(id, title.clone(), &theme),
        PickerItem::Row { row, selected } => render_row(row.clone(), *selected, &theme),
        PickerItem::Empty(label) => empty(label.clone(), &theme),
    })
    .with_sizing_behavior(ListSizingBehavior::Infer)
    .min_h_0()
    .py(ui_size(4.));
    div()
        .debug_selector(move || id.to_owned())
        .flex()
        .flex_col()
        .w(ui_size(width))
        .min_h(ui_size(120.))
        .max_h(ui_size(480.))
        .bg(rgb(t.detail_bg))
        .border_1()
        .border_color(rgb(t.border))
        .rounded_lg()
        .overflow_hidden()
        .occlude()
        .child(header)
        .child(body)
        .into_any_element()
}

pub(crate) fn header(
    filter_id: &'static str,
    query: &PickerQuery,
    ime_layer: impl IntoElement,
    buttons: impl IntoIterator<Item = AnyElement>,
    t: &Theme,
) -> Div {
    div()
        .flex()
        .flex_none()
        .items_center()
        .gap(ui_size(8.))
        .h(ui_size(45.))
        .px(ui_size(12.))
        .border_b_1()
        .border_color(rgb(t.border))
        .child(search_box(filter_id, &query.input, ime_layer, t))
        .children(buttons)
}

pub(crate) fn header_button(
    id: &'static str,
    icon: &'static str,
    label: impl Into<SharedString>,
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

fn search_box(
    id: &'static str,
    query: &LineInput,
    ime_layer: impl IntoElement,
    t: &Theme,
) -> Stateful<Div> {
    div()
        .id(id)
        .relative()
        .debug_selector(move || id.to_owned())
        .flex()
        .items_center()
        .min_w_0()
        .flex_1()
        .gap(ui_size(7.))
        .text_size(ui_size(13.))
        .cursor_text()
        .child(icons::icon(glyph::SEARCH, 13., t.fg_dim))
        .child(ime_layer)
        .child(line_input_content(
            query,
            jayjay_i18n::repo::filter(),
            t,
            Some("picker-search-caret"),
        ))
}

pub(crate) fn section_header(
    id: &'static str,
    label: impl Into<SharedString>,
    t: &Theme,
) -> AnyElement {
    div()
        .id(id)
        .debug_selector(move || id.to_owned())
        .px(ui_size(14.))
        .pt(ui_size(8.))
        .pb(ui_size(3.))
        .text_size(ui_size(11.))
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_color(rgb(t.fg_dim))
        .child(label.into())
        .into_any_element()
}

pub(crate) fn empty(label: impl Into<SharedString>, t: &Theme) -> AnyElement {
    div()
        .w_full()
        .py(ui_size(18.))
        .text_size(ui_size(12.))
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
        .h(ui_size(height))
        .px(ui_size(14.))
        .bg(rgb(background))
        .hover(|style| style.bg(rgb(t.row_alt_bg)))
}
