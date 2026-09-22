use gpui::{
    Bounds, Hsla, PaintQuad, Pixels, Point, ShapedLine, SharedString, TextRun, Window, fill, point,
    px, rgb, size,
};

use super::super::TextArea;
use crate::app::theme::Theme;

const NUMBER_PADDING: Pixels = px(6.);

pub(super) fn gutter_width(input: &TextArea, window: &mut Window) -> Pixels {
    if !input.line_numbers {
        return px(0.);
    }
    let digits = "0".repeat(input.logical_line_count().to_string().len());
    shape_number(digits.into(), gpui::black(), window).width() + NUMBER_PADDING * 2.
}

pub(super) fn shape_number(text: SharedString, color: Hsla, window: &mut Window) -> ShapedLine {
    let style = window.text_style();
    let font_size = style.font_size.to_pixels(window.rem_size());
    let run = TextRun {
        len: text.len(),
        font: style.font(),
        color,
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    window
        .text_system()
        .shape_line(text, font_size, &[run], None)
}

pub(super) fn gutter_quads(gutter: Bounds<Pixels>, theme: &Theme) -> [PaintQuad; 2] {
    [
        fill(gutter, rgb(theme.diff_gutter_bg)),
        fill(
            Bounds::new(
                point(gutter.right() - px(1.), gutter.top()),
                size(px(1.), gutter.size.height),
            ),
            rgb(theme.border),
        ),
    ]
}

pub(super) fn number_origin(
    gutter: Bounds<Pixels>,
    number: &ShapedLine,
    top: Pixels,
) -> Point<Pixels> {
    point(
        gutter.right() - NUMBER_PADDING - number.width(),
        gutter.top() + top,
    )
}
