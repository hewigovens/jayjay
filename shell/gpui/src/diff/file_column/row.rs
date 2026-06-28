use gpui::{
    AnyElement, App, ClickEvent, Div, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, Window, div, px, rgb,
};
use jayjay_core::DiffHunk;

use crate::app::fonts;
use crate::app::theme::Theme;
use crate::diff::file_status;
use crate::ui::icons::{self, glyph};

pub(super) fn row_bg(is_selected: bool, _ix: usize, t: &Theme) -> u32 {
    if is_selected {
        t.selected_bg
    } else {
        t.sidebar_bg
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
    let border = if reviewed {
        t.file_added_color
    } else {
        t.fg_faint
    };
    let mut checkbox = div()
        .id(id)
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .w(px(14.))
        .h(px(14.))
        .rounded_full()
        .border_1()
        .border_color(rgb(border))
        .cursor_pointer()
        .on_click(move |ev, w, cx| {
            cx.stop_propagation();
            on_click(ev, w, cx);
        });
    if reviewed {
        checkbox =
            checkbox
                .bg(rgb(t.file_added_color))
                .child(icons::icon(glyph::CHECK, 9., 0xffffff));
    }
    checkbox.into_any_element()
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

pub(super) fn status_dot(hunk: &DiffHunk, t: &Theme) -> impl IntoElement {
    div()
        .flex_none()
        .w(px(6.))
        .h(px(6.))
        .rounded_full()
        .bg(rgb(file_status::color(hunk, t)))
}
