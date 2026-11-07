use std::collections::HashSet;

use gpui::{
    App, AppContext, Context, Entity, FocusHandle, Focusable, ScrollHandle, SharedString, Task,
    Window,
};
use jayjay_core::converge::{ConvergeItem, ConvergeReview};
use jayjay_core::{ConvergeOutcome, ConvergePlan, JayError, JayResult};

use super::contents::ConvergeContents;
use crate::repo::window::RepoWindow;
use crate::ui::text_area::TextArea;

#[derive(Default)]
pub(crate) struct ConvergeState {
    pub(super) sheet: Option<ConvergeSheet>,
    pending_editor: Option<(String, String)>,
}

impl ConvergeState {
    pub(crate) fn is_open(&self) -> bool {
        self.sheet.is_some()
    }
}

pub(super) enum ConvergePhase {
    Loading,
    Ready,
    Applying,
    Failed(SharedString),
}

pub(crate) struct ConvergeSheet {
    revision: String,
    preview: Option<Task<()>>,
    pub(super) review: Option<ConvergeReview>,
    pub(super) phase: ConvergePhase,
    pub(super) was_stale: bool,
    pub(super) description: Option<Entity<TextArea>>,
    pub(super) contents: ConvergeContents,
    pub(super) expanded: HashSet<&'static str>,
    pub(super) scroll: ScrollHandle,
    focus_pending: bool,
}

impl ConvergeSheet {
    pub(super) fn is_interactive(&self) -> bool {
        matches!(self.phase, ConvergePhase::Ready)
    }

    pub(super) fn can_apply(&self) -> bool {
        self.is_interactive() && self.review.as_ref().is_some_and(ConvergeReview::can_apply)
    }

    pub(super) fn take_focus(&mut self, root: &FocusHandle, window: &mut Window, cx: &mut App) {
        if !std::mem::take(&mut self.focus_pending) {
            return;
        }
        let editing = self.review.as_ref().and_then(ConvergeReview::focused)
            == Some(ConvergeItem::Description);
        match self.description.as_ref().filter(|_| editing) {
            Some(editor) => window.focus(&editor.read(cx).focus_handle(cx), cx),
            None => window.focus(root, cx),
        }
    }
}

impl RepoWindow {
    pub fn open_converge(&mut self, revision: String, cx: &mut Context<Self>) {
        self.converge.sheet = Some(ConvergeSheet {
            revision,
            preview: None,
            review: None,
            phase: ConvergePhase::Loading,
            was_stale: false,
            description: None,
            contents: ConvergeContents::default(),
            expanded: HashSet::new(),
            scroll: ScrollHandle::new(),
            focus_pending: true,
        });
        self.load_converge_preview(cx);
    }

    pub(crate) fn close_converge(&mut self, cx: &mut Context<Self>) {
        if let Some(sheet) = &self.converge.sheet
            && !matches!(sheet.phase, ConvergePhase::Applying)
        {
            self.converge.sheet = None;
            cx.notify();
        }
    }

    pub(super) fn load_converge_preview(&mut self, cx: &mut Context<Self>) {
        let Some(sheet) = self.converge.sheet.as_mut() else {
            return;
        };
        sheet.phase = ConvergePhase::Loading;
        let revision = sheet.revision.clone();
        let Some(repo) = self.vm.read(cx).repo.clone() else {
            let error = JayError::internal(jayjay_i18n::converge::unavailable());
            sheet.phase = ConvergePhase::Failed(crate::app::error_text(error));
            cx.notify();
            return;
        };
        sheet.preview = Some(cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { repo.converge_preview(&revision) })
                .await;
            let _ = this.update(cx, |view, cx| view.finish_converge_preview(result, cx));
        }));
        cx.notify();
    }

    fn finish_converge_preview(&mut self, result: JayResult<ConvergePlan>, cx: &mut Context<Self>) {
        let Some(sheet) = self.converge.sheet.as_mut() else {
            return;
        };
        match result {
            Ok(plan) => {
                sheet.revision = plan.change_id.clone();
                sheet.contents = ConvergeContents::comparing(&plan);
                sheet.review = Some(ConvergeReview::new(plan));
                sheet.phase = ConvergePhase::Ready;
                sheet.description = None;
                sheet.focus_pending = true;
                self.load_converge_contents(cx);
            }
            Err(JayError::ConvergeNotDivergent { .. }) => {
                sheet.review = None;
                sheet.phase = ConvergePhase::Failed(jayjay_i18n::converge::not_divergent().into());
            }
            Err(error) => sheet.phase = ConvergePhase::Failed(crate::app::error_text(error)),
        }
        cx.notify();
    }

    pub(super) fn apply_converge(&mut self, cx: &mut Context<Self>) {
        let Some(sheet) = self
            .converge
            .sheet
            .as_mut()
            .filter(|sheet| sheet.can_apply())
        else {
            return;
        };
        let text = sheet
            .description
            .as_ref()
            .map(|editor| editor.read(cx).text());
        let Some(review) = sheet.review.as_mut() else {
            return;
        };
        if let Some(text) = text {
            review.set_custom_description(text);
        }
        let plan = review.plan().clone();
        let choices = review.choices();
        sheet.phase = ConvergePhase::Applying;
        let task = self
            .vm
            .update(cx, |vm, cx| vm.converge_change(plan, choices, cx));
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |view, cx| {
                view.finish_converge_apply(result.map(|(outcome, _)| outcome), cx);
            });
        })
        .detach();
        cx.notify();
    }

    fn finish_converge_apply(
        &mut self,
        result: JayResult<ConvergeOutcome>,
        cx: &mut Context<Self>,
    ) {
        let Some(sheet) = self
            .converge
            .sheet
            .as_mut()
            .filter(|sheet| matches!(sheet.phase, ConvergePhase::Applying))
        else {
            return;
        };
        match result {
            Ok(outcome) => {
                self.converge.sheet = None;
                self.converge.pending_editor = outcome
                    .conflicted_paths
                    .into_iter()
                    .next()
                    .map(|path| (outcome.successor_commit_id, path));
                self.open_pending_conflict_editor(cx);
                cx.notify();
            }
            Err(error) => {
                // The sheet reports the failure itself; the window-level error overlay would cover it.
                self.vm.update(cx, |vm, _| vm.clear_error());
                if matches!(error, JayError::ConvergePlanStale { .. }) {
                    sheet.was_stale = true;
                    self.load_converge_preview(cx);
                } else {
                    sheet.phase = ConvergePhase::Failed(crate::app::error_text(error));
                    cx.notify();
                }
            }
        }
    }

    pub(super) fn update_converge_review(
        &mut self,
        cx: &mut Context<Self>,
        update: impl FnOnce(&mut ConvergeReview),
    ) {
        let Some(sheet) = self
            .converge
            .sheet
            .as_mut()
            .filter(|sheet| sheet.is_interactive())
        else {
            return;
        };
        let Some(review) = sheet.review.as_mut() else {
            return;
        };
        let focused = review.focused();
        update(review);
        match (review.custom_description(), sheet.description.is_some()) {
            (Some(text), false) => {
                let text = text.to_owned();
                sheet.description = Some(cx.new(|cx| {
                    TextArea::new(text, jayjay_i18n::converge::description(), true, 80., cx)
                }));
                sheet.focus_pending = true;
            }
            (None, true) => {
                sheet.description = None;
                sheet.focus_pending = true;
            }
            _ => {}
        }
        if review.focused() != focused {
            sheet.focus_pending = true;
        }
        if review.focused() != focused
            && let Some(item) = review.focused()
            && let Some(ix) = review.card_items().iter().position(|card| *card == item)
        {
            sheet
                .scroll
                .scroll_to_item(super::render::FIRST_CARD_CHILD + ix);
        }
        cx.notify();
    }

    /// Waits for the post-converge refresh and file load, then opens the merge editor only if they landed on the successor's conflicted file.
    pub(crate) fn open_pending_conflict_editor(&mut self, cx: &mut Context<Self>) {
        let Some((commit_id, path)) = self.converge.pending_editor.as_ref() else {
            return;
        };
        let vm = self.vm.read(cx);
        if vm.loading.refreshing || vm.loading.files || vm.loading.diff {
            return;
        }
        let ready = vm.compare.is_none()
            && vm
                .selected_change()
                .is_some_and(|change| &change.commit_id.id == commit_id)
            && vm.selected_hunk().is_some_and(|hunk| &hunk.path == path);
        self.converge.pending_editor = None;
        if ready {
            self.enter_selected_conflict_editor(cx);
        }
    }

    pub(super) fn toggle_converge_section(&mut self, id: &'static str, cx: &mut Context<Self>) {
        if let Some(sheet) = self.converge.sheet.as_mut() {
            if !sheet.expanded.remove(id) {
                sheet.expanded.insert(id);
            }
            cx.notify();
        }
    }
}
