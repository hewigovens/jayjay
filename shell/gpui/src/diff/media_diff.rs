use gpui::{
    AnyElement, Div, InteractiveElement, IntoElement, ParentElement, SharedString, Styled, div, px,
    rgb,
};
use jayjay_core::HunkType;
use jayjay_core::diff::DiffSide;

use crate::app::fonts;
use crate::app::theme::{Theme, ui_font_size};
use crate::diff::line::gutter_column;

pub(crate) fn media_diff_layout<F>(hunk_type: HunkType, t: &Theme, pane: F) -> AnyElement
where
    F: Fn(DiffSide, &'static str, u32, u32, bool, &Theme) -> AnyElement,
{
    match hunk_type {
        HunkType::Added => single_pane_for(
            &pane,
            DiffSide::New,
            "Added",
            t.tag_added_bg,
            t.tag_added_fg,
            t,
        ),
        HunkType::Removed => single_pane_for(
            &pane,
            DiffSide::Old,
            "Removed",
            t.tag_removed_bg,
            t.tag_removed_fg,
            t,
        ),
        HunkType::Renamed => single_pane_for(
            &pane,
            DiffSide::New,
            "Renamed",
            t.tag_renamed_bg,
            t.tag_renamed_fg,
            t,
        ),
        HunkType::Modified => div()
            .flex()
            .flex_row()
            .flex_1()
            .min_w_0()
            .min_h_0()
            .gap(px(12.))
            .px(px(16.))
            .py(px(16.))
            .bg(rgb(t.detail_bg))
            .child(pane(
                DiffSide::Old,
                "Before",
                t.tag_removed_bg,
                t.tag_removed_fg,
                true,
                t,
            ))
            .child(pane(
                DiffSide::New,
                "After",
                t.tag_added_bg,
                t.tag_added_fg,
                true,
                t,
            ))
            .into_any_element(),
    }
}

fn single_pane_for<F>(
    pane: &F,
    side: DiffSide,
    label: &'static str,
    label_bg: u32,
    label_fg: u32,
    t: &Theme,
) -> AnyElement
where
    F: Fn(DiffSide, &'static str, u32, u32, bool, &Theme) -> AnyElement,
{
    single_pane_layout(pane(side, label, label_bg, label_fg, false, t), t)
}

pub(crate) fn single_pane_layout(pane: AnyElement, t: &Theme) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .flex_1()
        .min_w_0()
        .min_h_0()
        .px(px(16.))
        .py(px(16.))
        .bg(rgb(t.detail_bg))
        .child(pane)
        .into_any_element()
}

pub(crate) fn rich_preview_with_gutter(
    content: AnyElement,
    t: &Theme,
    shows_review: bool,
) -> AnyElement {
    diff_body_with_gutter(content, t, "rich-preview-gutter", shows_review)
}

pub(crate) fn diff_body_with_gutter(
    content: AnyElement,
    t: &Theme,
    debug_selector: &'static str,
    shows_review: bool,
) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .flex_1()
        .w_full()
        .h_full()
        .min_w_0()
        .min_h_0()
        .bg(rgb(t.detail_bg))
        .child(gutter_column(t, shows_review).debug_selector(move || debug_selector.to_owned()))
        .child(
            // `.flex()` lets `content`'s own flex_1/min_h_0 bound its height instead of growing to fit.
            div().flex().flex_1().min_w_0().min_h_0().child(content),
        )
        .into_any_element()
}

pub(crate) fn media_pane(
    label: &'static str,
    label_bg: u32,
    label_fg: u32,
    show_label: bool,
    viewer: AnyElement,
    metadata: Option<AnyElement>,
) -> AnyElement {
    let mut pane = div()
        .flex()
        .flex_col()
        .flex_1()
        .min_w_0()
        .min_h_0()
        .items_center()
        .gap(px(8.));
    if show_label {
        pane = pane.child(crate::ui::primitives::capsule(
            label, label_bg, label_fg, 11.,
        ));
    }
    pane = pane.child(viewer);
    if let Some(metadata) = metadata {
        pane = pane.child(metadata);
    }
    pane.into_any_element()
}

pub(crate) fn media_frame(t: &Theme) -> Div {
    div()
        .flex()
        .flex_1()
        .w_full()
        .min_h_0()
        .items_center()
        .justify_center()
        .rounded_md()
        .border_1()
        .border_color(rgb(t.border))
        .bg(rgb(if t.is_dark { 0x14171c } else { 0xeef0f3 }))
}

pub(crate) fn metadata_line(label: impl Into<SharedString>, t: &Theme) -> AnyElement {
    div()
        .debug_selector(|| "rich-preview-metadata".to_owned())
        .font_family(fonts::mono())
        .text_size(ui_font_size(10.))
        .text_color(rgb(t.fg_dim))
        .child(label.into())
        .into_any_element()
}

pub(crate) fn format_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    let b = bytes as f64;
    if b < KB {
        format!("{bytes} B")
    } else if b < MB {
        format!("{:.1} KB", b / KB)
    } else {
        format!("{:.1} MB", b / MB)
    }
}
