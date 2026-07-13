use gpui::{
    AnyElement, ClickEvent, Context, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px, rgb,
};

use super::OperationLogView;
use crate::app::theme::Theme;
use crate::ui::icons::{self, glyph};
use crate::ui::primitives::button;

pub(super) fn header(t: &Theme) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.))
        .px(px(16.))
        .py(px(10.))
        .bg(rgb(t.header_bg))
        .border_b_1()
        .border_color(rgb(t.border))
        .child(icons::icon(glyph::ARROW_CLOCKWISE, 14., t.fg_dim))
        .child(
            div()
                .text_size(px(13.))
                .text_color(rgb(t.fg))
                .child("Operation Log"),
        )
        .into_any_element()
}

pub(super) fn footer(
    can_restore: bool,
    restoring: bool,
    t: &Theme,
    cx: &mut Context<OperationLogView>,
) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_end()
        .gap(px(8.))
        .px(px(16.))
        .py(px(10.))
        .border_t_1()
        .border_color(rgb(t.border))
        .child(
            button("operation-log-close", "Close", t, false)
                .debug_selector(|| "operation-log-close".to_owned())
                .on_click(cx.listener(|_, _: &ClickEvent, window, _| {
                    window.remove_window();
                })),
        )
        .child(restore_button(can_restore && !restoring, restoring, t, cx))
        .into_any_element()
}

fn restore_button(
    enabled: bool,
    restoring: bool,
    t: &Theme,
    cx: &mut Context<OperationLogView>,
) -> AnyElement {
    let (bg, fg) = if enabled {
        (t.toggle_active_bg, t.toggle_active_fg)
    } else {
        (t.toggle_inactive_bg, t.fg_faint)
    };
    let label = if restoring { "Restoring..." } else { "Restore" };
    let base = div()
        .id("operation-log-restore")
        .debug_selector(|| "operation-log-restore".to_owned())
        .flex()
        .items_center()
        .justify_center()
        .px(px(10.))
        .h(px(28.))
        .rounded_md()
        .bg(rgb(bg))
        .text_color(rgb(fg))
        .text_size(px(12.))
        .child(label);
    if enabled {
        base.cursor_pointer()
            .hover(|s| s.bg(rgb(t.row_alt_bg)))
            .on_click(cx.listener(move |view, _: &ClickEvent, _, cx| {
                view.restore_selected(cx);
            }))
            .into_any_element()
    } else {
        base.opacity(0.55).into_any_element()
    }
}

pub(super) fn placeholder(text: &'static str, t: &Theme) -> AnyElement {
    div()
        .flex()
        .flex_1()
        .items_center()
        .justify_center()
        .text_color(rgb(t.fg_dim))
        .child(text)
        .into_any_element()
}

pub(super) fn placeholder_err(text: &SharedString, t: &Theme) -> AnyElement {
    div()
        .flex()
        .flex_1()
        .items_center()
        .justify_center()
        .text_color(rgb(t.error_fg))
        .child(text.clone())
        .into_any_element()
}
