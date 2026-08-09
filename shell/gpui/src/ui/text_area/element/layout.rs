use std::ops::Range;

use gpui::{Bounds, Hsla, Pixels, SharedString, TextRun, Window, hsla, px, rgb};
use jayjay_core::diff::DiffSpanStyle;

use super::super::{LineLayout, TextArea};
use crate::app::theme::Theme;
use crate::ui::input::{next_boundary, previous_boundary};

pub(super) fn build_lines(
    input: &TextArea,
    bounds: Bounds<Pixels>,
    window: &mut Window,
    theme: &Theme,
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
    for (logical_line_ix, range) in ranges.into_iter().enumerate() {
        let line_style = input.line_style(logical_line_ix);
        let content_color = if input
            .emphasized_line()
            .is_some_and(|line| line != logical_line_ix)
        {
            style.color.opacity(0.6)
        } else {
            style.color
        };
        if content.is_empty() {
            let placeholder_len = input.placeholder.len();
            lines.push(line_layout(
                input.placeholder.clone(),
                0..0,
                px(lines.len() as f32 * f32::from(line_height)),
                font_size,
                DiffSpanStyle::Context,
                vec![(placeholder_len, text_run_color(true, style.color), None)],
                window,
            ));
            continue;
        }
        if range.is_empty() {
            lines.push(line_layout(
                SharedString::from(""),
                range,
                px(lines.len() as f32 * f32::from(line_height)),
                font_size,
                line_style,
                vec![(
                    0,
                    content_color,
                    line_background(line_style, DiffSpanStyle::Unchanged, theme),
                )],
                window,
            ));
            continue;
        }

        if input.is_selectable_code() {
            let runs = highlighted_runs(
                input.syntax_spans(logical_line_ix),
                0..range.len(),
                content_color,
                line_style,
                theme,
            );
            lines.push(line_layout(
                SharedString::from(content[range.clone()].to_string()),
                range,
                px(lines.len() as f32 * f32::from(line_height)),
                font_size,
                line_style,
                runs,
                window,
            ));
            continue;
        }

        let mut start = range.start;
        let line_start = range.start;
        while start < range.end {
            let end = wrapped_segment_end(content.as_ref(), start..range.end, max_width, window);
            let segment = SharedString::from(content[start..end].to_string());
            let runs = highlighted_runs(
                input.syntax_spans(logical_line_ix),
                start - line_start..end - line_start,
                content_color,
                line_style,
                theme,
            );
            lines.push(line_layout(
                segment,
                start..end,
                px(lines.len() as f32 * f32::from(line_height)),
                font_size,
                line_style,
                runs,
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
    top: Pixels,
    font_size: Pixels,
    style: DiffSpanStyle,
    colors: Vec<(usize, Hsla, Option<Hsla>)>,
    window: &mut Window,
) -> LineLayout {
    let font = window.text_style().font();
    let runs = colors
        .into_iter()
        .map(|(len, color, background_color)| TextRun {
            len,
            font: font.clone(),
            color,
            background_color,
            underline: None,
            strikethrough: None,
        })
        .collect::<Vec<_>>();
    let shaped = window
        .text_system()
        .shape_line(display_text, font_size, &runs, None);
    LineLayout {
        range,
        shaped,
        top,
        style,
    }
}

fn highlighted_runs(
    spans: Option<&[jayjay_core::diff::DiffSpan]>,
    segment: Range<usize>,
    fallback: Hsla,
    line_style: DiffSpanStyle,
    theme: &Theme,
) -> Vec<(usize, Hsla, Option<Hsla>)> {
    let Some(spans) = spans else {
        return vec![(
            segment.len(),
            fallback,
            line_background(line_style, DiffSpanStyle::Unchanged, theme),
        )];
    };
    let mut runs = Vec::new();
    let mut span_start = 0;
    let mut covered = segment.start;
    for span in spans {
        let span_end = span_start + span.text.len();
        let start = span_start.max(segment.start);
        let end = span_end.min(segment.end);
        if start < end {
            if covered < start {
                runs.push((
                    start - covered,
                    fallback,
                    line_background(line_style, DiffSpanStyle::Unchanged, theme),
                ));
            }
            let color = theme
                .syntax_token_color(span.token)
                .map(|color| rgb(color).into())
                .unwrap_or(fallback);
            runs.push((
                end - start,
                color,
                line_background(line_style, span.style, theme),
            ));
            covered = end;
        }
        span_start = span_end;
        if span_start >= segment.end {
            break;
        }
    }
    if covered < segment.end {
        runs.push((
            segment.end - covered,
            fallback,
            line_background(line_style, DiffSpanStyle::Unchanged, theme),
        ));
    }
    if runs.is_empty() {
        runs.push((
            segment.len(),
            fallback,
            line_background(line_style, DiffSpanStyle::Unchanged, theme),
        ));
    }
    runs
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
        px(0.),
        window.text_style().font_size.to_pixels(window.rem_size()),
        DiffSpanStyle::Context,
        vec![(text.len(), window.text_style().color, None)],
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

fn line_background(
    line_style: DiffSpanStyle,
    span_style: DiffSpanStyle,
    theme: &Theme,
) -> Option<Hsla> {
    match span_style {
        DiffSpanStyle::Added => Some(rgb(theme.diff_added_word_bg).into()),
        DiffSpanStyle::Removed => Some(rgb(theme.diff_removed_word_bg).into()),
        _ => match line_style {
            DiffSpanStyle::Added => Some(rgb(theme.diff_added_bg).into()),
            DiffSpanStyle::Removed => Some(rgb(theme.diff_removed_bg).into()),
            _ => None,
        },
    }
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
