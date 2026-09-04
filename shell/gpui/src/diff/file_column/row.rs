use gpui::{
    AnyElement, App, ClickEvent, Div, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, Window, div, px, rgb, rgba,
};
use jayjay_core::DiffHunk;
use jayjay_review::ReviewFileRollup;

use crate::app::fonts;
use crate::app::theme::{Theme, ui_font_size, with_alpha};
use crate::diff::file_status;
use crate::ui::primitives::{CheckCircleState, check_circle, text_tooltip};

pub(super) struct FileRowState<'a> {
    pub(super) hunk: &'a DiffHunk,
    pub(super) is_selected: bool,
    pub(super) review_rollup: ReviewFileRollup,
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

pub(super) fn file_name_opacity(show_review: bool, rollup: ReviewFileRollup) -> f32 {
    if show_review && rollup == ReviewFileRollup::Reviewed {
        0.5
    } else {
        1.0
    }
}

pub(super) fn review_checkbox<FRev>(
    id: (&'static str, usize),
    rollup: ReviewFileRollup,
    t: &Theme,
    on_click: FRev,
) -> AnyElement
where
    FRev: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
{
    let selector = format!("{}-{}", id.0, id.1);
    // Mirrors the SwiftUI chrome: outlined check for partial, half-filled green for changed since review.
    let (state, accent, label) = match rollup {
        ReviewFileRollup::Unreviewed => (CheckCircleState::Off, t.file_added_color, "Unreviewed"),
        ReviewFileRollup::Partial => (
            CheckCircleState::CheckOutline,
            t.fg_dim,
            "Partially reviewed",
        ),
        ReviewFileRollup::Reviewed => (CheckCircleState::On, t.file_added_color, "Reviewed"),
        ReviewFileRollup::ChangedSinceReview => (
            CheckCircleState::HalfFilled,
            t.file_added_color,
            "Changed since review",
        ),
    };
    check_circle(id, state, accent, t)
        .debug_selector(move || selector.clone())
        .tooltip(text_tooltip(label))
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
                .text_size(ui_font_size(12.))
                .line_height(px(t.scaled_font_size(16.)))
                .text_color(rgb(t.fg))
                .opacity(name_opacity)
                .child(name.into()),
        )
        .child(
            div()
                .font_family(fonts::mono())
                .text_size(ui_font_size(10.))
                .line_height(px(t.scaled_font_size(13.)))
                .text_color(rgb(t.fg_faint))
                .truncate()
                .child(path.into()),
        )
}

pub(super) fn file_row_height(t: &Theme) -> f32 {
    46. + t.scaled_font_size(16.) + t.scaled_font_size(13.) - 16. - 13.
}

pub(super) fn file_text_limits(width: f32, t: &Theme) -> (usize, usize) {
    (
        ((width / t.scaled_font_size(7.2)) as usize).max(8),
        ((width / t.scaled_font_size(6.)) as usize).max(10),
    )
}

pub(super) fn file_text_inset(show_review: bool) -> f32 {
    let review_col = if show_review { 22. } else { 0. };
    review_col + 14.
}

pub(super) fn row_separator(inset: f32, t: &Theme) -> AnyElement {
    div()
        .absolute()
        .bottom_0()
        .left(px(inset))
        .right_0()
        .h(px(1.))
        .bg(rgb(t.row_border))
        .into_any_element()
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
        .text_size(ui_font_size(9.))
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_color(rgb(t.file_modified_color))
        .child(SharedString::from(format!("\u{25cf}{count}")))
        .into_any_element()
}
