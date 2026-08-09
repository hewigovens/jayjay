use std::ops::Range;

use gpui::{Bounds, PaintQuad, Pixels, fill, point, px, rgb, size};
use jayjay_core::diff::DiffSpanStyle;

use super::super::LineLayout;
use crate::app::theme::Theme;
use crate::ui::input::selection_bg;

pub(super) fn line_background_quads(
    lines: &[LineLayout],
    bounds: Bounds<Pixels>,
    line_height: Pixels,
    theme: &Theme,
) -> Vec<PaintQuad> {
    lines
        .iter()
        .filter_map(|line| {
            let color = match line.style {
                DiffSpanStyle::Added => theme.diff_added_bg,
                DiffSpanStyle::Removed => theme.diff_removed_bg,
                _ => return None,
            };
            Some(fill(
                Bounds::new(
                    point(bounds.left(), bounds.top() + line.top),
                    size(bounds.size.width, line_height),
                ),
                rgb(color),
            ))
        })
        .collect()
}

pub(super) fn selection_quads(
    lines: &[LineLayout],
    selected: &Range<usize>,
    bounds: Bounds<Pixels>,
    line_height: Pixels,
    theme: &Theme,
) -> Vec<PaintQuad> {
    if selected.is_empty() {
        return Vec::new();
    }
    let mut quads = Vec::new();
    for line in lines {
        let start = selected.start.max(line.range.start);
        let end = selected.end.min(line.range.end);
        if start > end || (start == end && !line.range.is_empty()) {
            continue;
        }
        let local_start = start.saturating_sub(line.range.start).min(line.range.len());
        let local_end = end.saturating_sub(line.range.start).min(line.range.len());
        quads.push(fill(
            Bounds::from_corners(
                point(
                    bounds.left() + line.shaped.x_for_index(local_start),
                    bounds.top() + line.top,
                ),
                point(
                    bounds.left() + line.shaped.x_for_index(local_end),
                    bounds.top() + line.top + line_height,
                ),
            ),
            selection_bg(theme),
        ));
    }
    quads
}

pub(super) fn cursor_quad(
    lines: &[LineLayout],
    cursor: usize,
    bounds: Bounds<Pixels>,
    line_height: Pixels,
    color: u32,
) -> Option<PaintQuad> {
    let line = lines
        .iter()
        .find(|line| line.range.start <= cursor && cursor <= line.range.end)
        .or_else(|| lines.last())?;
    let local = cursor
        .saturating_sub(line.range.start)
        .min(line.range.len());
    Some(fill(
        Bounds::new(
            point(
                bounds.left() + line.shaped.x_for_index(local),
                bounds.top() + line.top,
            ),
            size(px(1.5), line_height),
        ),
        rgb(color),
    ))
}
