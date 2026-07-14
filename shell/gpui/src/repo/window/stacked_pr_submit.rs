use gpui::Context;

use super::RepoWindow;
use super::stacked_pr::{StackedPrPhase, next_stacked_pr_generation};

impl RepoWindow {
    pub fn submit_stacked_pr(&mut self, cx: &mut Context<Self>) {
        let Some(state) = self.stacked_pr.as_mut() else {
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
                let Some(state) = view.stacked_pr.as_mut() else {
                    return;
                };
                if state.generation != generation {
                    return;
                }
                state.phase = match result {
                    Ok(result) => StackedPrPhase::Results(result),
                    Err(error) => StackedPrPhase::Error(error.to_string()),
                };
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }

    pub fn close_stacked_pr(&mut self, cx: &mut Context<Self>) {
        let submitting = self
            .stacked_pr
            .as_ref()
            .is_some_and(|state| matches!(state.phase, StackedPrPhase::Submitting(_)));
        if submitting {
            return;
        }
        if self.stacked_pr.take().is_some() {
            cx.notify();
        }
    }
}
