use gpui::Context;

use super::stacked_pr::{StackedPrPhase, next_stacked_pr_generation};
use super::{RepoModal, RepoWindow};

impl RepoWindow {
    pub fn submit_stacked_pr(&mut self, cx: &mut Context<Self>) {
        let Some(RepoModal::StackedPr(state)) = &mut self.modal else {
            return;
        };
        if !state.can_submit() {
            return;
        }
        let Some(payload) = state.payload() else {
            return;
        };
        let stack = match &state.phase {
            StackedPrPhase::Preview(stack) => stack.clone(),
            _ => return,
        };
        state.generation = next_stacked_pr_generation();
        let generation = state.generation;
        let provider = state.provider.clone();
        state.active_input = None;
        state.phase = StackedPrPhase::Submitting(stack);
        let task = self
            .vm
            .update(cx, |vm, cx| vm.submit_stack(provider, payload, cx));
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |view, cx| {
                let Some(RepoModal::StackedPr(state)) = &mut view.modal else {
                    return;
                };
                if state.generation != generation {
                    return;
                }
                state.phase = match result {
                    Ok(result) => StackedPrPhase::Results(result),
                    Err(error) => StackedPrPhase::Error(crate::app::error_text(error).to_string()),
                };
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }

    pub fn close_stacked_pr(&mut self, cx: &mut Context<Self>) {
        let submitting = self
            .stacked_pr()
            .is_some_and(|state| matches!(state.phase, StackedPrPhase::Submitting(_)));
        if submitting {
            return;
        }
        self.close_modal(|modal| matches!(modal, RepoModal::StackedPr(_)), cx);
    }
}
