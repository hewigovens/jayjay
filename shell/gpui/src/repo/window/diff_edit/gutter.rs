use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, Pixels, SharedString,
    StatefulInteractiveElement, Styled, div, px, rgb, svg,
};
use jayjay_core::diff::DiffLine;

use crate::app::fonts;
use crate::app::theme::Theme;
use crate::diff::line::{ROW_HEIGHT, content_row, gutter_cell, line_bg_color};
use crate::repo::window::RepoWindow;

const CHECKBOX_WIDTH: f32 = 18.;

pub(super) struct DiffEditLineRowState<'a> {
    pub(super) path: &'a str,
    pub(super) line: &'a DiffLine,
    pub(super) display_line: u32,
    pub(super) editable: bool,
    pub(super) checked: bool,
    pub(super) advance: Pixels,
}

pub(super) fn diff_edit_line_row(
    state: DiffEditLineRowState<'_>,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let DiffEditLineRowState {
        path,
        line,
        display_line,
        editable,
        checked,
        advance,
    } = state;
    let bg = line_bg_color(line.style, line.conflict_kind, t);
    let checkbox = if editable && line.is_changed() {
        checkbox_cell(path, display_line, checked, bg, t, cx)
    } else {
        div()
            .flex_none()
            .w(px(CHECKBOX_WIDTH))
            .h(px(ROW_HEIGHT))
            .bg(rgb(bg))
            .into_any_element()
    };
    let old_no = line.old_line_no.map(|n| n.to_string()).unwrap_or_default();
    let new_no = line.new_line_no.map(|n| n.to_string()).unwrap_or_default();
    div()
        .flex()
        .w_full()
        .h(px(ROW_HEIGHT))
        .px(px(18.))
        .font_family(fonts::mono())
        .text_size(px(12.))
        .line_height(px(ROW_HEIGHT))
        .child(
            div()
                .flex()
                .flex_none()
                .border_r_1()
                .border_color(rgb(t.border))
                .child(checkbox)
                .child(gutter_cell(old_no, t, bg))
                .child(gutter_cell(new_no, t, bg)),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .pl(px(4.))
                .child(content_row(line, t, None, None, advance)),
        )
        .into_any_element()
}

fn checkbox_cell(
    path: &str,
    display_line: u32,
    checked: bool,
    bg: u32,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let toggle_path = path.to_owned();
    div()
        .id(SharedString::from(format!(
            "diff-edit-line-{path}-{display_line}"
        )))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .w(px(CHECKBOX_WIDTH))
        .h(px(ROW_HEIGHT))
        .bg(rgb(bg))
        .text_color(rgb(t.fg_faint))
        .cursor_pointer()
        .on_click(cx.listener(move |view, _, _, cx| {
            view.toggle_diff_edit_display_line(&toggle_path, display_line, cx);
        }))
        .child(if checked {
            svg()
                .path(crate::ui::icons::CHECK_SVG)
                .w(px(10.))
                .h(px(10.))
                .text_color(rgb(t.selected_accent))
                .into_any_element()
        } else {
            "□".into_any_element()
        })
        .into_any_element()
}
