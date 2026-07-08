use gpui::{AnyElement, Div, IntoElement, ParentElement, SharedString, Styled, div, px, rgb};
use jayjay_core::HunkType;

use crate::app::fonts;
use crate::app::theme::Theme;
use crate::ui::primitives::capsule;

#[derive(Clone, Copy)]
pub(crate) enum MediaSide {
    Old,
    New,
}

pub(crate) fn media_diff_layout<F>(hunk_type: HunkType, t: &Theme, pane: F) -> AnyElement
where
    F: Fn(MediaSide, &'static str, u32, u32, &Theme) -> AnyElement,
{
    match hunk_type {
        HunkType::Added => single_pane_layout(
            pane(MediaSide::New, "Added", t.tag_added_bg, t.tag_added_fg, t),
            t,
        ),
        HunkType::Removed => single_pane_layout(
            pane(
                MediaSide::Old,
                "Removed",
                t.tag_removed_bg,
                t.tag_removed_fg,
                t,
            ),
            t,
        ),
        HunkType::Renamed => single_pane_layout(
            pane(
                MediaSide::New,
                "Renamed",
                t.tag_renamed_bg,
                t.tag_renamed_fg,
                t,
            ),
            t,
        ),
        HunkType::Modified => div()
            .flex()
            .flex_row()
            .flex_1()
            .min_h_0()
            .gap(px(12.))
            .px(px(16.))
            .py(px(16.))
            .bg(rgb(t.detail_bg))
            .child(pane(
                MediaSide::Old,
                "Before",
                t.tag_removed_bg,
                t.tag_removed_fg,
                t,
            ))
            .child(pane(
                MediaSide::New,
                "After",
                t.tag_added_bg,
                t.tag_added_fg,
                t,
            ))
            .into_any_element(),
    }
}

fn single_pane_layout(pane: AnyElement, t: &Theme) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .flex_1()
        .min_h_0()
        .px(px(16.))
        .py(px(16.))
        .bg(rgb(t.detail_bg))
        .child(pane)
        .into_any_element()
}

pub(crate) fn media_pane(
    label: &'static str,
    label_bg: u32,
    label_fg: u32,
    viewer: AnyElement,
    metadata: AnyElement,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .flex_1()
        .min_w_0()
        .min_h_0()
        .items_center()
        .gap(px(8.))
        .child(capsule(label, label_bg, label_fg, 11.))
        .child(viewer)
        .child(metadata)
        .into_any_element()
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
        .font_family(fonts::mono())
        .text_size(px(10.))
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
