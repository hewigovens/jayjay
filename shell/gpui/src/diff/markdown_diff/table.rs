//! Content-based column sizing for Markdown tables (CSS auto-table-layout approximation):
//! each column gets exactly the width its content needs, with the widest column absorbing
//! any leftover space, so short Yes/No-style columns don't waste width and long
//! description columns get the room to wrap.
use gpui::{
    AnyElement, Font, FontWeight, InteractiveElement, IntoElement, ParentElement, Pixels,
    SharedString, Styled, TextRun, Window, black, div, px, rgb,
};
use jayjay_markdown::MarkdownTableRow;

use crate::app::theme::Theme;

use super::blocks::MarkdownDocumentStyle;

// Doc column horizontal padding (`markdown_document`'s `.px(...)`) that the table's
// available width sits inside, plus the fixed 1px borders around it.
const TABLE_BORDER: f32 = 2.;
const MARKDOWN_FRAME_BORDER: f32 = 2.;

// Used before the pane's bounds are known (first frame, before the bounds canvas paints).
const FALLBACK_AVAILABLE_WIDTH: f32 = 640.;

pub(super) fn table_block(
    rows: &[MarkdownTableRow],
    style: MarkdownDocumentStyle,
    available_width: Option<Pixels>,
    t: &Theme,
    window: &Window,
) -> AnyElement {
    let mut table = div()
        .flex()
        .flex_col()
        .border_1()
        .border_color(rgb(t.border));
    if style.is_table_projection() {
        table = table
            .w_full()
            .debug_selector(|| "markdown-table-preview".to_owned());
    } else {
        table = table.min_w(px(360.)).rounded_sm();
    }

    let column_count = rows.iter().map(|row| row.cells.len()).max().unwrap_or(0);
    let widths = column_widths(rows, column_count, style, available_width, window);
    for row in rows {
        table = table.child(table_row(row, style, &widths, t));
    }

    let mut wrapper = div();
    if style.is_table_projection() {
        wrapper = wrapper.w_full();
    }
    wrapper.child(table).into_any_element()
}

fn table_row(
    row: &MarkdownTableRow,
    style: MarkdownDocumentStyle,
    widths: &[Pixels],
    t: &Theme,
) -> AnyElement {
    let mut el = div().flex().flex_row();
    for (i, cell) in row.cells.iter().enumerate() {
        let mut cell_el = div()
            .px(px(8.))
            .py(px(6.))
            .border_r_1()
            .border_b_1()
            .border_color(rgb(t.border))
            .text_size(px(12.))
            .line_height(px(18.))
            .text_color(rgb(t.fg))
            .child(SharedString::from(cell.clone()));
        if style.is_table_projection() {
            cell_el = cell_el
                .px(px(9.))
                .py(px(8.))
                .text_size(px(14.))
                .line_height(px(21.));
        }
        // Ragged rows (more cells than the computed column count) fall back to the old
        // equal-share behavior for the overflowing cells rather than panicking or dropping them.
        cell_el = match widths.get(i) {
            Some(&width) => cell_el.w(width).flex_none(),
            None => cell_el.min_w(px(120.)).flex_1(),
        };
        if row.header {
            cell_el = cell_el
                .bg(rgb(if style.is_table_projection() {
                    t.row_alt_bg
                } else {
                    t.header_bg
                }))
                .font_weight(FontWeight::SEMIBOLD);
        }
        el = el.child(cell_el);
    }
    el.into_any_element()
}

fn column_widths(
    rows: &[MarkdownTableRow],
    column_count: usize,
    style: MarkdownDocumentStyle,
    available_width: Option<Pixels>,
    window: &Window,
) -> Vec<Pixels> {
    if column_count == 0 {
        return Vec::new();
    }
    let (max_content, min_content) = measure_columns(rows, column_count, style, window);
    let available = available_width
        .map(f32::from)
        .unwrap_or(FALLBACK_AVAILABLE_WIDTH)
        - chrome_padding(style);
    distribute_column_widths(&max_content, &min_content, available.max(0.))
        .into_iter()
        .map(px)
        .collect()
}

fn chrome_padding(style: MarkdownDocumentStyle) -> f32 {
    let doc_padding = if style.is_table_projection() {
        22.
    } else {
        18.
    } * 2.;
    let frame_border = if style.is_table_projection() {
        0.
    } else {
        MARKDOWN_FRAME_BORDER
    };
    doc_padding + frame_border + TABLE_BORDER
}

fn cell_font_size(style: MarkdownDocumentStyle) -> f32 {
    if style.is_table_projection() {
        14.
    } else {
        12.
    }
}

fn cell_padding(style: MarkdownDocumentStyle) -> f32 {
    (if style.is_table_projection() { 9. } else { 8. }) * 2.
}

fn measure_columns(
    rows: &[MarkdownTableRow],
    column_count: usize,
    style: MarkdownDocumentStyle,
    window: &Window,
) -> (Vec<f32>, Vec<f32>) {
    let size = px(cell_font_size(style));
    let padding = cell_padding(style);
    let mut max_content = vec![0f32; column_count];
    let mut min_content = vec![0f32; column_count];
    for row in rows {
        let weight = if row.header {
            FontWeight::SEMIBOLD
        } else {
            FontWeight::default()
        };
        for (i, cell) in row.cells.iter().enumerate().take(column_count) {
            max_content[i] =
                max_content[i].max(measure_width(window, cell, size, weight) + padding);
            min_content[i] =
                min_content[i].max(longest_word_width(window, cell, size, weight) + padding);
        }
    }
    (max_content, min_content)
}

fn measure_width(window: &Window, text: &str, size: Pixels, weight: FontWeight) -> f32 {
    if text.is_empty() {
        return 0.;
    }
    let font = Font {
        weight,
        ..Font::default()
    };
    let run = TextRun {
        len: text.len(),
        font,
        color: black(),
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    f32::from(
        window
            .text_system()
            .layout_line(text, size, &[run], None)
            .width,
    )
}

fn longest_word_width(window: &Window, text: &str, size: Pixels, weight: FontWeight) -> f32 {
    text.split_whitespace()
        .map(|word| measure_width(window, word, size, weight))
        .fold(0f32, f32::max)
}

/// Approximates CSS auto-table-layout: when every column's natural content width fits the
/// available space, each gets exactly that width and the widest column absorbs the leftover
/// (matching the "Notes" column in a feature-matrix table). When it doesn't fit, only the
/// columns wider than a fair share shrink — proportionally to their own content width — and
/// never below their min-content floor (the longest unbreakable word), so short columns
/// (e.g. "Yes"/"No") keep their natural width.
pub(super) fn distribute_column_widths(
    max_content: &[f32],
    min_content: &[f32],
    available: f32,
) -> Vec<f32> {
    let n = max_content.len();
    if n == 0 {
        return Vec::new();
    }

    let total_max: f32 = max_content.iter().sum();
    if total_max <= available {
        let mut widths = max_content.to_vec();
        let leftover = available - total_max;
        if leftover > 0. {
            let grow = widths
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.total_cmp(b.1))
                .map(|(i, _)| i)
                .unwrap_or(n - 1);
            widths[grow] += leftover;
        }
        return widths;
    }

    let fair_share = available / n as f32;
    let is_wide: Vec<bool> = max_content.iter().map(|&w| w > fair_share).collect();
    let narrow_total: f32 = max_content
        .iter()
        .enumerate()
        .filter(|&(i, _)| !is_wide[i])
        .map(|(_, &w)| w)
        .sum();
    let wide_max_total: f32 = max_content
        .iter()
        .enumerate()
        .filter(|&(i, _)| is_wide[i])
        .map(|(_, &w)| w)
        .sum();
    if wide_max_total <= 0. {
        return max_content.to_vec();
    }
    let remaining = (available - narrow_total).max(0.);

    let mut widths = max_content.to_vec();
    for (i, &wide) in is_wide.iter().enumerate() {
        if !wide {
            continue;
        }
        let share = remaining * (max_content[i] / wide_max_total);
        widths[i] = share.max(min_content[i]);
    }
    widths
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fits_gives_each_column_its_content_width_and_grows_the_widest() {
        let max_content = [40., 40., 200.];
        let min_content = [30., 30., 60.];
        let widths = distribute_column_widths(&max_content, &min_content, 400.);

        assert_eq!(widths[0], 40.);
        assert_eq!(widths[1], 40.);
        // Leftover (400 - 280 = 120) goes to the widest (Notes-like) column.
        assert_eq!(widths[2], 320.);
    }

    #[test]
    fn exact_fit_leaves_widths_untouched() {
        let max_content = [50., 50.];
        let min_content = [20., 20.];
        let widths = distribute_column_widths(&max_content, &min_content, 100.);

        assert_eq!(widths, vec![50., 50.]);
    }

    #[test]
    fn doesnt_fit_shrinks_only_wide_columns_proportionally() {
        // "SwiftUI"/"GPUI" style narrow columns keep their width; a single wide column
        // shrinks to fill what's left.
        let max_content = [40., 40., 600.];
        let min_content = [30., 30., 60.];
        let widths = distribute_column_widths(&max_content, &min_content, 300.);

        assert_eq!(widths[0], 40.);
        assert_eq!(widths[1], 40.);
        assert_eq!(widths[2], 220.);
    }

    #[test]
    fn doesnt_fit_splits_remaining_space_across_multiple_wide_columns_by_weight() {
        let max_content = [30., 300., 600.];
        let min_content = [20., 60., 60.];
        let widths = distribute_column_widths(&max_content, &min_content, 330.);

        assert_eq!(widths[0], 30.);
        // Remaining 300 split 300:600 between the two wide columns.
        assert_eq!(widths[1], 100.);
        assert_eq!(widths[2], 200.);
    }

    #[test]
    fn shrink_never_goes_below_min_content_floor() {
        let max_content = [40., 40., 600.];
        let min_content = [30., 30., 500.];
        let widths = distribute_column_widths(&max_content, &min_content, 300.);

        assert_eq!(widths[0], 40.);
        assert_eq!(widths[1], 40.);
        // Proportional share (220) would undercut the longest word (500); floor wins,
        // so the total legitimately exceeds `available` here — the table overflows
        // rather than breaking an unbreakable word, matching CSS min-content behavior.
        assert_eq!(widths[2], 500.);
    }

    #[test]
    fn single_column_table_absorbs_all_available_width() {
        let widths = distribute_column_widths(&[80.], &[40.], 500.);
        assert_eq!(widths, vec![500.]);
    }

    #[test]
    fn empty_table_returns_no_columns() {
        assert_eq!(distribute_column_widths(&[], &[], 500.), Vec::<f32>::new());
    }
}
