use std::sync::Arc;

use gpui::{App, AppContext, Context, SharedString};
use jayjay_core::diff::{
    ContextExpansion, ContextExpansionFinish, ContextExpansionOutcome, ContextExpansionRequest,
    ContextExpansionSession, ContextExpansionSource,
};

use super::RepoWindow;

#[derive(Default)]
pub(crate) struct ContextExpansionState {
    session: ContextExpansionSession,
    error: Option<SharedString>,
}

impl RepoWindow {
    pub fn expand_context(
        &mut self,
        region_id: u32,
        expansion: ContextExpansion,
        cx: &mut Context<Self>,
    ) {
        self.request_context_expansion(
            ContextExpansionRequest::Region {
                region_id,
                expansion,
            },
            cx,
        );
    }

    fn request_context_expansion(
        &mut self,
        request: ContextExpansionRequest,
        cx: &mut Context<Self>,
    ) {
        let (diff, old_content, new_content) = {
            let vm = self.vm.read(cx);
            let (Some(diff), Some(old_content), Some(new_content)) = (
                vm.current_diff.clone(),
                vm.current_diff_old_content.clone(),
                vm.current_diff_new_content.clone(),
            ) else {
                return;
            };
            (diff, old_content, new_content)
        };
        let basis = self.context_expansion_basis(cx);
        let Some(attempt) = self.diff.context_expansion.session.begin(&basis, request) else {
            return;
        };
        let needs_source = attempt.needs_source();

        cx.spawn(async move |this, cx| {
            let outcome = cx
                .background_spawn(async move {
                    // Built off the UI thread: line indexing and the diff clone are exactly the large-file cost this action targets.
                    let source = needs_source.then(|| ContextExpansionSource {
                        diff: diff.as_ref().clone(),
                        old_content,
                        new_content,
                    });
                    attempt.run(source)
                })
                .await;
            let _ = this.update(cx, move |view, cx| {
                view.finish_context_expansion(outcome, cx);
            });
        })
        .detach();
    }

    fn finish_context_expansion(
        &mut self,
        outcome: ContextExpansionOutcome,
        cx: &mut Context<Self>,
    ) {
        let basis = self.context_expansion_basis(cx);
        match self.diff.context_expansion.session.finish(&basis, outcome) {
            ContextExpansionFinish::Discarded => {}
            ContextExpansionFinish::Applied { diff, next, .. } => {
                self.diff.selection = None;
                self.diff.gutter_selection = None;
                self.diff.context_expansion.error = None;
                self.vm.update(cx, |vm, cx| {
                    vm.current_diff = Some(Arc::new(diff));
                    cx.notify();
                });
                // Stored matches are display-row indices into the replaced diff.
                self.recompute_find_matches(cx);
                if let Some(next) = next {
                    self.request_context_expansion(next, cx);
                }
            }
            ContextExpansionFinish::Failed { message } => {
                self.diff.context_expansion.error = Some(message.into());
                cx.notify();
            }
        }
    }

    /// Every diff install bumps `diff_gen`, so this pair identifies the render an expansion was started against.
    fn context_expansion_basis(&self, cx: &App) -> String {
        let vm = self.vm.read(cx);
        format!("{}:{:?}", vm.loading.diff_gen, vm.selected_file_ix)
    }

    pub(crate) fn context_expansion_error(&self) -> Option<SharedString> {
        self.diff.context_expansion.error.clone()
    }

    pub(crate) fn dismiss_context_expansion_error(&mut self, cx: &mut Context<Self>) {
        self.diff.context_expansion.error = None;
        cx.notify();
    }

    pub(crate) fn reset_context_expansion(&mut self) {
        self.diff.context_expansion.session.reset();
        self.diff.context_expansion.error = None;
    }

    pub(crate) fn reset_context_expansion_if_basis_changed(&mut self, cx: &App) {
        if self
            .diff
            .context_expansion
            .session
            .is_stale(&self.context_expansion_basis(cx))
        {
            self.reset_context_expansion();
        }
    }
}
