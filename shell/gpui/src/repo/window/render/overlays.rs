use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    ParentElement, StatefulInteractiveElement, Styled, div, px, rgb, rgba,
};

use crate::app::theme::Theme;
use crate::repo::window::{RepoWindow, TextModalState};
use crate::ui::icons::{glyph, icon};
use crate::ui::primitives::{button, icon_label};

pub(super) fn text_modal_overlay(
    modal: &TextModalState,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    div()
        .absolute()
        .top_0()
        .left_0()
        .right_0()
        .bottom_0()
        .flex()
        .items_center()
        .justify_center()
        .bg(rgba(0x00000033))
        .on_mouse_down(MouseButton::Left, |_: &MouseDownEvent, _, _| {})
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(12.))
                .w(px(520.))
                .max_w_full()
                .px(px(18.))
                .py(px(16.))
                .rounded_lg()
                .border_1()
                .border_color(rgb(t.border))
                .bg(rgb(t.header_bg))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .child(
                            icon_label(glyph::PENCIL_CIRCLE, modal.title.clone(), 16., t.fg_dim)
                                .text_size(px(14.))
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(rgb(t.fg)),
                        )
                        .child(div().flex_1())
                        .child(
                            div()
                                .font_family(crate::app::fonts::mono())
                                .text_size(px(11.))
                                .text_color(rgb(t.fg_dim))
                                .child(modal.subtitle.clone()),
                        ),
                )
                .child(modal.input.clone())
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .justify_end()
                        .gap(px(8.))
                        .child(button("text-modal-cancel", "Cancel", t, false).on_click(
                            cx.listener(|view, _, _, cx| {
                                view.close_text_modal(cx);
                            }),
                        ))
                        .child(
                            button("text-modal-primary", modal.primary_label.clone(), t, true)
                                .on_click(cx.listener(|view, _, _, cx| {
                                    view.submit_text_modal(cx);
                                })),
                        ),
                ),
        )
        .into_any_element()
}

pub(super) fn error_overlay(
    message: gpui::SharedString,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    div()
        .absolute()
        .top_0()
        .left_0()
        .right_0()
        .bottom_0()
        .flex()
        .items_center()
        .justify_center()
        .bg(rgba(0x00000033))
        .on_mouse_down(MouseButton::Left, |_: &MouseDownEvent, _, _| {})
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(12.))
                .w(px(460.))
                .max_w_full()
                .px(px(20.))
                .py(px(18.))
                .rounded_lg()
                .border_1()
                .border_color(rgb(t.border))
                .bg(rgb(t.header_bg))
                .child(
                    div().flex().flex_row().items_center().child(
                        icon_label(glyph::WARNING, "Operation failed", 18., t.error_fg)
                            .text_size(px(14.))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(rgb(t.fg)),
                    ),
                )
                .child(
                    div()
                        .text_size(px(12.))
                        .line_height(px(18.))
                        .text_color(rgb(t.fg_dim))
                        .child(message),
                )
                .child(
                    div().flex().flex_row().justify_end().child(
                        div()
                            .id("error-ok")
                            .px(px(12.))
                            .py(px(5.))
                            .rounded_sm()
                            .bg(rgb(t.toggle_active_bg))
                            .text_color(rgb(t.toggle_active_fg))
                            .text_size(px(12.))
                            .cursor_pointer()
                            .on_click(cx.listener(|view, _, _, cx| {
                                view.vm.update(cx, |vm, cx| {
                                    vm.clear_error();
                                    cx.notify();
                                });
                            }))
                            .child("OK"),
                    ),
                ),
        )
        .into_any_element()
}

pub(super) fn toast_overlay(message: gpui::SharedString) -> AnyElement {
    div()
        .absolute()
        .top_0()
        .left_0()
        .right_0()
        .bottom_0()
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap(px(10.))
                .px(px(32.))
                .py(px(24.))
                .min_w(px(220.))
                .rounded_lg()
                .bg(rgb(0x1c1c1e))
                .text_color(rgb(0xf2f2f7))
                .child(icon(glyph::INFO, 40., 0xf2f2f7))
                .child(
                    div()
                        .text_size(px(14.))
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .child(message),
                ),
        )
        .into_any_element()
}
