use std::sync::Arc;

use gpui::{
    AnyElement, Div, IntoElement, ParentElement, SharedString, Styled, UniformListScrollHandle,
    div, px, rgb, uniform_list,
};
use jayjay_core::AnnotationLine;

use crate::app::fonts;
use crate::app::theme::{ANNOTATE_PALETTE, Theme};
use crate::ui::primitives::no_scrollbar_gutter;

fn change_color(change_id: &str) -> u32 {
    let bytes = change_id.as_bytes();
    let mut h: u32 = 0;
    for &b in bytes.iter().take(16) {
        h = h.wrapping_mul(31).wrapping_add(b as u32);
    }
    ANNOTATE_PALETTE[(h as usize) % ANNOTATE_PALETTE.len()]
}

pub fn annotate_body(
    lines: Arc<Vec<AnnotationLine>>,
    theme: Theme,
    scroll: UniformListScrollHandle,
) -> AnyElement {
    let count = lines.len();
    let theme = Arc::new(theme);
    let list = uniform_list(
        "annotate-lines",
        count,
        move |range: std::ops::Range<usize>, _w, _cx| {
            range.map(|ix| annotate_row(&lines[ix], &theme)).collect()
        },
    )
    .track_scroll(&scroll);
    no_scrollbar_gutter(list).h_full().into_any_element()
}

fn annotate_row(line: &AnnotationLine, t: &Theme) -> AnyElement {
    let short_id = line.change_id.chars().take(8).collect::<String>();
    let author_initials: String = line
        .author
        .split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase();
    let stripe_color = change_color(&line.change_id);

    div()
        .flex()
        .flex_row()
        .w_full()
        .h(px(18.))
        .font_family(fonts::mono())
        .text_size(px(12.))
        .line_height(px(18.))
        .child(stripe(stripe_color))
        .child(line_no_cell(line.line_number, t))
        .child(change_cell(short_id, t))
        .child(author_cell(&author_initials, &line.author, t))
        .child(date_cell(&line.timestamp, t))
        .child(text_cell(&line.text, t))
        .into_any_element()
}

fn stripe(color: u32) -> Div {
    div().flex_none().w(px(3.)).h(px(18.)).bg(rgb(color))
}

fn line_no_cell(line_no: u32, t: &Theme) -> Div {
    div()
        .flex_none()
        .w(px(36.))
        .h(px(18.))
        .px(px(4.))
        .text_color(rgb(t.diff_gutter_fg))
        .bg(rgb(t.diff_gutter_bg))
        .child(SharedString::from(line_no.to_string()))
}

fn change_cell(short_id: String, t: &Theme) -> Div {
    div()
        .flex_none()
        .w(px(64.))
        .h(px(18.))
        .px(px(4.))
        .text_color(rgb(t.fg_dim))
        .child(SharedString::from(short_id))
}

fn author_cell(initials: &str, full_name: &str, t: &Theme) -> Div {
    let _ = full_name; // todo: hover tooltip with full name
    div()
        .flex_none()
        .w(px(28.))
        .h(px(18.))
        .px(px(4.))
        .text_color(rgb(t.fg_dim))
        .child(SharedString::from(initials.to_owned()))
}

fn date_cell(ts: &str, t: &Theme) -> Div {
    let short = ts.chars().take(10).collect::<String>();
    div()
        .flex_none()
        .w(px(72.))
        .h(px(18.))
        .px(px(4.))
        .text_color(rgb(t.fg_faint))
        .child(SharedString::from(short))
}

fn text_cell(text: &str, t: &Theme) -> Div {
    div()
        .flex_1()
        .min_w_0()
        .h(px(18.))
        .px(px(8.))
        .text_color(rgb(t.fg))
        .child(SharedString::from(text.to_owned()))
}
