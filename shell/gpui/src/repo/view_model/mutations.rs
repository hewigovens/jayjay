use gpui::Context;
use jayjay_core::{
    CoreResult, DiffEditDestination, DiffEditFileSelection, FetchResult, init_jj_git_repo,
};

use super::RepoViewModel;

impl RepoViewModel {
    pub fn describe_change(
        &mut self,
        rev: String,
        message: String,
        cx: &mut Context<Self>,
    ) -> gpui::Task<CoreResult<()>> {
        self.repo_write_task(
            cx,
            move |repo| repo.describe(&rev, &message),
            |vm, cx| vm.refresh(false, cx),
        )
    }

    pub fn commit_working_copy(
        &mut self,
        message: String,
        cx: &mut Context<Self>,
    ) -> gpui::Task<CoreResult<()>> {
        self.repo_write_task(
            cx,
            move |repo| repo.jj_commit(&message),
            |vm, cx| {
                vm.selected = None;
                vm.refresh(false, cx);
            },
        )
    }

    /// Split `paths` out of `rev` into a new change described by `message` (`jj split`; on @ this is also `jj commit FILESETS`); `parallel` makes the split parts siblings sharing `rev`'s parents instead of parent/child. The remainder keeps the rest under a fresh change id.
    pub fn split_files(
        &mut self,
        rev: String,
        paths: Vec<String>,
        message: String,
        parallel: bool,
        cx: &mut Context<Self>,
    ) -> gpui::Task<CoreResult<()>> {
        self.repo_write_task(
            cx,
            move |repo| repo.split(&rev, &paths, &message, parallel),
            |vm, cx| {
                // The remainder is the new @; drop the old selection so the refresh lands on the working copy.
                vm.selected = None;
                vm.refresh(false, cx);
            },
        )
    }

    pub fn new_change_on_top(
        &mut self,
        parent: String,
        cx: &mut Context<Self>,
    ) -> gpui::Task<CoreResult<()>> {
        self.repo_write_task(
            cx,
            move |repo| repo.new_change(&parent, ""),
            |vm, cx| {
                vm.selected = None;
                vm.refresh(false, cx);
            },
        )
    }

    pub fn abandon_change(
        &mut self,
        rev: String,
        cx: &mut Context<Self>,
    ) -> gpui::Task<CoreResult<()>> {
        self.repo_write_task(
            cx,
            move |repo| repo.abandon(&rev),
            |vm, cx| {
                vm.selected = None;
                vm.refresh(false, cx);
            },
        )
    }

    /// `selection` is built from content retained at diff-display time (see `ComputedDiff`/`LoadedDiff`) — must not re-read the file here, since the working copy may have moved on.
    pub fn abandon_selected_diff_lines(
        &mut self,
        rev: String,
        selection: DiffEditFileSelection,
        ignore_whitespace: bool,
        cx: &mut Context<Self>,
    ) -> gpui::Task<CoreResult<()>> {
        // The refresh below reloads the file list from scratch; without this, `select_change` would default back to index 0 instead of the file the user was just working in.
        let path = selection.path.clone();
        self.repo_write_task(
            cx,
            move |repo| {
                repo.apply_diff_selection(
                    &rev,
                    DiffEditDestination::RemoveFromSource,
                    &[selection],
                    "",
                    ignore_whitespace,
                )
            },
            move |vm, cx| {
                vm.pending_file_selection = Some(path);
                vm.refresh(false, cx);
            },
        )
    }

    pub fn resolve_with_tool(
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

    pub fn create_bookmark(
        &mut self,
        name: String,
        rev: String,
        cx: &mut Context<Self>,
    ) -> gpui::Task<CoreResult<()>> {
        self.repo_write_task(
            cx,
            move |repo| repo.create_bookmark(&name, &rev),
            |vm, cx| vm.refresh(false, cx),
        )
    }

    pub fn move_bookmark_to_parent(
        &mut self,
        name: String,
        cx: &mut Context<Self>,
    ) -> gpui::Task<CoreResult<()>> {
        self.repo_write_task(
            cx,
            move |repo| repo.move_bookmark(&name, "@-"),
            |vm, cx| vm.refresh(false, cx),
        )
    }

    /// Move an existing bookmark onto an arbitrary revision (drag-and-drop target).
    pub fn move_bookmark(
        &mut self,
        name: String,
        to_rev: String,
        cx: &mut Context<Self>,
    ) -> gpui::Task<CoreResult<()>> {
        self.repo_write_task(
            cx,
            move |repo| repo.move_bookmark(&name, &to_rev),
            |vm, cx| vm.refresh(false, cx),
        )
    }

    pub fn delete_bookmark(
        &mut self,
        name: String,
        cx: &mut Context<Self>,
    ) -> gpui::Task<CoreResult<()>> {
        self.repo_write_task(
            cx,
            move |repo| repo.delete_bookmark(&name),
            |vm, cx| vm.refresh(false, cx),
        )
    }

    pub fn push_bookmark(
        &mut self,
        name: String,
        cx: &mut Context<Self>,
    ) -> gpui::Task<CoreResult<String>> {
        self.repo_result_task(
            cx,
            move |repo| repo.git_push(&name),
            |vm, _message, cx| vm.refresh(false, cx),
        )
    }

    pub fn git_fetch_origin(
        &mut self,
        cx: &mut Context<Self>,
    ) -> gpui::Task<CoreResult<FetchResult>> {
        self.repo_result_task(
            cx,
            move |repo| repo.git_fetch("origin"),
            |vm, _result, cx| vm.refresh(false, cx),
        )
    }

    pub fn forget_stale_bookmarks(
        &mut self,
        cx: &mut Context<Self>,
    ) -> gpui::Task<CoreResult<u32>> {
        self.repo_result_task(
            cx,
            move |repo| repo.forget_stale_bookmarks(),
            |vm, _count, cx| vm.refresh(false, cx),
        )
    }

    pub fn workspace_forget(
        &mut self,
        name: String,
        cx: &mut Context<Self>,
    ) -> gpui::Task<CoreResult<()>> {
        self.repo_write_task(
            cx,
            move |repo| repo.workspace_forget(&name),
            |vm, cx| vm.refresh(false, cx),
        )
    }

    pub fn workspace_add(
        &mut self,
        dest: String,
        name: String,
        cx: &mut Context<Self>,
    ) -> gpui::Task<CoreResult<String>> {
        self.repo_result_task(
            cx,
            move |repo| repo.workspace_add(&dest, &name, ""),
            |vm, _output, cx| vm.refresh(false, cx),
        )
    }

    pub fn initialize_repo(&mut self, cx: &mut Context<Self>) -> gpui::Task<CoreResult<()>> {
        let path = std::path::PathBuf::from(self.repo_path.as_ref());
        self.clear_error();
        self.begin_refreshing(cx);

        Self::core_result_task(
            cx,
            {
                let path = path.clone();
                async move { init_jj_git_repo(&path) }
            },
            move |vm, result, cx| match result {
                Ok(()) => {
                    // Open off the main thread; the window arms its FS watcher once the repo lands.
                    *vm = RepoViewModel::opening(path);
                    vm.open_async(cx);
                    cx.notify();
                    Ok(())
                }
                Err(error) => {
                    vm.finish_refreshing(cx);
                    vm.present_error(&error);
                    cx.notify();
                    Err(error)
                }
            },
        )
    }
}
