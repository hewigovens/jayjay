use gpui::{
    AnyElement, Context, FontWeight, HighlightStyle, InteractiveElement, IntoElement,
    ParentElement, SharedString, StatefulInteractiveElement, Styled, StyledText, div, px, rgb,
};

use super::{DETAIL_INSET, FileHeaderState};
use crate::app::fonts;
use crate::app::theme::{Theme, ui_font_size};
use crate::diff::file_column::head_elide;
use crate::repo::window::RepoWindow;
use crate::ui::icons::{self, glyph};
use crate::ui::primitives::text_tooltip;

pub(super) fn file_path_label(path: &str, max_chars: usize, t: &Theme) -> impl IntoElement {
    let display = SharedString::from(head_elide(path, max_chars));
    let dir_len = display.rfind('/').map(|ix| ix + 1).unwrap_or(0);
    let text = div()
        .id("diff-file-path")
        .debug_selector(|| "diff-file-path".to_owned())
        .flex_shrink_1()
        .min_w_0()
        .truncate()
        .text_size(ui_font_size(13.))
        .line_height(px(t.scaled_font_size(16.)))
        .font_weight(FontWeight::MEDIUM)
        .text_color(rgb(t.fg))
        .tooltip(text_tooltip(path.to_owned()));
    if dir_len > 0 {
        text.child(StyledText::new(display).with_highlights([(
            0..dir_len,
            HighlightStyle {
                color: Some(rgb(t.fg_dim).into()),
                font_weight: Some(FontWeight::NORMAL),
                ..Default::default()
            },
        )]))
    } else {
        text.child(display)
    }
}

pub(super) fn rename_origin_label(
    old_path: &str,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let width = mono_text_width(old_path, px(t.scaled_font_size(11.)), cx);
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(4.))
        .min_w_0()
        .child(
            div()
                .id("diff-file-old-path")
                .debug_selector(|| "diff-file-old-path".to_owned())
                .w(width)
                .flex_shrink_1()
                .min_w_0()
                .truncate()
                .font_family(fonts::mono())
                .text_size(ui_font_size(11.))
                .text_color(rgb(t.fg_dim))
                .line_through()
                .tooltip(text_tooltip(old_path.to_owned()))
                .child(SharedString::from(old_path.to_owned())),
        )
        .child(icons::icon(glyph::ARROW_RIGHT, 10., t.fg_dim))
        .into_any_element()
}

pub(super) fn header_reserved_width(
    state: &FileHeaderState<'_>,
    show_projection_button: bool,
    has_old_path: bool,
) -> f32 {
    let mut reserved = 2. * DETAIL_INSET + 60.; // padding + inter-child gaps
    reserved += 20.; // copy button
    reserved += 76.; // status pill
    reserved += 110.; // labeled view-mode toggle
    if state.can_edit_file {
        reserved += 26.;
    }
    if state.can_edit_diff {
        reserved += 100.;
    }
    if state.is_annotating {
        reserved += 100.;
    }
    if has_old_path {
        reserved += 170.;
    }
    let preview_buttons = show_projection_button as usize
        + state.html_external_url.is_some() as usize
        + state.can_render_markdown_preview as usize
        + state.can_render_svg_preview as usize;
    reserved += 28. * preview_buttons as f32;
    reserved
}

pub(super) fn path_budget_chars(detail_width: f32, reserved: f32, t: &Theme) -> usize {
    (((detail_width - reserved).max(0.)) / (t.scaled_font_size(13.) * 0.52)) as usize
}

fn mono_text_width(
    text: &str,
    font_size: gpui::Pixels,
    cx: &mut Context<RepoWindow>,
) -> gpui::Pixels {
    let advance = fonts::mono_advance(cx, font_size);
    px((f32::from(advance) * text.chars().count() as f32).ceil() + 2.)
}
