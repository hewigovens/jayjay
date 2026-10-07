use gpui::Context;
use jayjay_core::{FileEditorData, JayResult};

use super::RepoViewModel;

impl RepoViewModel {
    pub(crate) fn load_working_copy_file_editor(
        &mut self,
        path: String,
        cx: &mut Context<Self>,
    ) -> gpui::Task<JayResult<FileEditorData>> {
        self.repo_load_task(
            cx,
            move |repo| repo.working_copy_file_editor(&path),
            |_, _, _| {},
        )
    }

    pub(crate) fn apply_working_copy_file_editor(
        &mut self,
        data: FileEditorData,
        content: String,
        cx: &mut Context<Self>,
    ) -> gpui::Task<JayResult<()>> {
        let restore_path = data.path.clone();
        self.repo_write_task(
            cx,
            move |repo| repo.apply_working_copy_file_editor(&data, &content),
            move |vm, cx| {
                vm.pending_file_selection = Some(restore_path);
                vm.refresh(false, cx);
            },
        )
    }
}
