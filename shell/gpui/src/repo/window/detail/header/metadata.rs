mod byline;
mod cells;
mod grid;

use gpui::{AnyElement, Context, IntoElement, ParentElement, Styled, div, px};

use super::DetailHeaderState;
use crate::app::theme::Theme;
use crate::diff::DETAIL_INSET;
use crate::repo::RepoWindow;
use byline::byline;
use grid::grid;

pub(super) fn metadata_block(
    state: &DetailHeaderState<'_>,
    expanded: bool,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    if expanded {
        div()
            .flex()
            .flex_row()
            .items_end()
            .pr(px(DETAIL_INSET))
            .child(div().flex_1().min_w_0().child(grid(
                state.change,
                state.stats,
                state.recently_copied,
                state.bookmarks,
                t,
                cx,
            )))
            .children(state.diff_actions.render(false, state.focused, t, cx))
            .into_any_element()
    } else {
        byline(state, t, cx)
    }
}
