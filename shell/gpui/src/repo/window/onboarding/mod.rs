mod actions;
mod pages;
mod state;
mod widgets;

use gpui::{AnyElement, Context};

use crate::app::theme::Theme;
use crate::repo::window::RepoWindow;

pub(crate) use state::OnboardingState;

pub(super) fn onboarding_pane(
    state: &OnboardingState,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    pages::onboarding_pane(state, t, cx)
}
