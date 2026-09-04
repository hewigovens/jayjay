use std::sync::Arc;

use gpui::{AnyElement, Context, InteractiveElement, IntoElement, MouseButton, MouseDownEvent};

use super::state::ReviewDisplayState;
use crate::app::theme::Theme;
use crate::diff::line::{review_stripe, review_stripe_spacer};
use crate::repo::window::RepoWindow;

/// Gutter cell for one rendered row: a clickable stripe when the row belongs to a change group, otherwise a same-width spacer.
pub(super) fn review_stripe_cell(
    id: (&'static str, usize),
    review: Option<&Arc<ReviewDisplayState>>,
    group: Option<u32>,
    theme: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let Some((review, group)) = review.zip(group) else {
        return review_stripe_spacer().into_any_element();
    };
    let review = review.clone();
    review_stripe(id, review.state_for_group(group), theme)
        .debug_selector(move || format!("review-hunk-{group}"))
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |view, _: &MouseDownEvent, _, cx| {
                cx.stop_propagation();
                view.toggle_review_hunk(&review.path, &review.identity, group, cx);
            }),
        )
        .into_any_element()
}
