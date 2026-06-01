use std::ops::Range;

use gpui::{Bounds, Pixels, SharedString, TextRun, Window, hsla, px};

use super::super::{LineLayout, TextArea};
use crate::ui::input::{next_boundary, previous_boundary};

pub(super) fn build_lines(
    input: &TextArea,
    bounds: Bounds<Pixels>,
    window: &mut Window,
) -> (Vec<LineLayout>, Pixels) {
    let content = input.content.clone();
    let style = window.text_style();
    let font_size = style.font_size.to_pixels(window.rem_size());
    let line_height = window.line_height();
    let max_width = bounds.size.width.max(px(1.));
    let ranges = if content.is_empty() {
        std::iter::once(0..0).collect()
    } else {
        input.line_ranges()
    };
    let mut lines = Vec::new();
    for range in ranges {
        if content.is_empty() {
            lines.push(line_layout(
                input.placeholder.clone(),
                0..0,
                lines.len(),
                line_height,
                font_size,
                text_run_color(true, style.color),
                window,
            ));
            continue;
        }
        if range.is_empty() {
            lines.push(line_layout(
                SharedString::from(""),
                range,
                lines.len(),
                line_height,
                font_size,
                style.color,
                window,
            ));
            continue;
        }

        let mut start = range.start;
        while start < range.end {
            let end = wrapped_segment_end(content.as_ref(), start..range.end, max_width, window);
            let segment = SharedString::from(content[start..end].to_string());
            lines.push(line_layout(
                segment,
                start..end,
                lines.len(),
                line_height,
                font_size,
                style.color,
                window,
            ));
            start = end;
        }
    }
    (lines, line_height)
}

fn line_layout(
    display_text: SharedString,
    range: Range<usize>,
    line_ix: usize,
    line_height: Pixels,
    font_size: Pixels,
    color: gpui::Hsla,
    window: &mut Window,
) -> LineLayout {
    let run = TextRun {
        len: display_text.len(),
        font: window.text_style().font(),
        color,
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    let shaped = window
        .text_system()
        .shape_line(display_text, font_size, &[run], None);
    LineLayout {
        range,
        shaped,
        top: px(line_ix as f32 * f32::from(line_height)),
    }
}

fn wrapped_segment_end(
    content: &str,
    range: Range<usize>,
    max_width: Pixels,
    window: &mut Window,
) -> usize {
    let text = &content[range.clone()];
    let shaped = line_layout(
        SharedString::from(text.to_string()),
        range.clone(),
        0,
        px(0.),
        window.text_style().font_size.to_pixels(window.rem_size()),
        window.text_style().color,
        window,
    )
    .shaped;
    if shaped.width() <= max_width {
        return range.end;
    }

    let mut fit = shaped.closest_index_for_x(max_width).min(text.len());
    while fit > 0 && shaped.x_for_index(fit) > max_width {
        fit = previous_boundary(text, fit);
    }
    if fit == 0 {
        fit = next_boundary(text, 0);
    }

    let fit = word_wrap_boundary(text, fit);
    range.start + fit.min(text.len())
}

fn word_wrap_boundary(text: &str, fit: usize) -> usize {
    if fit >= text.len() {
        return text.len();
    }
    text[..fit]
        .char_indices()
        .filter_map(|(ix, ch)| ch.is_whitespace().then_some(ix + ch.len_utf8()))
        .next_back()
        .filter(|boundary| *boundary > 0)
        .unwrap_or(fit)
}

fn text_run_color(is_placeholder: bool, content_color: gpui::Hsla) -> gpui::Hsla {
    if is_placeholder {
        hsla(0., 0., 0.55, 0.62)
    } else {
        content_color
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_wrap_prefers_whitespace_boundary() {
        assert_eq!(word_wrap_boundary("alpha beta gamma", 10), 6);
    }

    #[test]
    fn word_wrap_hard_wraps_long_word() {
        assert_eq!(word_wrap_boundary("alphabetagamma", 7), 7);
    }
}
