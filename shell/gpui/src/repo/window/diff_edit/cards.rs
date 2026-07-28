use std::sync::Arc;

use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, Pixels, Styled, div, px,
    rgb, rgba, uniform_list,
};

use super::gutter::{DiffEditLineRowState, diff_edit_line_row};
use super::header::{header_bg, header_row};
use super::rows::{DiffEditRow, DiffEditRowModel};
use crate::app::fonts;
use crate::app::theme::Theme;
use crate::diff::bounds_capture;
use crate::diff::line::ROW_HEIGHT;
use crate::repo::window::RepoWindow;
use crate::ui::icons::{self, glyph};
use crate::ui::scrollbar::vertical_uniform_scrollbar;

pub(super) fn diff_edit_body(
    view: &mut RepoWindow,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    view.ensure_diff_edit_files(cx);
    let model = view.diff_edit_row_model(cx);
    let count = model.rows.len();
    let theme = Arc::new(t.clone());
    let advance = fonts::mono_advance(cx, px(12.));
    let scroll = view.diff_edit.scroll.clone();
    let bounds = view.diff_edit.bounds.clone();
    let list = uniform_list(
        "diff-edit-rows",
        count,
        cx.processor(move |view, range: std::ops::Range<usize>, _window, cx| {
            range
                .map(|ix| row_at(view, &model, ix, &theme, advance, cx))
                .collect()
        }),
    )
    .track_scroll(&scroll);
    div()
        .id("diff-edit-body")
        .relative()
        .flex_1()
        .min_h_0()
        .child(bounds_capture(bounds.clone()))
        .child(list.h_full())
        .child(vertical_uniform_scrollbar(
            scroll,
            bounds,
            px(count as f32 * ROW_HEIGHT),
            t,
            cx,
        ))
        .into_any_element()
}

fn row_at(
    view: &mut RepoWindow,
    model: &Arc<DiffEditRowModel>,
    ix: usize,
    t: &Theme,
    advance: Pixels,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    match &model.rows[ix] {
        DiffEditRow::Notice => notice_row(t),
        DiffEditRow::Gap => div().h(px(ROW_HEIGHT)).into_any_element(),
        DiffEditRow::HeaderPad { top } => header_pad_row(*top, t),
        DiffEditRow::Header(file) => header_row(view, &model.files[*file], t, cx),
        DiffEditRow::Line {
            file,
            line_ix,
            full_line,
        } => {
            let card = &model.files[*file];
            let Some(line) = card
                .diff
                .as_ref()
                .and_then(|diff| diff.lines.get(*line_ix as usize))
            else {
                return div().h(px(ROW_HEIGHT)).into_any_element();
            };
            let checked = full_line.is_some_and(|full| {
                view.diff_edit
                    .selected
                    .get(card.path.as_ref())
                    .is_some_and(|selected| selected.contains(&full))
            });
            diff_edit_line_row(
                DiffEditLineRowState {
                    path: &card.path,
                    line,
                    display_line: *line_ix + 1,
                    editable: full_line.is_some(),
                    checked,
                    advance,
                },
                t,
                cx,
            )
        }
        DiffEditRow::Placeholder { loading, .. } => placeholder_row(
            if *loading {
                "Loading file diff…"
            } else {
                "No textual preview available for this file."
            },
            t,
        ),
    }
}

fn notice_row(t: &Theme) -> AnyElement {
    div()
        .flex()
        .w_full()
        .items_center()
        .gap(px(8.))
        .h(px(ROW_HEIGHT))
        .px(px(18.))
        .text_size(px(11.))
        .text_color(rgb(t.fg_dim))
        .child(icons::icon(glyph::INFO, 11., t.fg_dim))
        .child("Projected, renamed, and non-text files can be previewed here but are not editable yet.")
        .into_any_element()
}

fn header_pad_row(top: bool, t: &Theme) -> AnyElement {
    let mut pad = div().size_full().bg(rgba(header_bg(t)));
    if top {
        pad = pad.rounded_t_lg();
    } else {
        pad = pad.rounded_b_lg();
    }
    div()
        .w_full()
        .h(px(ROW_HEIGHT))
        .px(px(18.))
        .child(pad)
        .into_any_element()
}

fn placeholder_row(text: &'static str, t: &Theme) -> AnyElement {
    div()
        .flex()
        .w_full()
        .items_center()
        .h(px(ROW_HEIGHT))
        .px(px(36.))
        .text_size(px(11.))
        .text_color(rgb(t.fg_dim))
        .child(text)
        .into_any_element()
}
