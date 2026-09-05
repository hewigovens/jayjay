use gpui::{Context, EventEmitter, IntoElement, Render, Window};

use super::pages::onboarding_pane;
use super::state::{JjCheckState, OnboardingPage};
use crate::app::theme::theme_for_window;
use crate::ui::logo::Logo;

pub(crate) struct OnboardingView {
    pub(super) page: OnboardingPage,
    pub(super) jj: JjCheckState,
    pub(super) logo: Logo,
}

pub(crate) struct OnboardingCompleted;

impl EventEmitter<OnboardingCompleted> for OnboardingView {}

impl OnboardingView {
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        let mut view = Self {
            page: OnboardingPage::Welcome,
            jj: JjCheckState::Checking,
            logo: Logo::load(cx),
        };
        view.check_jj(cx);
        view
    }
}

impl Render for OnboardingView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = theme_for_window(window, cx).clone();
        onboarding_pane(self, &theme, cx)
    }
}
