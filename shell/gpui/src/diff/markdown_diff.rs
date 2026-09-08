mod blocks;
mod images;
mod table;

use gpui::{
    AnyElement, Context, Div, InteractiveElement, IntoElement, ParentElement, ScrollHandle,
    SharedString, StatefulInteractiveElement, Styled, Window, div, px, rgb,
};
use std::sync::Arc;

use jayjay_core::DiffRenderKind;
use jayjay_markdown::MarkdownDocument;

use crate::app::theme::Theme;
use crate::diff::bounds_capture;
use crate::diff::media_diff::{format_size, rich_preview_with_gutter, single_pane_layout};
use crate::repo::window::{PanelBoundsSlot, RepoWindow};
use crate::ui::scrollbar::vertical_scrollbar;

use blocks::{MarkdownDocumentStyle, markdown_document};
pub(crate) use images::{MarkdownImageCache, MarkdownImageCacheSlot, MarkdownImages};

pub(crate) struct MarkdownDiffState<'a> {
    pub(crate) document: Option<&'a Arc<MarkdownDocument>>,
    pub(crate) images: MarkdownImages<'a>,
    pub(crate) scroll: ScrollHandle,
    pub(crate) bounds: PanelBoundsSlot,
    pub(crate) render_kind: Option<DiffRenderKind>,
    pub(crate) shows_review: bool,
    pub(crate) theme: &'a Theme,
    pub(crate) window: &'a Window,
}

pub(crate) fn markdown_diff_view(
    state: MarkdownDiffState<'_>,
    cx: &Context<RepoWindow>,
) -> AnyElement {
    let t = state.theme;
    let style = match state.render_kind {
        Some(DiffRenderKind::Table) => MarkdownDocumentStyle::TableProjection,
        _ => MarkdownDocumentStyle::Markdown,
    };
    let viewer = markdown_viewer(&state, style, cx);
    let mut pane = div()
        .flex()
        .flex_col()
        .flex_1()
        .min_w_0()
        .min_h_0()
        .gap(px(8.))
        .child(viewer);
    if !style.is_table_projection() {
        pane = pane.child(metadata_line(state.document.map(Arc::as_ref), t));
    }
    rich_preview_with_gutter(
        single_pane_layout(pane.into_any_element(), t),
        t,
        state.shows_review,
    )
}

fn markdown_viewer(
    state: &MarkdownDiffState<'_>,
    style: MarkdownDocumentStyle,
    cx: &Context<RepoWindow>,
) -> AnyElement {
    let t = state.theme;
    let chrome = preview_frame(t).relative();

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
        .track_scroll(&state.scroll);
    let available_width = state.bounds.get().map(|bounds| bounds.size.width);
    let scroller = match state.document {
        Some(document) if !document.source().trim().is_empty() => {
            scroller.child(markdown_document(
                document,
                style,
                available_width,
                &state.images,
                t,
                state.window,
            ))
        }
        _ => scroller
            .items_center()
            .justify_center()
            .text_color(rgb(t.fg_dim))
            .child("No post-change Markdown content."),
    };

    chrome
        .child(scroller)
        .child(bounds_capture(state.bounds.clone()))
        .child(vertical_scrollbar(state.scroll.clone(), t, cx))
        .into_any_element()
}

fn preview_frame(t: &Theme) -> Div {
    div()
        .flex()
        .flex_1()
        .w_full()
        .min_w_0()
        .min_h_0()
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
