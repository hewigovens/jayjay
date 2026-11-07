use std::sync::LazyLock;

use gpui::{
    AnyElement, App, Div, FontWeight, InteractiveElement, IntoElement, ParentElement, SharedString,
    Styled, canvas, div, px, rgb,
};
use jayjay_core::diff::DiffLine;
use jayjay_core::{FontSpecimen, SpecimenChange, SpecimenFile};

use crate::app::fonts::{self, CodeText as _};
use crate::app::theme::{FONT_BODY, FONT_ID, FONT_TAG, Theme, ui_font_size};
use crate::diff::file_status;
use crate::diff::line::{content_row, gutter_cell, line_bg_color};
use crate::repo::window::ref_chips;
use crate::repo::window::{
    DagNodeStyle, LinePattern, NodeFlags, id_cell, paint_node, stroke_line_pattern,
};
use crate::ui::primitives::capsule;

static SPECIMEN: LazyLock<FontSpecimen> = LazyLock::new(FontSpecimen::default);

fn specimen_diff_rows<'a>(
    lines: &'a [DiffLine],
    t: &'a Theme,
    cx: &App,
) -> impl Iterator<Item = Div> + 'a {
    let advance = fonts::mono_advance(cx, px(t.code_font_size()));
    lines.iter().map(move |line| {
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
    })
}

pub(super) fn font_preview(t: &Theme, cx: &App) -> AnyElement {
    let specimen = &*SPECIMEN;
    let (directory, file_name) = specimen
        .path
        .split_at(specimen.path.rfind('/').map_or(0, |slash| slash + 1));
    let rows = specimen_diff_rows(&specimen.diff.lines, t, cx);
    div()
        .id("settings-font-preview")
        .debug_selector(|| "settings-font-preview".to_owned())
        .flex()
        .flex_col()
        .w_full()
        .gap(px(10.))
        .p(px(16.))
        .rounded_lg()
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
                .py(px(6.))
                .rounded_md()
                .bg(rgb(t.diff_context_bg))
                .overflow_hidden()
                .children(rows),
        )
        .into_any_element()
}

pub(super) fn theme_preview(pt: &Theme, cx: &App) -> AnyElement {
    let specimen = &*SPECIMEN;
    let count = specimen.changes.len();
    let changes = specimen
        .changes
        .iter()
        .enumerate()
        .map(|(ix, change)| specimen_change_row(change, ix, count, pt));
    let files = specimen
        .files
        .iter()
        .enumerate()
        .map(|(ix, file)| specimen_file_row(file, ix == 0, pt));
    let diff_rows = specimen_diff_rows(&specimen.diff.lines, pt, cx);

    div()
        .id("settings-theme-preview")
        .debug_selector(|| "settings-theme-preview".to_owned())
        .flex()
        .flex_col()
        .w_full()
        .rounded(px(8.))
        .overflow_hidden()
        .border_1()
        .border_color(rgb(pt.border))
        .bg(rgb(pt.detail_bg))
        .text_color(rgb(pt.fg))
        .child(
            div()
                .flex()
                .flex_col()
                .w_full()
                .py(px(4.))
                .bg(rgb(pt.sidebar_bg))
                .children(changes),
        )
        .child(div().h(px(1.)).w_full().bg(rgb(pt.border)))
        .child(div().flex().flex_col().w_full().py(px(4.)).children(files))
        .child(div().h(px(1.)).w_full().bg(rgb(pt.border)))
        .child(
            div()
                .flex()
                .flex_col()
                .w_full()
                .py(px(4.))
                .bg(rgb(pt.diff_context_bg))
                .overflow_hidden()
                .children(diff_rows),
        )
        .into_any_element()
}

fn specimen_change_row(change: &SpecimenChange, ix: usize, count: usize, pt: &Theme) -> Div {
    let mut chips = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(5.))
        .child(id_cell(
            &change.change_id.id,
            change.change_id.short_len,
            pt.change_id_prefix,
            FONT_ID,
            pt,
        ));
    if change.is_working_copy {
        chips = chips.child(capsule("@", pt.tag_wc_bg, pt.tag_wc_fg, FONT_TAG));
    }
    if let Some(bookmark) = &change.bookmark {
        chips = chips.child(ref_chips::bookmark_chip(
            bookmark.clone().into(),
            false,
            true,
            FONT_TAG,
            pt,
        ));
    }
    chips = chips.child(
        div()
            .flex_1()
            .min_w_0()
            .truncate()
            .text_size(ui_font_size(12.))
            .font_weight(if change.is_working_copy {
                FontWeight::MEDIUM
            } else {
                FontWeight::NORMAL
            })
            .text_color(rgb(pt.fg))
            .child(SharedString::from(change.summary.clone())),
    );

    let mut row = div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .h(px(pt.scaled_control_height(26., 12.)))
        .px(px(10.))
        .gap(px(8.));
    if ix == 0 {
        row = row.bg(rgb(pt.selected_bg));
    }
    let node = DagNodeStyle::new(
        NodeFlags {
            is_working_copy: change.is_working_copy,
            is_immutable: change.is_immutable,
            has_bookmark: change.bookmark.is_some(),
            ..NodeFlags::default()
        },
        pt,
    );
    let node = DagNodeStyle {
        radius: DagNodeStyle::radius(node.shape, false),
        ..node
    };
    row.child(specimen_graph_lane(ix > 0, ix + 1 < count, node, pt))
        .child(chips.flex_1().min_w_0())
}

fn specimen_graph_lane(has_top: bool, has_bottom: bool, node: DagNodeStyle, pt: &Theme) -> Div {
    let line = pt.dag_line;
    div().flex_none().w(px(14.)).h_full().child(
        canvas(
            |_, _, _| (),
            move |bounds, _, window, _| {
                let center = bounds.center();
                let r = px(node.radius);
                if has_top {
                    stroke_line_pattern(
                        window,
                        center.x,
                        bounds.top(),
                        center.x,
                        center.y - r,
                        line,
                        LinePattern::Solid,
                    );
                }
                if has_bottom {
                    stroke_line_pattern(
                        window,
                        center.x,
                        center.y + r,
                        center.x,
                        bounds.bottom(),
                        line,
                        LinePattern::Solid,
                    );
                }
                paint_node(window, center.x, center.y, node);
            },
        )
        .size_full(),
    )
}

fn specimen_file_row(file: &SpecimenFile, is_selected: bool, pt: &Theme) -> Div {
    let (directory, file_name) = file
        .path
        .split_at(file.path.rfind('/').map_or(0, |slash| slash + 1));
    let mut row = div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .h(px(pt.scaled_control_height(24., 12.)))
        .px(px(10.))
        .gap(px(8.));
    if is_selected {
        row = row.bg(rgb(pt.selected_bg));
    }
    row.child(file_status::badge(file.hunk_type, 14., pt))
        .child(
            div()
                .flex()
                .flex_row()
                .flex_1()
                .min_w_0()
                .text_size(ui_font_size(12.))
                .child(
                    div()
                        .text_color(rgb(pt.fg_dim))
                        .child(SharedString::from(directory.to_owned())),
                )
                .child(
                    div()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(pt.fg))
                        .truncate()
                        .child(SharedString::from(file_name.to_owned())),
                ),
        )
}
