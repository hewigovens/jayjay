use std::sync::Arc;

use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, Pixels, SharedString,
    StatefulInteractiveElement, Styled, div, px, rgb, rgba, uniform_list,
};

use super::RepoWindow;
use super::diff_edit_gutter::diff_edit_line_row;
use super::diff_edit_rows::{DiffEditCardFile, DiffEditRow, DiffEditRowModel};
use crate::app::fonts;
use crate::app::theme::{Theme, with_alpha};
use crate::diff::bounds_capture;
use crate::diff::file_status;
use crate::diff::line::ROW_HEIGHT;
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
                &card.path,
                line,
                *line_ix + 1,
                full_line.is_some(),
                checked,
                t,
                advance,
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

fn header_bg(t: &Theme) -> u32 {
    with_alpha(t.fg, if t.is_dark { 0x12 } else { 0x0a })
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

fn header_row(
    view: &RepoWindow,
    card: &DiffEditCardFile,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let selected_count = view
        .diff_edit
        .selected
        .get(card.path.as_ref())
        .map(|selected| selected.len())
        .unwrap_or(0);
    let mut row = div()
        .flex()
        .flex_1()
        .items_center()
        .gap(px(8.))
        .h(px(ROW_HEIGHT))
        .px(px(14.))
        .bg(rgba(header_bg(t)));
    if card.supported {
        let path = card.path.to_string();
        let checked = selected_count > 0;
        row = row.child(
            div()
                .id(SharedString::from(format!(
                    "diff-edit-file-checkbox-{}",
                    card.path
                )))
                .font_family(fonts::mono())
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_size(px(11.))
                .text_color(rgb(if checked { t.selected_accent } else { t.fg_dim }))
                .cursor_pointer()
                .on_click(cx.listener(move |view, _, _, cx| view.toggle_diff_edit_file(&path, cx)))
                .child(if checked { "[x]" } else { "[ ]" }),
        );
    }
    let (icon, color) = file_icon(card, t);
    row = row.child(icons::icon(icon, 12., color)).child(
        div()
            .font_family(fonts::mono())
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .text_size(px(12.))
            .child(card.path.to_string()),
    );
    if card.supported && card.changed_total > 0 {
        let badge = if selected_count == card.changed_total {
            "File".to_owned()
        } else if selected_count == 0 {
            "None".to_owned()
        } else {
            format!("{selected_count} / {} lines", card.changed_total)
        };
        row = row.child(
            div()
                .px(px(6.))
                .rounded_full()
                .bg(rgba(with_alpha(t.selected_accent, 0x24)))
                .text_size(px(10.))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .child(badge),
        );
    }
    let row =
        row.child(div().flex_1())
            .child(
                div()
                    .text_size(px(11.))
                    .text_color(rgb(t.fg_dim))
                    .child(if card.supported {
                        "Select files or lines to edit"
                    } else {
                        "Text edits not supported"
                    }),
            );
    div()
        .flex()
        .w_full()
        .h(px(ROW_HEIGHT))
        .px(px(18.))
        .child(row)
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

fn file_icon(card: &DiffEditCardFile, t: &Theme) -> (&'static str, u32) {
    let color = file_status::color_for_hunk_type(card.hunk_type, t);
    match card.hunk_type {
        jayjay_core::HunkType::Added => (glyph::PLUS_CIRCLE, color),
        jayjay_core::HunkType::Removed => (glyph::MINUS_CIRCLE, color),
        jayjay_core::HunkType::Modified => (glyph::PENCIL_CIRCLE, color),
        jayjay_core::HunkType::Renamed => (glyph::ARROW_CIRCLE_RIGHT, color),
    }
}
