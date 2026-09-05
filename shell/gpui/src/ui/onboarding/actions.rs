use gpui::{AppContext, Context};
use jayjay_core::check_jj_environment;

use super::state::{JjCheckState, OnboardingPage};
use super::{OnboardingCompleted, OnboardingView};
use crate::app::config;

impl OnboardingView {
    pub(super) fn check_jj(&mut self, cx: &mut Context<Self>) {
        self.jj = JjCheckState::Checking;
        cx.spawn(async move |this, cx| {
            let status = cx.background_spawn(async { check_jj_environment() }).await;
            let _ = this.update(cx, move |view, cx| {
                view.jj = JjCheckState::Loaded(status);
                cx.notify();
            });
        })
        .detach();
    }

    pub(super) fn set_page(&mut self, page: OnboardingPage, cx: &mut Context<Self>) {
        self.page = page;
        cx.notify();
    }

    pub(super) fn finish(&mut self, cx: &mut Context<Self>) {
        config::update(cx, |cfg| cfg.onboarding.completed = true);
        cx.emit(OnboardingCompleted);
    }
}
