use std::sync::Arc;

use gpui::Context;
use jayjay_core::{ConflictEditorData, JayResult, Repo};

use super::RepoViewModel;

impl RepoViewModel {
    pub(crate) fn resolve_file(
        &mut self,
        resolve: impl FnOnce(Arc<Repo>) -> JayResult<()> + Send + 'static,
        cx: &mut Context<Self>,
    ) -> gpui::Task<JayResult<()>> {
        self.repo_write_task(cx, resolve, |vm, cx| vm.refresh(false, cx))
    }

    pub(crate) fn load_conflict_editor(
        &mut self,
        rev: String,
        path: String,
        cx: &mut Context<Self>,
    ) -> gpui::Task<JayResult<ConflictEditorData>> {
        self.repo_load_task(
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
    ) -> gpui::Task<JayResult<()>> {
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
