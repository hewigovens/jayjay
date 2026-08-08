use gpui::{
    AnyElement, App, ClickEvent, Div, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, Window, div, px, rgb, rgba,
};
use jayjay_core::DiffHunk;

use crate::app::fonts;
use crate::app::theme::{Theme, with_alpha};
use crate::diff::file_status;
use crate::ui::primitives::{CheckCircleState, check_circle};

pub(super) struct FileRowState<'a> {
    pub(super) hunk: &'a DiffHunk,
    pub(super) is_selected: bool,
    pub(super) reviewed: bool,
    pub(super) show_review: bool,
    pub(super) note_count: usize,
    pub(super) ix: usize,
    pub(super) theme: &'a Theme,
}

pub(super) struct FileRowHandlers<F, FR, FRev> {
    pub(super) on_click: F,
    pub(super) on_right_click: FR,
    pub(super) on_review_click: FRev,
}

pub(super) fn row_bg(is_selected: bool, t: &Theme) -> u32 {
    if is_selected {
        t.selected_bg
    } else {
        t.detail_bg
    }
}

pub(super) fn file_name_opacity(show_review: bool, reviewed: bool) -> f32 {
    if show_review && reviewed { 0.5 } else { 1.0 }
}

pub(super) fn review_checkbox<FRev>(
    id: (&'static str, usize),
    reviewed: bool,
    t: &Theme,
    on_click: FRev,
) -> AnyElement
where
    FRev: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
{
    let state = if reviewed {
        CheckCircleState::On
    } else {
        CheckCircleState::Off
    };
    check_circle(id, state, t.file_added_color, t)
        .on_click(move |ev, w, cx| {
            cx.stop_propagation();
            on_click(ev, w, cx);
        })
        .into_any_element()
}

pub(super) fn file_text_content(
    name: impl Into<SharedString>,
    path: impl Into<SharedString>,
    name_opacity: f32,
    t: &Theme,
) -> Div {
    div()
        .flex()
        .flex_col()
        .gap(px(2.))
        .flex_1()
        .min_w_0()
        .child(
            div()
                .font_family(fonts::mono())
                .text_size(px(12.))
                .text_color(rgb(t.fg))
                .opacity(name_opacity)
                .child(name.into()),
        )
        .child(
            div()
                .font_family(fonts::mono())
                .text_size(px(10.))
                .text_color(rgb(t.fg_faint))
                .truncate()
                .child(path.into()),
        )
}

fn status_dot(hunk: &DiffHunk, t: &Theme) -> impl IntoElement {
    div()
        .flex_none()
        .w(px(6.))
        .h(px(6.))
        .rounded_full()
        .bg(rgb(file_status::color(hunk, t)))
}

pub(super) fn finish_file_row(
    row: impl ParentElement + IntoElement,
    hunk: &DiffHunk,
    content: impl IntoElement,
    note_count: usize,
    t: &Theme,
) -> AnyElement {
    let mut row = row
        .child(status_dot(hunk, t))
        .child(super::file_name_container(content));
    if note_count > 0 {
        row = row.child(note_badge(note_count, t));
    }
    row.into_any_element()
}

/// Counts only notes with status == Current — callers must pre-filter before passing count.
fn note_badge(count: usize, t: &Theme) -> AnyElement {
    div()
        .flex_none()
        .px(px(5.))
        .py(px(1.))
        .rounded_full()
        .bg(rgba(with_alpha(
            t.file_modified_color,
            if t.is_dark { 0x2a } else { 0x1f },
        )))
        .text_size(px(9.))
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_color(rgb(t.file_modified_color))
        .child(SharedString::from(format!("\u{25cf}{count}")))
        .into_any_element()
}
