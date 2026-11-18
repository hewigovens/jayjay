use gpui::{Context, Task};
use jayjay_core::{JayResult, UndoState, capture_mutation_operation_id};

use super::tasks::RepoTaskKind;
use super::{OperationStep, RepoViewModel};

impl RepoViewModel {
    /// What Undo and Redo would step, or `None` while an action or refresh is still landing.
    pub(crate) fn operation_steps(&self) -> Option<&UndoState> {
        (!self.loading.refreshing).then_some(&self.undo_state)
    }

    pub(crate) fn step_operation(
        &mut self,
        step: OperationStep,
        operation_id: String,
        cx: &mut Context<Self>,
    ) -> Task<JayResult<Option<String>>> {
        self.run_repo_task(
            cx,
            RepoTaskKind::OperationStep,
            move |repo| match step {
                OperationStep::Undo => repo.undo(&operation_id),
                OperationStep::Redo => repo.redo(&operation_id),
            },
            |vm, _, cx| vm.refresh_selecting_revision(None, cx),
        )
    }

    /// `None` while another restore is in flight.
    pub(crate) fn restore_operation(
        &mut self,
        operation_id: String,
        cx: &mut Context<Self>,
    ) -> Option<Task<JayResult<Option<String>>>> {
        if self.restoring_operation {
            return None;
        }
        self.restoring_operation = true;
        let restore = self.repo_result_task(
            cx,
            move |repo| {
                let (restored, operation) =
                    capture_mutation_operation_id(|| repo.op_restore(&operation_id));
                restored.map(|()| operation)
            },
            |vm, _, cx| vm.refresh(false, cx),
        );
        Some(cx.spawn(async move |this, cx| {
            let result = restore.await;
            let _ = this.update(cx, |vm, cx| {
                vm.restoring_operation = false;
                cx.notify();
            });
            result
        }))
    }
}
