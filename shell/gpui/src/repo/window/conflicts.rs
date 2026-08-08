use gpui::Context;

use super::RepoWindow;

impl RepoWindow {
    pub(crate) fn resolve_selected_file_with_tool(&mut self, tool: String, cx: &mut Context<Self>) {
        let Some((rev, path)) = self.selected_resolution_target(cx) else {
            self.show_toast("No conflicted file selected", cx);
            return;
        };
        let task = self
            .vm
            .update(cx, |vm, cx| vm.resolve_with_tool(rev, path, tool, cx));
        task.detach();
    }

    pub(crate) fn open_selected_file_in_editor(&mut self, cx: &mut Context<Self>) {
        let Some((_, path)) = self.selected_resolution_target(cx) else {
            self.show_toast("No file selected", cx);
            return;
        };
        let repo_path = self.vm.read(cx).repo_path.to_string();
        if !crate::app::tools::open_in_editor(&repo_path, &path, cx) {
            self.show_toast("Editor could not be opened", cx);
        }
    }

    fn selected_resolution_target(&self, cx: &Context<Self>) -> Option<(String, String)> {
        let vm = self.vm.read(cx);
        let rev = vm.selected_revision()?;
        let path = vm.selected_hunk()?.path.clone();
        Some((rev, path))
    }
}
