use gpui::{AppContext, Context};
use jayjay_core::check_jj_environment;

use super::state::{JjCheckState, OnboardingPage};
use crate::app::config;
use crate::repo::window::RepoWindow;

impl RepoWindow {
    pub(crate) fn check_jj_for_onboarding(&mut self, cx: &mut Context<Self>) {
        if let Some(onboarding) = self.onboarding.as_mut() {
            onboarding.jj = JjCheckState::Checking;
        }
        cx.spawn(async move |this, cx| {
            let status = cx.background_spawn(async { check_jj_environment() }).await;
            let _ = this.update(cx, move |view, cx| {
                if let Some(onboarding) = view.onboarding.as_mut() {
                    onboarding.jj = JjCheckState::Loaded(status);
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(crate) fn set_onboarding_page(&mut self, page: OnboardingPage, cx: &mut Context<Self>) {
        if let Some(onboarding) = self.onboarding.as_mut() {
            onboarding.page = page;
        }
        cx.notify();
    }

    pub(crate) fn finish_onboarding(&mut self, cx: &mut Context<Self>) {
        config::update(cx, |cfg| cfg.onboarding.completed = true);
        self.onboarding = None;
        let vm = self.vm.clone();
        vm.update(cx, |vm, cx| vm.open_async(cx));
        cx.notify();
    }
}
