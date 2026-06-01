mod compare;
mod metadata;

use gpui::{AnyElement, Context, IntoElement, ParentElement, SharedString, Styled, div, px, rgb};
use jayjay_core::{ChangeInfo, DiffStats};

use super::description::description_block;
use crate::app::theme::Theme;
use crate::repo::RepoWindow;
use crate::repo::revset::CompareState;
use compare::compare_banner;
use metadata::metadata_block;

pub(super) struct DetailHeaderState<'a> {
    pub change: &'a ChangeInfo,
    pub stats: Option<&'a DiffStats>,
    pub compare: Option<&'a CompareState>,
    pub file_count: Option<usize>,
    pub recently_copied: Option<&'a SharedString>,
    pub description_height: f32,
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

    div()
        .flex()
        .flex_col()
        .gap(px(10.))
        .px(px(16.))
        .py(px(12.))
        .bg(rgb(t.detail_bg))
        .child(metadata_block(
            change,
            state.stats,
            state.recently_copied,
            t,
            cx,
        ))
        .child(description_block(change, state.description_height, t, cx))
        .into_any_element()
}
