use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    ParentElement, Pixels, SharedString, Styled, TextRun, WindowTextSystem, black, div, font, px,
    rgb, rgba,
};
use jayjay_core::diff::{DiffSpanStyle, MovedRow};

use crate::app::fonts;
use crate::app::theme::{Theme, with_alpha};
use crate::repo::window::RepoWindow;

const LABEL_FONT_SIZE: f32 = 11.;
const TRAILING_INSET: f32 = 8.;
const HORIZONTAL_PADDING: f32 = 6.;
const CODE_GAP: f32 = 4.;

pub(super) fn label_font_size(theme: &Theme) -> Pixels {
    px(theme
        .scaled_font_size(LABEL_FONT_SIZE)
        .min(theme.code_font_size()))
}

pub(super) fn label_text(moved: MovedRow, style: DiffSpanStyle) -> String {
    let arrow = if style == DiffSpanStyle::Removed {
        "→"
    } else {
        "←"
    };
    format!("{arrow} L{}", moved.counterpart_line)
}

pub(super) fn label_width(title: &str, size: Pixels, text_system: &WindowTextSystem) -> Pixels {
    let run = TextRun {
        len: title.len(),
        font: font(fonts::interface()),
        color: black(),
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    text_system.layout_line(title, size, &[run], None).width
        + px(TRAILING_INSET + HORIZONTAL_PADDING * 2. + CODE_GAP)
}

pub(super) fn moved_label(
    id: (&'static str, usize),
    moved: MovedRow,
    style: DiffSpanStyle,
    first_fragment: bool,
    theme: &Theme,
    cx: &mut Context<RepoWindow>,
) -> Option<AnyElement> {
    if !(moved.starts_block && first_fragment) {
        return None;
    }
    let accent = theme.selected_accent;
    Some(
        div()
            .id(id)
            .debug_selector(move || format!("{}-{}", id.0, id.1))
            .absolute()
            .right(px(TRAILING_INSET))
            .top_0()
            .h_full()
            .flex()
            .items_center()
            .cursor_pointer()
            .child(
                div()
                    .px(px(HORIZONTAL_PADDING))
                    .rounded_md()
                    .font_family(fonts::interface())
                    .text_size(label_font_size(theme))
                    .line_height(px(theme.code_line_height()))
                    .text_color(rgb(accent))
                    .bg(rgba(with_alpha(accent, 0x24)))
                    .child(SharedString::from(label_text(moved, style))),
            )
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |view, _: &MouseDownEvent, _, cx| {
                    cx.stop_propagation();
                    view.reveal_moved_counterpart(style, moved.counterpart_line, cx);
                }),
            )
            .into_any_element(),
    )
}
