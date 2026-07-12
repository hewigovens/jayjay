mod blocks;
mod table;

use gpui::{
    AnyElement, Context, Div, InteractiveElement, IntoElement, ParentElement, ScrollHandle,
    SharedString, StatefulInteractiveElement, Styled, Window, div, px, rgb,
};
use jayjay_core::DiffRenderKind;
use jayjay_markdown::MarkdownDocument;

use crate::app::theme::Theme;
use crate::diff::bounds_capture;
use crate::diff::media_diff::{format_size, rich_preview_with_gutter, single_pane_layout};
use crate::repo::window::{PanelBoundsSlot, RepoWindow};
use crate::ui::scrollbar::vertical_scrollbar;

use blocks::{MarkdownDocumentStyle, markdown_document};

pub(crate) fn markdown_diff_view(
    document: Option<&MarkdownDocument>,
    scroll: ScrollHandle,
    bounds: PanelBoundsSlot,
    render_kind: Option<DiffRenderKind>,
    t: &Theme,
    window: &Window,
    cx: &Context<RepoWindow>,
) -> AnyElement {
    let style = match render_kind {
        Some(DiffRenderKind::Table) => MarkdownDocumentStyle::TableProjection,
        _ => MarkdownDocumentStyle::Markdown,
    };
    let viewer = markdown_viewer(document, scroll, bounds, style, t, window, cx);
    let mut pane = div()
        .flex()
        .flex_col()
        .flex_1()
        .min_w_0()
        .min_h_0()
        .gap(px(8.))
        .child(viewer);
    if !style.is_table_projection() {
        pane = pane.child(metadata_line(document, t));
    }
    rich_preview_with_gutter(single_pane_layout(pane.into_any_element(), t), t)
}

fn markdown_viewer(
    document: Option<&MarkdownDocument>,
    scroll: ScrollHandle,
    bounds: PanelBoundsSlot,
    style: MarkdownDocumentStyle,
    t: &Theme,
    window: &Window,
    cx: &Context<RepoWindow>,
) -> AnyElement {
    let chrome = match style {
        MarkdownDocumentStyle::Markdown => markdown_frame(t),
        MarkdownDocumentStyle::TableProjection => table_projection_frame(t),
    }
    .relative();

    let scroller = div()
        .id(SharedString::from("markdown-preview"))
        .debug_selector(|| "markdown-preview-pane".to_owned())
        .flex()
        // Column layout: height (main axis) grows past the viewport for scrolling while the cross-axis stretch clamps the document to the pane width so text wraps.
        .flex_col()
        .size_full()
        .justify_start()
        .overflow_y_scroll()
        .scrollbar_width(px(0.))
        .track_scroll(&scroll);
    let available_width = bounds.get().map(|bounds| bounds.size.width);
    let scroller = match document {
        Some(document) if !document.source().trim().is_empty() => scroller.child(
            markdown_document(document, style, available_width, t, window),
        ),
        _ => scroller
            .items_center()
            .justify_center()
            .text_color(rgb(t.fg_dim))
            .child("No post-change Markdown content."),
    };

    chrome
        .child(scroller)
        .child(bounds_capture(bounds))
        .child(vertical_scrollbar(scroll, t, cx))
        .into_any_element()
}

fn table_projection_frame(t: &Theme) -> Div {
    div()
        .flex()
        .flex_1()
        .w_full()
        .min_w_0()
        .min_h_0()
        .bg(rgb(t.detail_bg))
}

fn markdown_frame(t: &Theme) -> Div {
    div()
        .flex()
        .flex_1()
        .w_full()
        .min_w_0()
        .min_h_0()
        .rounded_md()
        .border_1()
        .border_color(rgb(t.border))
        .bg(rgb(t.detail_bg))
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
