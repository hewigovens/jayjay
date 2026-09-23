use gpui::{Div, ParentElement, Styled, div, px, rgb};
use jayjay_core::FileDiffStats;

use crate::app::fonts;
use crate::app::theme::ui_font_size;

/// `+N -M`, omitting a zero side; `None` when the file has no line changes.
pub(crate) fn line_stats(
    stats: &FileDiffStats,
    size: f32,
    added: u32,
    removed: u32,
) -> Option<Div> {
    if stats.insertions == 0 && stats.deletions == 0 {
        return None;
    }
    let mut label = div()
        .flex()
        .flex_none()
        .flex_row()
        .items_baseline()
        .gap(px(4.))
        .font_family(fonts::mono())
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_size(ui_font_size(size));
    if stats.insertions > 0 {
        label = label.child(
            div()
                .text_color(rgb(added))
                .child(format!("+{}", stats.insertions)),
        );
    }
    if stats.deletions > 0 {
        label = label.child(
            div()
                .text_color(rgb(removed))
                .child(format!("-{}", stats.deletions)),
        );
    }
    Some(label)
}
