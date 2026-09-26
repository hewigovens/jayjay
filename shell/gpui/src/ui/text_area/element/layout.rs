use std::ops::Range;

use gpui::{Bounds, Font, Hsla, Pixels, ShapedLine, SharedString, TextRun, Window, hsla, px, rgb};
use jayjay_core::diff::DiffSpanStyle;

use super::super::{LineLayout, TextArea, Tint};
use super::gutter;
use crate::app::theme::Theme;
use crate::ui::input::{next_boundary, previous_boundary};

/// Captured so a measured layout can shape lines outside the element's own style scope.
pub(super) struct Typeset {
    font: Font,
    font_size: Pixels,
    color: Hsla,
    line_height: Pixels,
}

impl Typeset {
    pub(super) fn current(window: &Window) -> Self {
        let style = window.text_style();
        Self {
            font: style.font(),
            font_size: style.font_size.to_pixels(window.rem_size()),
            color: style.color,
            line_height: window.line_height(),
        }
    }

    fn shape(
        &self,
        text: SharedString,
        colors: Vec<(usize, Hsla, Option<Hsla>)>,
        window: &mut Window,
    ) -> ShapedLine {
        let runs = colors
            .into_iter()
            .map(|(len, color, background_color)| TextRun {
                len,
                font: self.font.clone(),
                color,
                background_color,
                underline: None,
                strikethrough: None,
            })
            .collect::<Vec<_>>();
        window
            .text_system()
            .shape_line(text, self.font_size, &runs, None)
    }
}

pub(super) fn build_lines(
    input: &TextArea,
    bounds: Bounds<Pixels>,
    typeset: &Typeset,
    window: &mut Window,
    theme: &Theme,
) -> (Vec<LineLayout>, Pixels) {
    let content = input.content.clone();
    let line_height = typeset.line_height;
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
            typeset.color.opacity(0.6)
        } else {
            typeset.color
        };
        if content.is_empty() {
            let placeholder_len = input.placeholder.len();
            lines.push(line_layout(
                logical_line_ix,
                0..0,
                px(lines.len() as f32 * f32::from(line_height)),
                DiffSpanStyle::Context,
                typeset.shape(
                    input.placeholder.clone(),
                    vec![(placeholder_len, text_run_color(true, typeset.color), None)],
                    window,
                ),
            ));
            continue;
        }
        if range.is_empty() {
            lines.push(line_layout(
                logical_line_ix,
                range,
                px(lines.len() as f32 * f32::from(line_height)),
                line_style,
                typeset.shape(
                    SharedString::from(""),
                    vec![(
                        0,
                        content_color,
                        line_background(line_style, DiffSpanStyle::Unchanged, theme),
                    )],
                    window,
                ),
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
            let text = SharedString::from(content[range.clone()].to_string());
            lines.push(line_layout(
                logical_line_ix,
                range,
                px(lines.len() as f32 * f32::from(line_height)),
                line_style,
                typeset.shape(text, runs, window),
            ));
            continue;
        }

        let mut start = range.start;
        let line_start = range.start;
        while start < range.end {
            let end = wrapped_segment_end(
                content.as_ref(),
                start..range.end,
                max_width,
                typeset,
                window,
            );
            let segment = SharedString::from(content[start..end].to_string());
            let runs = if input.tints.is_empty() {
                highlighted_runs(
                    input.syntax_spans(logical_line_ix),
                    start - line_start..end - line_start,
                    content_color,
                    line_style,
                    theme,
                )
            } else {
                tinted_runs(&input.tints, start..end, content_color, theme)
            };
            lines.push(line_layout(
                logical_line_ix,
                start..end,
                px(lines.len() as f32 * f32::from(line_height)),
                line_style,
                typeset.shape(segment, runs, window),
            ));
            start = end;
        }
    }
    if input.line_numbers {
        number_first_rows(&mut lines, window, theme);
    }
    (lines, line_height)
}

fn number_first_rows(lines: &mut [LineLayout], window: &mut Window, theme: &Theme) {
    let color = rgb(theme.diff_gutter_fg).into();
    let mut numbered = None;
    for line in lines {
        if numbered == Some(line.logical_line) {
            continue;
        }
        numbered = Some(line.logical_line);
        let label = (line.logical_line + 1).to_string();
        line.number = Some(gutter::shape_number(label.into(), color, window));
    }
}

fn line_layout(
    logical_line: usize,
    range: Range<usize>,
    top: Pixels,
    style: DiffSpanStyle,
    shaped: ShapedLine,
) -> LineLayout {
    LineLayout {
        logical_line,
        range,
        shaped,
        number: None,
        top,
        style,
    }
}

fn tinted_runs(
    tints: &[Tint],
    segment: Range<usize>,
    fallback: Hsla,
    theme: &Theme,
) -> Vec<(usize, Hsla, Option<Hsla>)> {
    let mut runs = Vec::new();
    let mut at = segment.start;
    for tint in tints {
        let start = tint.range.start.clamp(at, segment.end);
        let end = tint.range.end.clamp(start, segment.end);
        if start > at {
            runs.push((start - at, fallback, None));
        }
        if end > start {
            runs.push((end - start, rgb((tint.color)(theme)).into(), None));
        }
        at = at.max(end);
    }
    if at < segment.end {
        runs.push((segment.end - at, fallback, None));
    }
    runs
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
    typeset: &Typeset,
    window: &mut Window,
) -> usize {
    let text = &content[range.clone()];
    let shaped = typeset.shape(
        SharedString::from(text.to_string()),
        vec![(text.len(), typeset.color, None)],
        window,
    );
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
    fn tints_split_a_wrapped_segment_at_their_boundaries() {
        let theme = Theme::light();
        let fallback: Hsla = rgb(theme.fg_dim).into();
        let prefix: Hsla = rgb(theme.change_id_prefix).into();
        let tints = [Tint {
            range: 0..3,
            color: |theme| theme.change_id_prefix,
        }];
        let lens = |segment| {
            tinted_runs(&tints, segment, fallback, &theme)
                .into_iter()
                .map(|(len, color, _)| (len, color == prefix))
                .collect::<Vec<_>>()
        };
        assert_eq!(lens(0..8), [(3, true), (5, false)]);
        assert_eq!(lens(2..8), [(1, true), (5, false)]);
        assert_eq!(lens(4..8), [(4, false)]);
    }

    #[test]
    fn word_wrap_prefers_whitespace_boundary() {
        assert_eq!(word_wrap_boundary("alpha beta gamma", 10), 6);
    }

    #[test]
    fn word_wrap_hard_wraps_long_word() {
        assert_eq!(word_wrap_boundary("alphabetagamma", 7), 7);
    }
}
