use gpui::{Context, Task, Window};
use jayjay_core::JayResult;

use super::RepoWindow;
use crate::repo::view_model::{OperationStep, Rewrite};
use crate::ui::text_area::{Redo, Undo};
use crate::windows::operation_log::OperationLogView;

impl RepoWindow {
    /// Stepping refreshes the repo, which would close an open sheet or editor and drop what it holds.
    fn can_step_operations(&self) -> bool {
        self.modal.is_none() && !self.has_refresh_sensitive_interaction()
    }

    /// As on macOS, Undo and Redo edit focused text and step jj operations otherwise; a keystroke propagates on to a single-line input's own key handler.
    pub(crate) fn operation_step_action(
        &mut self,
        step: OperationStep,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.find.query.is_some() || self.is_text_input_focused(window, cx) {
            match step {
                OperationStep::Undo => window.dispatch_action(Box::new(Undo), cx),
                OperationStep::Redo => window.dispatch_action(Box::new(Redo), cx),
            }
            cx.propagate();
        } else if self.can_step_operations() {
            self.step_operation(step, cx);
        } else {
            cx.propagate();
        }
    }

    /// Steps from the operation loaded now, so a request that races a newer operation steps nothing.
    pub(crate) fn step_operation(&mut self, step: OperationStep, cx: &mut Context<Self>) {
        let Some(state) = self.vm.read(cx).operation_steps().cloned() else {
            return;
        };
        if step.description(&state).is_none() {
            self.show_toast(step.nothing_to_step(), cx);
            return;
        }
        self.step_from(step, state.operation_id, cx);
    }

    pub(crate) fn undo_toast_operation(&mut self, operation_id: String, cx: &mut Context<Self>) {
        if self.step_from(OperationStep::Undo, operation_id, cx) {
            self.feedback.toast = None;
        }
    }

    fn step_from(
        &mut self,
        step: OperationStep,
        operation_id: String,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.can_step_operations() {
            return false;
        }
        let task = self.vm.update(cx, |vm, cx| {
            vm.step_operation(step, operation_id.clone(), cx)
        });
        Self::spawn_ok(cx, task, move |view, stepped, cx| match stepped {
            Some(operation) => view.show_toast(step.stepped(&operation), cx),
            None => view.open_operation_log_undoing(operation_id, cx),
        });
        true
    }

    fn open_operation_log_undoing(&mut self, operation_id: String, cx: &mut Context<Self>) {
        if let Some(repo) = self.vm.read(cx).repo.clone() {
            OperationLogView::open(repo, cx.entity(), self.vm.clone(), Some(operation_id), cx);
        }
    }

    /// One toast per rewrite, offering Undo for the operation it produced.
    pub(crate) fn notify_rewrite<T: 'static>(
        &mut self,
        task: Task<JayResult<Rewrite<T>>>,
        message: impl FnOnce(&Rewrite<T>) -> String + 'static,
        cx: &mut Context<Self>,
    ) {
        Self::spawn_ok(cx, task, move |view, rewrite, cx| {
            let message = message(&rewrite);
            view.present_toast(message.into(), rewrite.operation_id, cx);
        });
    }
}
