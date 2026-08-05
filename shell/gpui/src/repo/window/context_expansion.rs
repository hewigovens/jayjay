use std::sync::Arc;

use gpui::{App, AppContext, Context, SharedString};
use jayjay_core::diff::{ContextExpansion, ExpandableDiff, FileDiff};

use super::RepoWindow;

#[derive(Default)]
pub(crate) struct ContextExpansionState {
    generation: u64,
    session: Option<ContextExpansionSession>,
    error: Option<SharedString>,
}

struct ContextExpansionSession {
    displayed_diff: Arc<FileDiff>,
    document: Option<ExpandableDiff>,
    pending: Option<(u32, ContextExpansion)>,
    in_flight: bool,
}

struct ContextExpansionCompletion {
    generation: u64,
    diff_generation: u64,
    selected_file_ix: Option<usize>,
    displayed_diff: Arc<FileDiff>,
    document: ExpandableDiff,
    result:
        Result<jayjay_core::diff::ContextExpansionResult, jayjay_core::diff::ContextExpansionError>,
}

impl ContextExpansionState {
    fn reset(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.session = None;
        self.error = None;
    }
}

impl RepoWindow {
    pub fn expand_context(
        &mut self,
        region_id: u32,
        expansion: ContextExpansion,
        cx: &mut Context<Self>,
    ) {
        self.reset_context_expansion_if_basis_changed(cx);
        let (current_diff, old_content, new_content, diff_generation, selected_file_ix) = {
            let vm = self.vm.read(cx);
            let (Some(diff), Some(old_content), Some(new_content)) = (
                vm.current_diff.clone(),
                vm.current_diff_old_content.clone(),
                vm.current_diff_new_content.clone(),
            ) else {
                return;
            };
            if !diff.lines.iter().any(|line| {
                line.context_region
                    .is_some_and(|region| region.id == region_id)
            }) {
                return;
            }
            (
                diff,
                old_content,
                new_content,
                vm.loading.diff_gen,
                vm.selected_file_ix,
            )
        };

        if self.diff.context_expansion.session.is_none() {
            self.diff.context_expansion.session = Some(ContextExpansionSession {
                displayed_diff: current_diff.clone(),
                document: None,
                pending: None,
                in_flight: false,
            });
        }

        let Some(session) = self.diff.context_expansion.session.as_mut() else {
            return;
        };
        if session.in_flight {
            session.pending = Some((region_id, expansion));
            return;
        }
        session.in_flight = true;
        let document = session.document.take();
        let displayed_diff = session.displayed_diff.clone();
        self.diff.context_expansion.generation =
            self.diff.context_expansion.generation.wrapping_add(1);
        let generation = self.diff.context_expansion.generation;

        cx.spawn(async move |this, cx| {
            let (document, result) = cx
                .background_spawn(async move {
                    // Built off the UI thread: line indexing and the diff clone are exactly the large-file cost this action targets.
                    let mut document = document.unwrap_or_else(|| {
                        ExpandableDiff::from_shared(
                            current_diff.as_ref().clone(),
                            old_content,
                            new_content,
                        )
                    });
                    let result = document.expand(region_id, expansion);
                    (document, result)
                })
                .await;
            let _ = this.update(cx, move |view, cx| {
                view.finish_context_expansion(
                    ContextExpansionCompletion {
                        generation,
                        diff_generation,
                        selected_file_ix,
                        displayed_diff,
                        document,
                        result,
                    },
                    cx,
                );
            });
        })
        .detach();
    }

    fn finish_context_expansion(
        &mut self,
        completion: ContextExpansionCompletion,
        cx: &mut Context<Self>,
    ) {
        let ContextExpansionCompletion {
            generation,
            diff_generation,
            selected_file_ix,
            displayed_diff,
            document,
            result,
        } = completion;
        if self.diff.context_expansion.generation != generation {
            return;
        }
        let basis_is_current = {
            let vm = self.vm.read(cx);
            vm.loading.diff_gen == diff_generation
                && vm.selected_file_ix == selected_file_ix
                && vm
                    .current_diff
                    .as_ref()
                    .is_some_and(|current| Arc::ptr_eq(current, &displayed_diff))
        };
        if !basis_is_current {
            self.reset_context_expansion();
            return;
        }

        let (pending, installed) = {
            let Some(session) = self.diff.context_expansion.session.as_mut() else {
                return;
            };
            session.in_flight = false;
            match result {
                Ok(result) => {
                    let expanded = Arc::new(result.diff);
                    session.displayed_diff = expanded.clone();
                    session.document = Some(document);
                    self.diff.selection = None;
                    self.diff.gutter_selection = None;
                    self.vm.update(cx, |vm, cx| {
                        vm.current_diff = Some(expanded);
                        cx.notify();
                    });
                    self.diff.context_expansion.error = None;
                    (session.pending.take(), true)
                }
                Err(error) => {
                    session.document = Some(document);
                    session.pending = None;
                    self.diff.context_expansion.error = Some(format!("{error}").into());
                    cx.notify();
                    (None, false)
                }
            }
        };
        if installed {
            // Stored matches are display-row indices into the replaced diff.
            self.recompute_find_matches(cx);
        }
        if let Some((region_id, expansion)) = pending {
            self.expand_context(region_id, expansion, cx);
        }
    }

    pub fn context_expansion_error(&self) -> Option<SharedString> {
        self.diff.context_expansion.error.clone()
    }

    pub fn dismiss_context_expansion_error(&mut self, cx: &mut Context<Self>) {
        self.diff.context_expansion.error = None;
        cx.notify();
    }

    pub(crate) fn reset_context_expansion(&mut self) {
        self.diff.context_expansion.reset();
    }

    pub(crate) fn reset_context_expansion_if_basis_changed(&mut self, cx: &App) {
        let Some(session) = self.diff.context_expansion.session.as_ref() else {
            return;
        };
        let is_current = self
            .vm
            .read(cx)
            .current_diff
            .as_ref()
            .is_some_and(|current| Arc::ptr_eq(current, &session.displayed_diff));
        if !is_current {
            self.reset_context_expansion();
        }
    }
}
