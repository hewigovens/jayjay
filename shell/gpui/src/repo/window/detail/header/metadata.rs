mod byline;
mod cells;
mod grid;

use gpui::{AnyElement, Context};

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
        grid(
            state.change,
            state.stats,
            state.recently_copied,
            state.bookmarks,
            t,
            cx,
        )
    } else {
        byline(
            state.change,
            state.stats,
            state.recently_copied,
            state.bookmarks,
            state.detail_width - 2. * DETAIL_INSET,
            t,
            cx,
        )
    }
}
