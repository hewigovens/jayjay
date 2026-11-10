use gpui::{Context, Task};
use jayjay_core::{JayResult, UndoState};

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
        self.repo_result_task(
            cx,
            move |repo| match step {
                OperationStep::Undo => repo.undo(&operation_id),
                OperationStep::Redo => repo.redo(&operation_id),
            },
            |vm, _, cx| vm.refresh_selecting_revision(None, cx),
        )
    }
}
