use gpui::{
    AnyElement, FontWeight, InteractiveElement, IntoElement, ParentElement, Pixels, SharedString,
    Styled, Window, div, px, rgb,
};
use jayjay_markdown::{MarkdownBlock, MarkdownDocument, MarkdownImageAlign, MarkdownListItem};

use crate::app::fonts;
use crate::app::theme::Theme;

use super::table::table_block;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MarkdownDocumentStyle {
    Markdown,
    TableProjection,
}

impl MarkdownDocumentStyle {
    pub(super) fn is_table_projection(self) -> bool {
        self == Self::TableProjection
    }
}

pub(super) fn markdown_document(
    document: &MarkdownDocument,
    style: MarkdownDocumentStyle,
    available_width: Option<Pixels>,
    t: &Theme,
    window: &Window,
) -> AnyElement {
    let mut col = div()
        .debug_selector(|| "markdown-document".to_owned())
        .flex()
        .flex_col()
        // flex_none keeps the document at natural height inside the scroll column so long content overflows and scrolls instead of being shrunk to fit.
        .flex_none()
        .w_full()
        .text_color(rgb(t.fg));
    if style.is_table_projection() {
        col = col.px(px(22.)).py(px(18.));
    } else {
        col = col.gap(px(10.)).px(px(18.)).py(px(16.));
    }

    if document.blocks().is_empty() {
        col = col.child(
            div()
                .text_size(px(13.))
                .text_color(rgb(t.fg_dim))
                .child("(empty Markdown document)"),
        );
    }

    for block in document.blocks() {
        col = col.child(block_element(block, style, available_width, t, window));
    }
    col.into_any_element()
}

fn block_element(
    block: &MarkdownBlock,
    style: MarkdownDocumentStyle,
    available_width: Option<Pixels>,
    t: &Theme,
    window: &Window,
) -> AnyElement {
    match block {
        MarkdownBlock::Heading { level, text } => heading_block(*level, text, t),
        MarkdownBlock::Paragraph(text) => paragraph_block(text, t),
        MarkdownBlock::CodeBlock { language, text } => code_block(language.as_deref(), text, t),
        MarkdownBlock::Image {
            source, alt, align, ..
        } => image_block(source, alt, *align, t),
        MarkdownBlock::BlockQuote(text) => quote_block(text, t),
        MarkdownBlock::List { start, items } => list_block(*start, items, t),
        MarkdownBlock::Table { rows } => table_block(rows, style, available_width, t, window),
        MarkdownBlock::Rule => div()
            .h(px(1.))
            .w_full()
            .bg(rgb(t.border))
            .into_any_element(),
    }
}

fn image_block(source: &str, alt: &str, align: MarkdownImageAlign, t: &Theme) -> AnyElement {
    let label = if alt.is_empty() { "Image" } else { alt };
    let mut wrapper = div().flex().w_full();
    if align == MarkdownImageAlign::Center {
        wrapper = wrapper.justify_center();
    }
    wrapper
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(4.))
                .max_w(px(420.))
                .rounded_sm()
                .border_1()
                .border_color(rgb(t.border))
                .bg(rgb(t.header_bg))
                .px(px(10.))
                .py(px(8.))
                .child(
                    div()
                        .text_size(px(12.))
                        .line_height(px(18.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(t.fg))
                        .child(SharedString::from(label.to_owned())),
                )
                .child(
                    div()
                        .font_family(fonts::mono())
                        .text_size(px(10.))
                        .line_height(px(14.))
                        .text_color(rgb(t.fg_dim))
                        .child(SharedString::from(source.to_owned())),
                ),
        )
        .into_any_element()
}

fn heading_block(level: u8, text: &str, t: &Theme) -> AnyElement {
    let (size, weight) = match level {
        1 => (22., FontWeight::BOLD),
        2 => (18., FontWeight::SEMIBOLD),
        3 => (15., FontWeight::SEMIBOLD),
        _ => (13., FontWeight::SEMIBOLD),
    };
    let mut el = div()
        .text_size(px(size))
        .line_height(px(size + 5.))
        .font_weight(weight)
        .text_color(rgb(t.fg))
        .child(SharedString::from(text.to_owned()));
    if level <= 2 {
        el = el.pb(px(4.)).border_b_1().border_color(rgb(t.border));
    }
    el.into_any_element()
}

fn paragraph_block(text: &str, t: &Theme) -> AnyElement {
    div()
        .text_size(px(13.))
        .line_height(px(20.))
        .text_color(rgb(t.fg))
        .child(SharedString::from(text.to_owned()))
        .into_any_element()
}

fn quote_block(text: &str, t: &Theme) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .items_start()
        .gap(px(10.))
        .px(px(10.))
        .py(px(8.))
        .rounded_sm()
        .bg(rgb(t.header_bg))
        .child(
            div()
                .w(px(3.))
                .h_full()
                .min_h(px(20.))
                .rounded_sm()
                .bg(rgb(t.border)),
        )
        .child(
            div()
                .flex_1()
                .text_size(px(13.))
                .line_height(px(20.))
                .text_color(rgb(t.fg_dim))
                .child(SharedString::from(text.to_owned())),
        )
        .into_any_element()
}

fn code_block(language: Option<&str>, text: &str, t: &Theme) -> AnyElement {
    let mut frame = div()
        .flex()
        .flex_col()
        .rounded_sm()
        .border_1()
        .border_color(rgb(t.border))
        .bg(rgb(t.header_bg));

    if let Some(language) = language {
        frame = frame.child(
            div()
                .px(px(10.))
                .py(px(5.))
                .border_b_1()
                .border_color(rgb(t.border))
                .font_family(fonts::mono())
                .text_size(px(10.))
                .text_color(rgb(t.fg_dim))
                .child(SharedString::from(language.to_owned())),
        );
    }

    frame
        .child(
            div()
                .px(px(10.))
                .py(px(8.))
                .font_family(fonts::mono())
                .text_size(px(12.))
                .line_height(px(18.))
                .text_color(rgb(t.fg))
                .child(SharedString::from(text.to_owned())),
        )
        .into_any_element()
}

fn list_block(start: Option<u64>, items: &[MarkdownListItem], t: &Theme) -> AnyElement {
    let mut list = div().flex().flex_col().gap(px(4.));
    for (index, item) in items.iter().enumerate() {
        list = list.child(list_item(start, index, item, t));
    }
    list.into_any_element()
}

fn list_item(start: Option<u64>, index: usize, item: &MarkdownListItem, t: &Theme) -> AnyElement {
    let marker = match item.checked {
        Some(true) => "[x]".to_owned(),
        Some(false) => "[ ]".to_owned(),
        None => start
            .map(|start| format!("{}.", start + index as u64))
            .unwrap_or_else(|| "-".to_owned()),
    };
    div()
        .flex()
        .flex_row()
        .items_start()
        .gap(px(8.))
        .child(
            div()
                .w(px(28.))
                .font_family(fonts::mono())
                .text_size(px(12.))
                .line_height(px(20.))
                .text_color(rgb(t.fg_dim))
                .child(SharedString::from(marker)),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .text_size(px(13.))
                .line_height(px(20.))
                .text_color(rgb(t.fg))
                .child(SharedString::from(item.text.clone())),
        )
        .into_any_element()
}
