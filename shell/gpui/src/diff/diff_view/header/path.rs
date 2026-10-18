use gpui::{
    AnyElement, Context, FontWeight, HighlightStyle, InteractiveElement, IntoElement,
    ParentElement, SharedString, StatefulInteractiveElement, Styled, StyledText, div, px, rgb,
};
use jayjay_core::{FileKind, SetiGlyph};
use unicode_width::UnicodeWidthStr;

use super::{COMPACT_HEADER_WIDTH, DETAIL_INSET, FileHeaderState};
use crate::app::fonts;
use crate::app::fonts::CodeText as _;
use crate::app::theme::{Theme, ui_font_size};
use crate::diff::file_column::head_elide;
use crate::repo::window::RepoWindow;
use crate::ui::icons::{self, FileKindGlyph as _, glyph};
use crate::ui::primitives::text_tooltip;

pub(super) const PATH_FONT_SIZE: f32 = 13.;
const PATH_LINE_HEIGHT: f32 = 16.;

pub(super) fn file_kind_icon(path: &str, t: &Theme) -> impl IntoElement {
    let base = t.scaled_font_size(PATH_FONT_SIZE);
    let (glyph, size, (left, width)) = match SetiGlyph::for_path(path) {
        Some(seti) => {
            let size = base * 1.5;
            let glyph = div()
                .font_family(icons::SETI_FONT)
                .text_size(px(size))
                .text_color(rgb(t.fg_dim))
                .child(SharedString::from(seti.0.to_string()));
            (glyph, size, seti.ink())
        }
        None => {
            let kind = FileKind::for_path(path);
            (icons::icon(kind.glyph(), base, t.fg_dim), base, kind.ink())
        }
    };
    // Sized to the ink so the icon takes the row's usual gap; icon fonts sit high, like the copy button's glyph.
    div()
        .flex()
        .flex_none()
        .w(px(width * size))
        .line_height(px(t.scaled_font_size(PATH_LINE_HEIGHT)))
        .relative()
        .top(px(1.))
        .child(glyph.flex_none().ml(px(-left * size)))
}

pub(super) fn file_path_label(path: &str, max_chars: usize, t: &Theme) -> impl IntoElement {
    let display = SharedString::from(head_elide(path, max_chars));
    let dir_len = display.rfind('/').map(|ix| ix + 1).unwrap_or(0);
    let text = div()
        .id("diff-file-path")
        .debug_selector(|| "diff-file-path".to_owned())
        .flex_shrink_1()
        .min_w_0()
        .truncate()
        .text_size(ui_font_size(PATH_FONT_SIZE))
        .line_height(px(t.scaled_font_size(PATH_LINE_HEIGHT)))
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

/// Only a rename's source is struck through; a copy's source still exists.
pub(super) fn source_path_label(
    old_path: &str,
    renamed: bool,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let width = mono_text_width(old_path, px(t.scaled_code_font_size(11.)), cx);
    let path = div()
        .id("diff-file-old-path")
        .debug_selector(|| "diff-file-old-path".to_owned())
        .w(width)
        .flex_shrink_1()
        .min_w_0()
        .truncate()
        .code_text(11., t)
        .text_color(rgb(t.fg_dim))
        .tooltip(text_tooltip(old_path.to_owned()))
        .child(SharedString::from(old_path.to_owned()));
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(4.))
        .min_w_0()
        .child(if renamed { path.line_through() } else { path })
        .child(icons::icon(glyph::ARROW_RIGHT, 10., t.fg_dim))
        .into_any_element()
}

pub(super) fn header_reserved_width(
    state: &FileHeaderState<'_>,
    status: &str,
    show_projection_button: bool,
    has_old_path: bool,
    t: &Theme,
) -> f32 {
    let text = |chars: usize, size: f32| chars as f32 * t.scaled_font_size(size) * 0.55;
    let compact = state.detail_width < COMPACT_HEADER_WIDTH;
    let action = |label: &str| 28. + if compact { 0. } else { text(label.len(), 11.) };
    let mut reserved = 2. * DETAIL_INSET + 40. + 20.; // padding, inter-child gaps, file icon
    reserved += 20. + 16. + text(status.len(), 10.); // copy button, status pill
    if let Some(stats) = state.line_stats {
        reserved += text(
            format!("+{} -{}", stats.insertions, stats.deletions).len(),
            11.,
        );
    }
    reserved += action(super::controls::mode_label(state.view_mode));
    if state.can_edit_file {
        reserved += action("Edit File");
    }
    if state.can_edit_diff {
        reserved += action("Edit Diff");
    }
    if state.is_annotating {
        reserved += 100.;
    }
    if has_old_path {
        reserved += 170.;
    }
    let preview_buttons = show_projection_button as usize
        + state.can_render_markdown_preview as usize
        + state.can_render_svg_preview as usize;
    reserved += action("Preview") * preview_buttons as f32;
    reserved += 28. * state.html_external_url.is_some() as usize as f32;
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
    px((f32::from(advance) * text.width() as f32).ceil() + 2.)
}
