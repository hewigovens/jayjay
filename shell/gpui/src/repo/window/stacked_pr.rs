use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use gpui::{AppContext, Context, KeyDownEvent};
use jayjay_core::{Stack, StackedPrResult, SubmitStackLayer, is_valid_bookmark_name};

use super::RepoWindow;
use crate::repo::StackedPrProvider;
use crate::ui::input::LineInput;

static NEXT_GENERATION: AtomicU64 = AtomicU64::new(1);

pub(super) fn next_stacked_pr_generation() -> u64 {
    NEXT_GENERATION.fetch_add(1, Ordering::Relaxed)
}

pub(crate) struct StackedPrState {
    pub(crate) tip_rev: String,
    pub(crate) phase: StackedPrPhase,
    pub(crate) inputs: Vec<LineInput>,
    pub(crate) active_input: Option<usize>,
    pub(crate) generation: u64,
    pub(crate) ai_generation: u64,
    pub(crate) ai_in_flight: bool,
    pub(super) provider: Arc<dyn StackedPrProvider>,
}

pub(crate) enum StackedPrPhase {
    Loading,
    Preview(Stack),
    Submitting(Stack),
    Results(StackedPrResult),
    Error(String),
}

impl StackedPrState {
    fn new(tip_rev: String, provider: Arc<dyn StackedPrProvider>) -> Self {
        Self {
            tip_rev,
            phase: StackedPrPhase::Loading,
            inputs: Vec::new(),
            active_input: None,
            generation: next_stacked_pr_generation(),
            ai_generation: next_stacked_pr_generation(),
            ai_in_flight: false,
            provider,
        }
    }

    pub(crate) fn stack(&self) -> Option<&Stack> {
        match &self.phase {
            StackedPrPhase::Preview(stack) | StackedPrPhase::Submitting(stack) => Some(stack),
            _ => None,
        }
    }

    pub(crate) fn warning(&self, index: usize) -> Option<&'static str> {
        let name = self.inputs.get(index)?.text();
        if name.is_empty() || !is_valid_bookmark_name(name) {
            return Some("Not a valid bookmark name");
        }
        if self
            .inputs
            .iter()
            .enumerate()
            .any(|(other, input)| other != index && input.text() == name)
        {
            return Some("Duplicate bookmark name");
        }
        None
    }

    pub(crate) fn can_submit(&self) -> bool {
        matches!(self.phase, StackedPrPhase::Preview(_))
            && !self.ai_in_flight
            && !self.inputs.is_empty()
            && (0..self.inputs.len()).all(|index| self.warning(index).is_none())
    }

    pub(super) fn payload(&self) -> Option<Vec<SubmitStackLayer>> {
        let stack = self.stack()?;
        Some(
            stack
                .layers
                .iter()
                .zip(&self.inputs)
                .map(|(layer, input)| SubmitStackLayer {
                    change_id: layer.change_id.clone(),
                    bookmark: input.text().to_owned(),
                    title: layer.title.clone(),
                    body: layer.body.clone(),
                })
                .collect(),
        )
    }
}

impl RepoWindow {
    fn stacked_pr_active_input(view: &mut Self) -> Option<&mut LineInput> {
        let state = view.stacked_pr.as_mut()?;
        state.inputs.get_mut(state.active_input?)
    }

    pub(crate) fn activate_stacked_pr_input(&mut self, index: usize, cx: &mut Context<Self>) {
        let Some(state) = self.stacked_pr.as_mut() else {
            return;
        };
        if !matches!(state.phase, StackedPrPhase::Preview(_)) || index >= state.inputs.len() {
            return;
        }
        LineInput::hide_for_owner(self, cx, Self::stacked_pr_active_input);
        self.stacked_pr.as_mut().unwrap().active_input = Some(index);
        LineInput::show_for_owner(self, cx, Self::stacked_pr_active_input);
        cx.notify();
    }

    pub(crate) fn deactivate_stacked_pr_input(&mut self, cx: &mut Context<Self>) {
        if self.stacked_pr.is_none() {
            return;
        }
        LineInput::hide_for_owner(self, cx, Self::stacked_pr_active_input);
        if let Some(state) = self.stacked_pr.as_mut() {
            state.active_input = None;
        }
        cx.notify();
    }

    pub fn set_stacked_pr_provider(&mut self, provider: Arc<dyn StackedPrProvider>) {
        self.stacked_pr_provider = provider;
    }

    pub fn open_stacked_pr(&mut self, tip_rev: String, cx: &mut Context<Self>) {
        let provider = self.stacked_pr_provider.clone();
        self.stacked_pr = Some(StackedPrState::new(tip_rev, provider));
        self.start_stacked_pr_detection(cx);
    }

    pub fn retry_stacked_pr(&mut self, cx: &mut Context<Self>) {
        let Some(state) = self.stacked_pr.as_mut() else {
            return;
        };
        state.generation = next_stacked_pr_generation();
        state.phase = StackedPrPhase::Loading;
        state.inputs.clear();
        state.active_input = None;
        state.ai_generation = next_stacked_pr_generation();
        state.ai_in_flight = false;
        self.start_stacked_pr_detection(cx);
    }

    pub fn complete_stacked_pr(&mut self, cx: &mut Context<Self>) {
        let Some(StackedPrPhase::Results(result)) =
            self.stacked_pr.as_ref().map(|state| &state.phase)
        else {
            return;
        };
        let open_urls = result.open_urls.clone();
        for url in open_urls {
            cx.open_url(&url);
        }
        self.close_stacked_pr(cx);
    }

    fn start_stacked_pr_detection(&mut self, cx: &mut Context<Self>) {
        let Some(state) = self.stacked_pr.as_ref() else {
            return;
        };
        let Some(repo) = self.vm.read(cx).repo.clone() else {
            self.finish_stacked_pr_detection(
                state.generation,
                Err(jayjay_core::Error::internal("repository is not open")),
                cx,
            );
            return;
        };
        let generation = state.generation;
        let tip_rev = state.tip_rev.clone();
        let provider = state.provider.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { provider.detect(&repo, "trunk()", &tip_rev) })
                .await;
            let _ = this.update(cx, |view, cx| {
                view.finish_stacked_pr_detection(generation, result, cx);
            });
        })
        .detach();
        cx.notify();
    }

    fn finish_stacked_pr_detection(
        &mut self,
        generation: u64,
        result: jayjay_core::CoreResult<Stack>,
        cx: &mut Context<Self>,
    ) {
        let Some(state) = self.stacked_pr.as_mut() else {
            return;
        };
        if state.generation != generation {
            return;
        }
        match result {
            Ok(stack) => {
                state.inputs = stack
                    .layers
                    .iter()
                    .map(|layer| LineInput::new(&layer.bookmark))
                    .collect();
                state.phase = StackedPrPhase::Preview(stack);
            }
            Err(error) => state.phase = StackedPrPhase::Error(error.to_string()),
        }
        cx.notify();
    }

    pub fn edit_stacked_pr_name(
        &mut self,
        index: usize,
        name: impl Into<String>,
        cx: &mut Context<Self>,
    ) {
        let Some(state) = self.stacked_pr.as_mut() else {
            return;
        };
        if matches!(state.phase, StackedPrPhase::Preview(_))
            && let Some(input) = state.inputs.get_mut(index)
        {
            input.set_text(name);
            state.active_input = Some(index);
            cx.notify();
        }
    }

    pub(super) fn handle_stacked_pr_key(
        &mut self,
        event: &KeyDownEvent,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(state) = self.stacked_pr.as_mut() else {
            return false;
        };
        if event.keystroke.key == "escape" {
            self.close_stacked_pr(cx);
            return true;
        }
        if event.keystroke.key == "enter" && state.active_input.is_some() {
            self.deactivate_stacked_pr_input(cx);
            return true;
        }
        let Some(index) = state.active_input else {
            return true;
        };
        let Some(input) = state.inputs.get_mut(index) else {
            return true;
        };
        if input.handle_key(event, cx).changed {
            cx.notify();
        }
        true
    }
}
