use gpui::{AnyElement, InteractiveElement, IntoElement, ParentElement, SharedString, Styled};
use jayjay_core::HunkType;

use crate::app::theme::Theme;
use crate::diff::media_diff::{format_size, media_diff_layout, media_frame, media_pane};
use jayjay_core::diff::DiffSide;

#[derive(Clone, Copy)]
pub(crate) struct SvgDiffContent<'a> {
    pub(crate) old: Option<&'a str>,
    pub(crate) new: Option<&'a str>,
}

pub(crate) fn svg_diff_view(
    content: SvgDiffContent<'_>,
    hunk_type: HunkType,
    t: &Theme,
) -> AnyElement {
    media_diff_layout(
        hunk_type,
        t,
        |side, label, label_bg, label_fg, show_label, t| {
            let content = match side {
                DiffSide::Old => content.old,
                DiffSide::New => content.new,
            };
            pane(content, label, label_bg, label_fg, show_label, t)
        },
    )
}

fn pane(
    content: Option<&str>,
    label: &'static str,
    label_bg: u32,
    label_fg: u32,
    show_label: bool,
    t: &Theme,
) -> AnyElement {
    let meta = metadata_line(content, t);
    let viewer = svg_viewer(content, label, t);
    media_pane(label, label_bg, label_fg, show_label, viewer, Some(meta))
}

fn svg_viewer(content: Option<&str>, label: &'static str, t: &Theme) -> AnyElement {
    let frame = media_frame(t)
        .id(label)
        .debug_selector(|| "svg-preview-pane".to_owned());
    match content {
        Some(content) => frame
            .child(super::svg_preview::svg_preview(content, t))
            .into_any_element(),
        None => frame
            .text_color(gpui::rgb(t.fg_dim))
            .child(SharedString::from("—"))
            .into_any_element(),
    }
}

fn metadata_line(content: Option<&str>, t: &Theme) -> AnyElement {
    let label = content
        .map(|content| format_size(content.len() as u64))
        .unwrap_or_else(|| " ".to_owned());
    crate::diff::media_diff::metadata_line(SharedString::from(label), t)
}
