use gpui::{
    AnyElement, Context, CursorStyle, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    ParentElement, SharedString, StatefulInteractiveElement, Styled, div, px, rgb, rgba,
};
use jayjay_core::ChangeInfo;

use crate::app::fonts;
use crate::app::theme::{FONT_BODY, Theme, ui_font_size};
use crate::repo::window::dag_row::first_line;
use crate::repo::window::{DragTarget, RepoWindow};
use crate::ui::icons::{glyph, icon};
use crate::ui::primitives::text_tooltip;

pub(super) fn description_block(
    change: &ChangeInfo,
    height: f32,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let title = first_line(&change.description);
    let has_description = !change.description.trim().is_empty();
    let body = change
        .description
        .lines()
        .skip(1)
        .collect::<Vec<_>>()
        .join("\n");
    let body = body.trim().to_string();

    let can_show_edit_diff = !change.has_conflict && !change.is_empty && !change.is_immutable;

    let header = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.))
        .child(
            div()
                .text_size(ui_font_size(14.))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(rgb(t.fg))
                .child("Description"),
        )
        .child(edit_button(
            !change.is_immutable && !change.is_working_copy,
            t,
            cx,
        ))
        .child(div().flex_1())
        .child(edit_diff_button(can_show_edit_diff, t, cx));

    let mut block = div()
        .flex()
        .flex_col()
        .gap(px(6.))
        .debug_selector(|| "detail-description".to_owned())
        .child(header);

    if !has_description {
        return block.into_any_element();
    }

    let mut body_scroll = gpui::div()
        .id(SharedString::from("description-body"))
        .debug_selector(|| "description-body".to_owned())
        .flex()
        .flex_col()
        .gap(px(4.))
        .h(px(height))
        .overflow_y_scroll();
    if !title.is_empty() {
        body_scroll = body_scroll.child(
            div()
                .debug_selector(|| "description-title".to_owned())
                .font_family(fonts::mono())
                .text_size(ui_font_size(FONT_BODY))
                .text_color(rgb(t.fg))
                .child(SharedString::from(title)),
        );
    }
    if !body.is_empty() {
        body_scroll = body_scroll.child(
            div()
                .font_family(fonts::mono())
                .text_size(ui_font_size(FONT_BODY))
                .text_color(rgb(t.fg_dim))
                .child(SharedString::from(body)),
        );
    }

    block = block.child(
        div()
            .flex()
            .flex_col()
            .child(body_scroll)
            .child(description_resize_handle(t, cx)),
    );
    block.into_any_element()
}

fn edit_button(can_edit: bool, t: &Theme, cx: &mut Context<RepoWindow>) -> AnyElement {
    if !can_edit {
        return div().into_any_element();
    }

    div()
        .id(SharedString::from("edit-description"))
        .debug_selector(|| "edit-description".to_owned())
        .flex()
        .items_center()
        .justify_center()
        .size(px(22.))
        .rounded_md()
        .child(icon(glyph::PENCIL_CIRCLE, 13., t.fg_dim))
        .cursor_pointer()
        .hover(|s| s.bg(rgb(t.row_alt_bg)))
        .on_click(cx.listener(|view, _, _, cx| view.edit_selected_description(cx)))
        .into_any_element()
}

fn edit_diff_button(visible: bool, t: &Theme, cx: &mut Context<RepoWindow>) -> AnyElement {
    if !visible {
        return div().into_any_element();
    }

    div()
        .id(SharedString::from("edit-diff"))
        .flex()
        .items_center()
        .justify_center()
        .px(px(9.))
        .h(px(t.scaled_control_height(22., 11.)))
        .rounded_md()
        .bg(rgb(t.toggle_inactive_bg))
        .text_color(rgb(t.toggle_inactive_fg))
        .text_size(ui_font_size(11.))
        .debug_selector(|| "edit-diff".to_owned())
        .cursor_pointer()
        .hover(|s| s.bg(rgb(t.row_alt_bg)))
        .tooltip(text_tooltip("Open Diff Edit Mode"))
        .on_click(cx.listener(|view, _, _, cx| view.enter_diff_edit(cx)))
        .child("Edit Diff...")
        .into_any_element()
}

fn description_resize_handle(t: &Theme, cx: &mut Context<RepoWindow>) -> AnyElement {
    div()
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .h(px(10.))
        .w_full()
        .debug_selector(|| "description-resize-handle".to_owned())
        .cursor(CursorStyle::ResizeUpDown)
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(|view, ev: &MouseDownEvent, window, cx| {
                let viewport_width = f32::from(window.viewport_size().width);
                view.start_drag(
                    DragTarget::Description,
                    f32::from(ev.position.y),
                    viewport_width,
                    cx,
                );
            }),
        )
        .child(
            div()
                .w(px(36.))
                .h(px(3.))
                .rounded_full()
                .bg(rgba(((t.fg_dim as u64) << 8) as u32 | 0x59)),
        )
        .into_any_element()
}
