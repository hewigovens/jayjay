use gpui::{
    AnyElement, Context, CursorStyle, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    ParentElement, SharedString, StatefulInteractiveElement, Styled, div, px, rgb,
};
use jayjay_core::ChangeInfo;

use crate::app::theme::{FONT_BODY, FONT_META, Theme};
use crate::log::commit_row::first_line;
use crate::log::{DragTarget, LogView};
use crate::ui::icons::{glyph, icon};

pub(super) fn description_block(
    change: &ChangeInfo,
    height: f32,
    t: &Theme,
    cx: &mut Context<LogView>,
) -> AnyElement {
    let title = first_line(&change.description);
    let body = change
        .description
        .lines()
        .skip(1)
        .collect::<Vec<_>>()
        .join("\n");
    let body = body.trim().to_string();

    let header = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.))
        .pb(px(4.))
        .border_b_1()
        .border_color(rgb(t.border))
        .child(
            div()
                .text_size(px(14.))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(rgb(t.fg))
                .child("Description"),
        )
        .child(div().flex_1())
        .child(edit_button(change.is_immutable, t, cx));

    let title_el: AnyElement = if title.is_empty() {
        div()
            .text_size(px(FONT_BODY))
            .text_color(rgb(t.fg_faint))
            .child("(no description)")
            .into_any_element()
    } else {
        div()
            .text_size(px(FONT_BODY))
            .text_color(rgb(t.fg))
            .child(SharedString::from(title))
            .into_any_element()
    };

    let mut body_scroll = gpui::div()
        .id(SharedString::from("description-body"))
        .flex()
        .flex_col()
        .gap(px(4.))
        .h(px(height))
        .overflow_y_scroll()
        .child(title_el);
    if !body.is_empty() {
        body_scroll = body_scroll.child(
            div()
                .text_size(px(FONT_META))
                .text_color(rgb(t.fg_dim))
                .child(SharedString::from(body)),
        );
    }

    div()
        .flex()
        .flex_col()
        .gap(px(6.))
        .child(header)
        .child(body_scroll)
        .child(description_resize_handle(t, cx))
        .into_any_element()
}

fn edit_button(immutable: bool, t: &Theme, cx: &mut Context<LogView>) -> AnyElement {
    if immutable {
        return div().into_any_element();
    }

    div()
        .id(SharedString::from("edit-description"))
        .flex()
        .items_center()
        .justify_center()
        .size(px(22.))
        .rounded_sm()
        .child(icon(glyph::PENCIL_CIRCLE, 13., t.fg_dim))
        .cursor_pointer()
        .hover(|s| s.bg(rgb(t.row_alt_bg)))
        .on_click(cx.listener(|view, _, _, cx| view.edit_selected_description(cx)))
        .into_any_element()
}

fn description_resize_handle(t: &Theme, cx: &mut Context<LogView>) -> AnyElement {
    div()
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .h(px(10.))
        .w_full()
        .cursor(CursorStyle::ResizeUpDown)
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(|view, ev: &MouseDownEvent, _w, cx| {
                view.start_drag(DragTarget::Description, f32::from(ev.position.y), cx);
            }),
        )
        .child(div().w(px(36.)).h(px(3.)).rounded_full().bg(rgb(t.border)))
        .into_any_element()
}
