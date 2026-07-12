use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, Pixels, SharedString,
    StatefulInteractiveElement, Styled, div, px, rgb,
};
use jayjay_core::diff::DiffLine;

use super::RepoWindow;
use crate::app::fonts;
use crate::app::theme::Theme;
use crate::diff::line::{
    NOTE_DOT_WIDTH, ROW_HEIGHT, content_row, interactive_gutter_row, line_bg_color, note_dot_cell,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn diff_edit_line_row(
    path: &str,
    line: &DiffLine,
    display_line: u32,
    editable: bool,
    checked: bool,
    t: &Theme,
    advance: Pixels,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let affordance = if editable && line.is_changed() {
        changed_line_affordance(path, display_line, checked, line.style, t, cx)
    } else {
        note_dot_cell(None, t, line_bg_color(line.style, line.conflict_kind, t)).into_any_element()
    };
    div()
        .flex()
        .h(px(ROW_HEIGHT))
        .px(px(18.))
        .child(interactive_gutter_row(line, t, false, affordance))
        .child(content_row(line, t, None, None, advance))
        .into_any_element()
}

fn changed_line_affordance(
    path: &str,
    display_line: u32,
    checked: bool,
    style: jayjay_core::diff::DiffSpanStyle,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let checkbox_path = path.to_owned();
    let stripe_path = path.to_owned();
    let stripe_color = match style {
        jayjay_core::diff::DiffSpanStyle::Added => t.file_added_color,
        jayjay_core::diff::DiffSpanStyle::Removed => t.file_removed_color,
        _ => t.file_modified_color,
    };
    div()
        .flex()
        .w(px(NOTE_DOT_WIDTH))
        .h(px(ROW_HEIGHT))
        .child(
            div()
                .id(SharedString::from(format!(
                    "diff-edit-line-{checkbox_path}-{display_line}"
                )))
                .flex()
                .flex_1()
                .items_center()
                .justify_center()
                .h_full()
                .font_family(fonts::mono())
                .text_size(px(8.))
                .text_color(rgb(if checked {
                    t.selected_accent
                } else {
                    t.fg_faint
                }))
                .cursor_pointer()
                .on_click(cx.listener(move |view, _, _, cx| {
                    view.toggle_diff_edit_display_line(&checkbox_path, display_line, cx);
                }))
                .child(if checked { "x" } else { "□" }),
        )
        .child(
            div()
                .id(SharedString::from(format!(
                    "diff-edit-group-stripe-{stripe_path}-{display_line}"
                )))
                .w(px(3.))
                .h_full()
                .bg(rgb(stripe_color))
                .cursor_pointer()
                .on_click(cx.listener(move |view, _, _, cx| {
                    view.select_diff_edit_display_group(&stripe_path, display_line, cx);
                })),
        )
        .into_any_element()
}
