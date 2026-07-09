mod blocks;

use gpui::{
    AnyElement, Div, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px, rgb,
};
use jayjay_core::{DiffRenderKind, HunkType};
use jayjay_markdown::MarkdownDocument;

use crate::app::theme::Theme;
use crate::diff::media_diff::{
    MediaSide, format_size, media_diff_layout, media_pane, rich_preview_with_gutter,
};

use blocks::{MarkdownDocumentStyle, markdown_document};

#[derive(Clone, Copy)]
pub(crate) struct MarkdownDiffContent<'a> {
    pub(crate) old: Option<&'a MarkdownDocument>,
    pub(crate) new: Option<&'a MarkdownDocument>,
}

pub(crate) fn markdown_diff_view(
    content: MarkdownDiffContent<'_>,
    hunk_type: HunkType,
    render_kind: Option<DiffRenderKind>,
    t: &Theme,
) -> AnyElement {
    let style = match render_kind {
        Some(DiffRenderKind::Table) => MarkdownDocumentStyle::TableProjection,
        _ => MarkdownDocumentStyle::Markdown,
    };
    let content = media_diff_layout(
        hunk_type,
        t,
        |side, label, label_bg, label_fg, show_label, t| {
            let document = match side {
                MediaSide::Old => content.old,
                MediaSide::New => content.new,
            };
            pane(document, label, label_bg, label_fg, show_label, style, t)
        },
    );
    rich_preview_with_gutter(content, t)
}

fn pane(
    document: Option<&MarkdownDocument>,
    label: &'static str,
    label_bg: u32,
    label_fg: u32,
    show_label: bool,
    style: MarkdownDocumentStyle,
    t: &Theme,
) -> AnyElement {
    let viewer = markdown_viewer(document, label, style, t);
    let meta = (!style.is_table_projection()).then(|| metadata_line(document, t));
    media_pane(label, label_bg, label_fg, show_label, viewer, meta)
}

fn markdown_viewer(
    document: Option<&MarkdownDocument>,
    label: &'static str,
    style: MarkdownDocumentStyle,
    t: &Theme,
) -> AnyElement {
    let frame = match style {
        MarkdownDocumentStyle::Markdown => markdown_frame(t),
        MarkdownDocumentStyle::TableProjection => table_projection_frame(t),
    }
    .id(SharedString::from(format!("markdown-preview-{label}")))
    .debug_selector(|| "markdown-preview-pane".to_owned())
    .overflow_y_scroll()
    .scrollbar_width(px(0.));
    match document {
        Some(document) if !document.source().trim().is_empty() => frame
            .child(markdown_document(document, style, t))
            .into_any_element(),
        _ => frame
            .items_center()
            .justify_center()
            .text_color(rgb(t.fg_dim))
            .child("-")
            .into_any_element(),
    }
}

fn table_projection_frame(t: &Theme) -> Div {
    div()
        .flex()
        .flex_1()
        .w_full()
        .min_h_0()
        .items_start()
        .justify_start()
        .bg(rgb(t.detail_bg))
}

fn markdown_frame(t: &Theme) -> Div {
    div()
        .flex()
        .flex_1()
        .w_full()
        .min_h_0()
        .items_start()
        .justify_start()
        .rounded_md()
        .border_1()
        .border_color(rgb(t.border))
        .bg(rgb(if t.is_dark { 0x14171c } else { 0xeef0f3 }))
}

fn metadata_line(document: Option<&MarkdownDocument>, t: &Theme) -> AnyElement {
    let label = document
        .map(|document| {
            format!(
                "{} blocks, {}",
                document.blocks().len(),
                format_size(document.source().len() as u64)
            )
        })
        .unwrap_or_else(|| " ".to_owned());
    crate::diff::media_diff::metadata_line(SharedString::from(label), t)
}
