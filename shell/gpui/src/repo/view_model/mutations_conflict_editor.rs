use gpui::Context;
use jayjay_core::{ConflictEditorData, CoreResult};

use super::RepoViewModel;

impl RepoViewModel {
    pub(crate) fn resolve_with_tool(
        &mut self,
        rev: String,
        path: String,
        tool: String,
        cx: &mut Context<Self>,
    ) -> gpui::Task<CoreResult<()>> {
        self.repo_write_task(
            cx,
            move |repo| repo.resolve_with_tool(&rev, &path, &tool),
            |vm, cx| vm.refresh(false, cx),
        )
    }

    pub(crate) fn load_conflict_editor(
        &mut self,
        rev: String,
        path: String,
        cx: &mut Context<Self>,
    ) -> gpui::Task<CoreResult<ConflictEditorData>> {
        self.repo_result_task_without_indicator(
            cx,
            move |repo| repo.conflict_editor(&rev, &path),
            |_, _, _| {},
        )
    }

    pub(crate) fn apply_conflict_editor(
        &mut self,
        rev: String,
        data: ConflictEditorData,
        content: String,
        cx: &mut Context<Self>,
    ) -> gpui::Task<CoreResult<()>> {
        let restore_path = data.path.clone();
        self.repo_write_task(
            cx,
            move |repo| repo.apply_conflict_editor(&rev, &data, &content),
            move |vm, cx| {
                vm.pending_file_selection = Some(restore_path);
                vm.refresh(false, cx);
            },
        )
    }
}
