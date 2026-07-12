use std::sync::Arc;

use gpui::{
    AnyElement, AppContext, ClickEvent, Context, InteractiveElement, IntoElement, SharedString,
    StatefulInteractiveElement, Styled,
};
use jayjay_core::{Repo, commit_message};

use super::RepoWindow;
use crate::app::theme::Theme;
use crate::ui::icons::glyph;
use crate::ui::primitives::{icon_button, inert_icon_button, text_tooltip};

/// Seam between the commit box and the AI CLIs so component tests can mock generation.
pub trait CommitMessageProvider: Send + Sync {
    /// Display name of the first available CLI ("Codex", then "Claude"), or `None` when neither binary resolves.
    fn detect(&self) -> Option<String>;
    /// Blocking generation from a working-copy diff summary; always runs on the background executor.
    fn generate(&self, diff_summary: &str) -> Result<String, String>;
    /// Generate a bookmark-name slug from a change description.
    fn generate_branch_name(&self, description: &str) -> Result<String, String> {
        self.generate(description)
    }
}

/// Real provider chain: codex first, then claude (Apple Intelligence stays SwiftUI-only).
struct CliCommitMessageProvider;

impl CommitMessageProvider for CliCommitMessageProvider {
    fn detect(&self) -> Option<String> {
        let name = jayjay_core::detect_ai_provider();
        (!name.is_empty()).then_some(name)
    }

    fn generate(&self, diff_summary: &str) -> Result<String, String> {
        jayjay_core::generate_commit_message_cli(diff_summary).ok_or_else(|| {
            "AI generation failed; check that codex or claude works in a terminal".to_owned()
        })
    }

    fn generate_branch_name(&self, description: &str) -> Result<String, String> {
        jayjay_core::generate_branch_name_cli(description).ok_or_else(|| {
            "AI naming failed; check that codex or claude works in a terminal".to_owned()
        })
    }
}

pub(crate) struct CommitAiState {
    pub(super) provider: Arc<dyn CommitMessageProvider>,
    /// `None` until detection lands or when no CLI is installed; gates the generate button.
    pub(super) provider_name: Option<String>,
    /// Monotonic guards so only the newest detection/generation may write back.
    pub(super) detect_gen: u64,
    pub(super) generation: u64,
    pub(super) in_flight: bool,
}

impl Default for CommitAiState {
    fn default() -> Self {
        Self {
            provider: Arc::new(CliCommitMessageProvider),
            provider_name: None,
            detect_gen: 0,
            generation: 0,
            in_flight: false,
        }
    }
}

enum GenerateOutcome {
    Message(String),
    EmptyDiff,
    Failed(String),
}

/// Blocking core work: snapshot-and-summarize the working-copy diff, then run the provider chain.
fn run_generation(provider: &dyn CommitMessageProvider, repo: &Repo) -> GenerateOutcome {
    let summary = match repo.diff_summary() {
        Ok(summary) => summary,
        Err(error) => {
            return GenerateOutcome::Failed(format!("Could not read working-copy diff: {error}"));
        }
    };
    if summary.trim().is_empty() {
        return GenerateOutcome::EmptyDiff;
    }
    match provider.generate(&summary) {
        Ok(message) if !message.trim().is_empty() => GenerateOutcome::Message(message),
        Ok(_) => GenerateOutcome::Failed("AI returned an empty message".to_owned()),
        Err(error) => GenerateOutcome::Failed(error),
    }
}

impl RepoWindow {
    /// Replace the provider chain (tests inject mocks here) and re-run detection against it.
    pub fn set_commit_message_provider(
        &mut self,
        provider: Arc<dyn CommitMessageProvider>,
        cx: &mut Context<Self>,
    ) {
        self.commit_ai.provider = provider;
        self.commit_ai.provider_name = None;
        self.redetect_commit_ai_provider(cx);
    }

    pub fn commit_ai_provider_name(&self) -> Option<String> {
        self.commit_ai.provider_name.clone()
    }

    pub fn is_generating_commit_message(&self) -> bool {
        self.commit_ai.in_flight
    }

    /// Detection may resolve the login-shell PATH (slow on first call), so it runs off the main thread.
    pub(super) fn redetect_commit_ai_provider(&mut self, cx: &mut Context<Self>) {
        self.commit_ai.detect_gen += 1;
        let detect_gen = self.commit_ai.detect_gen;
        let provider = self.commit_ai.provider.clone();
        cx.spawn(async move |this, cx| {
            let name = cx.background_spawn(async move { provider.detect() }).await;
            let _ = this.update(cx, |view, cx| {
                // A newer provider was installed while this detection ran; drop the stale name.
                if view.commit_ai.detect_gen == detect_gen {
                    view.commit_ai.provider_name = name;
                    cx.notify();
                }
            });
        })
        .detach();
    }

    /// Invalidate an in-flight generation whose reply would describe a working copy that no longer exists (e.g. after commit clears the inputs back to their trigger-time snapshot).
    pub(super) fn cancel_pending_commit_message_generation(&mut self) {
        if self.commit_ai.in_flight {
            self.commit_ai.generation += 1;
            self.commit_ai.in_flight = false;
        }
    }

    /// Generate a commit message from the working-copy diff and fill the commit box inputs.
    pub fn generate_commit_message(&mut self, cx: &mut Context<Self>) {
        if self.commit_ai.in_flight {
            return;
        }
        if self.commit_ai.provider_name.is_none() {
            self.show_toast(
                "No AI CLI found. Install codex or claude to generate messages.",
                cx,
            );
            return;
        }
        let Some(repo) = self.vm.read(cx).repo.clone() else {
            self.show_toast("Repository is not open", cx);
            return;
        };
        let snapshot = (
            self.summary_input.read(cx).text(),
            self.description_input.read(cx).text(),
        );
        self.commit_ai.generation += 1;
        let generation = self.commit_ai.generation;
        self.commit_ai.in_flight = true;
        cx.notify();
        let provider = self.commit_ai.provider.clone();
        cx.spawn(async move |this, cx| {
            let outcome = cx
                .background_spawn(async move { run_generation(provider.as_ref(), &repo) })
                .await;
            let _ = this.update(cx, move |view, cx| {
                view.finish_generation(generation, snapshot, outcome, cx);
            });
        })
        .detach();
    }

    fn finish_generation(
        &mut self,
        generation: u64,
        snapshot: (String, String),
        outcome: GenerateOutcome,
        cx: &mut Context<Self>,
    ) {
        // A newer request owns the in-flight flag; this reply is stale either way.
        if generation != self.commit_ai.generation {
            return;
        }
        self.commit_ai.in_flight = false;
        cx.notify();
        match outcome {
            GenerateOutcome::Message(message) => {
                let untouched = self.summary_input.read(cx).text() == snapshot.0
                    && self.description_input.read(cx).text() == snapshot.1;
                // The user typed while the AI ran; their words win and the reply is dropped.
                if !untouched {
                    return;
                }
                let summary = commit_message::summary(&message);
                let body = commit_message::body(&message);
                self.summary_input
                    .update(cx, |input, cx| input.set_text(summary, cx));
                self.description_input
                    .update(cx, |input, cx| input.set_text(body, cx));
            }
            GenerateOutcome::EmptyDiff => {
                self.show_toast("Working copy has no changes to describe", cx);
            }
            GenerateOutcome::Failed(message) => self.show_toast(message, cx),
        }
    }
}

/// Sparkle button beside the commit controls; dimmed while generating or when no CLI is installed.
pub(super) fn generate_button(
    view: &RepoWindow,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let in_flight = view.commit_ai.in_flight;
    let provider_name = view.commit_ai.provider_name.clone();
    let enabled = !in_flight && provider_name.is_some();
    let tooltip: SharedString = if in_flight {
        "Generating…".into()
    } else {
        match &provider_name {
            Some(name) => format!("Generate with {name}").into(),
            None => "No AI available".into(),
        }
    };
    let base = if enabled {
        icon_button(
            "commit-ai-generate",
            glyph::SPARKLE,
            14.,
            28.,
            28.,
            t.fg_dim,
            t,
        )
        .on_click(cx.listener(|view, _: &ClickEvent, _w, cx| view.generate_commit_message(cx)))
    } else {
        inert_icon_button(
            "commit-ai-generate",
            glyph::SPARKLE,
            14.,
            28.,
            28.,
            t.fg_faint,
        )
        .opacity(0.5)
    };
    base.debug_selector(|| "commit-ai-generate".to_owned())
        .tooltip(text_tooltip(tooltip))
        .into_any_element()
}
