use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, div, px, rgb,
};

use crate::app::theme::{Theme, ui_font_size};
use crate::repo::window::{RepoWindow, Toast};
use crate::ui::icons::glyph;
use crate::ui::overlay::{overlay_card, overlay_header, overlay_layer};
use crate::ui::primitives::{button, small_button};

pub(crate) fn error_overlay(
    message: gpui::SharedString,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    overlay_layer()
        .child(
            overlay_card(t, 460.)
                .child(overlay_header(
                    glyph::WARNING,
                    t.error_fg,
                    jayjay_i18n::repo::operation_failed(),
                    "",
                    t,
                ))
                .child(
                    div()
                        .text_size(ui_font_size(12.))
                        .line_height(ui_font_size(18.))
                        .text_color(rgb(t.fg_dim))
                        .child(message),
                )
                .child(
                    div().flex().flex_row().justify_end().child(
                        button("error-ok", jayjay_i18n::repo::ok(), t, true)
                            .debug_selector(|| "error-ok".to_owned())
                            .on_click(cx.listener(|view, _, _, cx| {
                                view.vm.update(cx, |vm, cx| {
                                    vm.clear_error();
                                    cx.notify();
                                });
                            })),
                    ),
                ),
        )
        .into_any_element()
}

pub(crate) fn toast_overlay(toast: Toast, t: &Theme, cx: &mut Context<RepoWindow>) -> AnyElement {
    let mut card = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(12.))
        .max_w(px(520.))
        .px(px(18.))
        .py(px(10.))
        .rounded(px(14.))
        .border_1()
        .border_color(rgb(t.border))
        .bg(rgb(t.header_bg))
        .text_size(ui_font_size(13.))
        .line_height(ui_font_size(18.))
        .font_weight(gpui::FontWeight::MEDIUM)
        .text_color(rgb(t.fg))
        .child(toast.message);
    if let Some(operation_id) = toast.undo_operation_id {
        card = card.child(
            small_button("toast-undo", jayjay_i18n::repo::undo(), t)
                .debug_selector(|| "toast-undo".to_owned())
                .on_click(cx.listener(move |view, _, _, cx| {
                    view.undo_toast_operation(operation_id.clone(), cx);
                })),
        );
    }
    div()
        .absolute()
        .top_0()
        .left_0()
        .right_0()
        .bottom_0()
        .flex()
        .items_center()
        .justify_center()
        .px(px(24.))
        .child(card)
        .into_any_element()
}
