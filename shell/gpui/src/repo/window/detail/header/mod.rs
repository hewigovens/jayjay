mod compare;
mod metadata;

use gpui::prelude::FluentBuilder;
use gpui::{
    AnyElement, Context, IntoElement, ParentElement, Pixels, SharedString, Styled, div, px, rgb,
};
use jayjay_core::compare::CompareState;
use jayjay_core::{BookmarkInfo, ChangeInfo, DiffStats};

use super::DescriptionState;
use super::description::description_block;
use crate::app::theme::Theme;
use crate::repo::RepoWindow;
use crate::repo::window::FocusStop;
use compare::compare_banner;
use metadata::metadata_block;

pub(super) struct DetailHeaderState<'a> {
    pub description: &'a DescriptionState,
    pub change: &'a ChangeInfo,
    pub stats: Option<&'a DiffStats>,
    pub compare: Option<&'a CompareState>,
    pub file_count: Option<usize>,
    pub recently_copied: Option<&'a SharedString>,
    pub bookmarks: &'a [BookmarkInfo],
    pub focused: Option<FocusStop>,
    pub expanded_description_height: Pixels,
    pub detail_width: f32,
}

pub(super) fn detail_header(
    state: DetailHeaderState<'_>,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let change = state.change;
    if let Some(compare) = state.compare {
        return div()
            .flex()
            .flex_col()
            .bg(rgb(t.detail_bg))
            .child(compare_banner(compare, state.file_count, t, cx))
            .into_any_element();
    }

    let expanded = state.description.expanded;
    div()
        .flex()
        .flex_col()
        .gap(px(if expanded { 12. } else { 0. }))
        .min_h_0()
        .pb(px(if expanded { 18. } else { 0. }))
        .bg(rgb(t.detail_bg))
        // Collapsed, the byline row draws the rule inside its fixed height so it shares a pixel row with the file rows' separators; an auto-height box would add the border below.
        .when(expanded, |el| el.border_b_1().border_color(rgb(t.border)))
        .child(description_block(
            change,
            state.description,
            state.focused,
            state.expanded_description_height,
            t,
            cx,
        ))
        .child(metadata_block(&state, expanded, t, cx))
        .into_any_element()
}
