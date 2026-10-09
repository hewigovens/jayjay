use std::sync::LazyLock;

use gpui::{
    AnyElement, App, FontWeight, InteractiveElement, IntoElement, ParentElement, SharedString,
    Styled, div, px, rgb,
};
use jayjay_core::FontSpecimen;

use crate::app::fonts::{self, CodeText as _};
use crate::app::theme::{FONT_BODY, FONT_ID, Theme, ui_font_size};
use crate::diff::line::{content_row, gutter_cell, line_bg_color};

static SPECIMEN: LazyLock<FontSpecimen> = LazyLock::new(FontSpecimen::default);

pub(super) fn font_preview(t: &Theme, cx: &App) -> AnyElement {
    let specimen = &*SPECIMEN;
    let (directory, file_name) = specimen
        .path
        .split_at(specimen.path.rfind('/').map_or(0, |slash| slash + 1));
    let advance = fonts::mono_advance(cx, px(t.code_font_size()));
    let rows = specimen.diff.lines.iter().map(|line| {
        let bg = line_bg_color(line.style, line.conflict_kind, t);
        let number = |n: Option<u32>| n.map(|n| n.to_string()).unwrap_or_default();
        div()
            .flex()
            .flex_row()
            .w_full()
            .code_text(12., t)
            .child(gutter_cell(number(line.old_line_no), t, bg))
            .child(gutter_cell(number(line.new_line_no), t, bg))
            .child(content_row(line, t, None, None, advance).flex_1())
    });
    div()
        .id("settings-font-preview")
        .debug_selector(|| "settings-font-preview".to_owned())
        .flex()
        .flex_col()
        .w_full()
        .gap(px(10.))
        .p(px(16.))
        .rounded_lg()
        .bg(rgb(t.row_alt_bg))
        .child(
            div()
                .text_size(ui_font_size(FONT_BODY))
                .font_weight(FontWeight::SEMIBOLD)
                .child(SharedString::from(specimen.summary.clone())),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .gap(px(8.))
                .items_baseline()
                .child(
                    div()
                        .code_text(FONT_ID, t)
                        .text_color(rgb(t.change_id_prefix))
                        .child(SharedString::from(specimen.change_id.clone())),
                )
                .child(
                    div()
                        .flex()
                        .text_size(ui_font_size(12.))
                        .child(
                            div()
                                .text_color(rgb(t.fg_dim))
                                .child(SharedString::from(directory.to_owned())),
                        )
                        .child(
                            div()
                                .font_weight(FontWeight::MEDIUM)
                                .child(SharedString::from(file_name.to_owned())),
                        ),
                ),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .w_full()
                .overflow_hidden()
                .children(rows),
        )
        .child(
            div()
                .text_size(ui_font_size(11.))
                .text_color(rgb(t.fg_faint))
                .child("Font choices stay the same when you change themes."),
        )
        .into_any_element()
}
