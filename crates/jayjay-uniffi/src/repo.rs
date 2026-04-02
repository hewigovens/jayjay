use std::path::PathBuf;
use std::sync::Arc;

use jayjay_core as core;

use crate::error::JayJayError;

#[uniffi::export]
pub fn build_file_tree(paths: Vec<String>) -> Vec<core::FileTreeEntry> {
    core::file_tree::build_file_tree(&paths)
}

#[uniffi::export]
pub fn detect_ai_provider() -> String {
    core::detect_ai_provider()
}

#[uniffi::export]
pub fn commit_message_prompt() -> String {
    core::COMMIT_MESSAGE_PROMPT.to_owned()
}

#[uniffi::export]
pub fn default_revset() -> String {
    core::DEFAULT_REVSET.to_owned()
}

#[uniffi::export]
pub fn default_revset_with_depth(depth: u32) -> String {
    core::build_default_revset(depth)
}

#[uniffi::export]
pub fn check_jj_environment() -> core::JJStatus {
    core::check_jj_environment()
}

#[derive(uniffi::Object)]
pub struct JayJayRepo {
    inner: core::Repo,
}

#[uniffi::export]
impl JayJayRepo {
    #[uniffi::constructor]
    pub fn open(path: String) -> Result<Arc<Self>, JayJayError> {
        let repo = core::Repo::open(&PathBuf::from(&path))?;
        Ok(Arc::new(Self { inner: repo }))
    }

    pub fn path(&self) -> String {
        self.inner.path().display().to_string()
    }

    pub fn refresh_working_copy(&self) -> Result<(), JayJayError> {
        Ok(self.inner.refresh_working_copy()?)
    }

    pub fn log(&self, revset: String) -> Result<Vec<core::ChangeInfo>, JayJayError> {
        Ok(self.inner.log(&revset)?)
    }

    pub fn log_graph(&self, revset: String) -> Result<Vec<core::GraphEntry>, JayJayError> {
        Ok(self.inner.log_graph(&revset)?)
    }

    pub fn show(&self, rev: String) -> Result<core::ChangeDetail, JayJayError> {
        Ok(self.inner.show(&rev)?)
    }

    /// Fast: file list without content.
    pub fn show_summary(&self, rev: String) -> Result<core::ChangeDetail, JayJayError> {
        Ok(self.inner.show_summary(&rev)?)
    }

    /// Single file with content.
    pub fn show_file(&self, rev: String, path: String) -> Result<core::DiffHunk, JayJayError> {
        Ok(self.inner.show_file(&rev, &path)?)
    }

    /// Renamed file: old content from old_path, new content from new_path.
    pub fn show_file_rename(
        &self,
        rev: String,
        old_path: String,
        new_path: String,
    ) -> Result<core::DiffHunk, JayJayError> {
        Ok(self.inner.show_file_rename(&rev, &old_path, &new_path)?)
    }

    /// Fast: file list between two arbitrary revisions (no content).
    pub fn interdiff_summary(
        &self,
        from_rev: String,
        to_rev: String,
    ) -> Result<core::ChangeDetail, JayJayError> {
        Ok(self.inner.interdiff_summary(&from_rev, &to_rev)?)
    }

    /// Single file content between two arbitrary revisions.
    pub fn interdiff_file(
        &self,
        from_rev: String,
        to_rev: String,
        path: String,
    ) -> Result<core::DiffHunk, JayJayError> {
        Ok(self.inner.interdiff_file(&from_rev, &to_rev, &path)?)
    }

    pub fn workspace_list(&self) -> Result<Vec<core::WorkspaceInfo>, JayJayError> {
        Ok(self.inner.workspace_list()?)
    }

    pub fn workspace_add(
        &self,
        dest: String,
        name: String,
        rev: String,
    ) -> Result<String, JayJayError> {
        Ok(self.inner.workspace_add(&dest, &name, &rev)?)
    }

    pub fn workspace_forget(&self, name: String) -> Result<(), JayJayError> {
        Ok(self.inner.workspace_forget(&name)?)
    }

    pub fn diff_stats(&self, rev: String) -> Result<core::DiffStats, JayJayError> {
        Ok(self.inner.diff_stats(&rev)?)
    }

    pub fn annotate_file(
        &self,
        rev: String,
        path: String,
    ) -> Result<Vec<core::AnnotationLine>, JayJayError> {
        Ok(self.inner.annotate_file(&rev, &path)?)
    }

    pub fn file_history(&self, path: String) -> Result<Vec<core::ChangeInfo>, JayJayError> {
        Ok(self.inner.file_history(&path)?)
    }

    pub fn resolve_list(&self, rev: String) -> Result<Vec<String>, JayJayError> {
        Ok(self.inner.resolve_list(&rev)?)
    }

    pub fn resolve_use_ours(&self, rev: String, path: String) -> Result<(), JayJayError> {
        Ok(self.inner.resolve_use_ours(&rev, &path)?)
    }

    pub fn resolve_use_theirs(&self, rev: String, path: String) -> Result<(), JayJayError> {
        Ok(self.inner.resolve_use_theirs(&rev, &path)?)
    }

    pub fn resolve_with_tool(
        &self,
        rev: String,
        path: String,
        tool: String,
    ) -> Result<(), JayJayError> {
        Ok(self.inner.resolve_with_tool(&rev, &path, &tool)?)
    }

    pub fn file_content(&self, rev: String, path: String) -> Result<String, JayJayError> {
        Ok(self.inner.file_content(&rev, &path)?)
    }

    pub fn restore_files(&self, rev: String, paths: Vec<String>) -> Result<(), JayJayError> {
        Ok(self.inner.restore_files(&rev, &paths)?)
    }

    pub fn move_to_working_copy(&self, rev: String, paths: Vec<String>) -> Result<(), JayJayError> {
        Ok(self.inner.move_to_working_copy(&rev, &paths)?)
    }

    pub fn delete_files(&self, paths: Vec<String>) -> Result<(), JayJayError> {
        Ok(self.inner.delete_files(&paths)?)
    }

    pub fn ignore_and_untrack(&self, paths: Vec<String>) -> Result<(), JayJayError> {
        Ok(self.inner.ignore_and_untrack(&paths)?)
    }

    pub fn split(
        &self,
        rev: String,
        paths: Vec<String>,
        message: String,
        parallel: bool,
    ) -> Result<(), JayJayError> {
        Ok(self.inner.split(&rev, &paths, &message, parallel)?)
    }

    pub fn describe(&self, rev: String, message: String) -> Result<(), JayJayError> {
        Ok(self.inner.describe(&rev, &message)?)
    }

    pub fn new_change(&self, parent: String, message: String) -> Result<(), JayJayError> {
        Ok(self.inner.new_change(&parent, &message)?)
    }

    pub fn squash(&self, rev: String, into_rev: Option<String>) -> Result<(), JayJayError> {
        Ok(self.inner.squash(&rev, into_rev.as_deref())?)
    }

    pub fn edit(&self, rev: String) -> Result<(), JayJayError> {
        Ok(self.inner.edit(&rev)?)
    }

    pub fn graft(&self, rev: String) -> Result<(), JayJayError> {
        Ok(self.inner.graft(&rev)?)
    }

    pub fn absorb(&self, rev: String) -> Result<(), JayJayError> {
        Ok(self.inner.absorb(&rev)?)
    }

    pub fn backout(&self, rev: String) -> Result<(), JayJayError> {
        Ok(self.inner.backout(&rev)?)
    }

    pub fn merge(&self, parent_revs: Vec<String>) -> Result<(), JayJayError> {
        Ok(self.inner.merge(&parent_revs)?)
    }

    pub fn duplicate(&self, rev: String) -> Result<(), JayJayError> {
        Ok(self.inner.duplicate(&rev)?)
    }

    pub fn abandon(&self, rev: String) -> Result<(), JayJayError> {
        Ok(self.inner.abandon(&rev)?)
    }

    pub fn rebase(&self, rev: String, dest: String) -> Result<(), JayJayError> {
        Ok(self.inner.rebase(&rev, &dest)?)
    }

    pub fn list_bookmarks(&self) -> Result<Vec<core::BookmarkInfo>, JayJayError> {
        Ok(self.inner.list_bookmarks()?)
    }

    pub fn create_bookmark(&self, name: String, rev: String) -> Result<(), JayJayError> {
        Ok(self.inner.create_bookmark(&name, &rev)?)
    }

    pub fn move_bookmark(&self, name: String, to_rev: String) -> Result<(), JayJayError> {
        Ok(self.inner.move_bookmark(&name, &to_rev)?)
    }

    pub fn delete_bookmark(&self, name: String) -> Result<(), JayJayError> {
        Ok(self.inner.delete_bookmark(&name)?)
    }

    pub fn rename_bookmark(&self, old_name: String, new_name: String) -> Result<(), JayJayError> {
        Ok(self.inner.rename_bookmark(&old_name, &new_name)?)
    }

    pub fn track_bookmark(&self, name: String, remote: String) -> Result<(), JayJayError> {
        Ok(self.inner.track_bookmark(&name, &remote)?)
    }

    pub fn git_push(&self, bookmark: String) -> Result<String, JayJayError> {
        Ok(self.inner.git_push(&bookmark)?)
    }

    pub fn git_remote_url(&self) -> Result<String, JayJayError> {
        Ok(self.inner.git_remote_url()?)
    }

    pub fn git_fetch(&self, remote: String) -> Result<String, JayJayError> {
        Ok(self.inner.git_fetch(&remote)?)
    }

    pub fn git_pull_bookmark(&self, bookmark: String) -> Result<String, JayJayError> {
        Ok(self.inner.git_pull_bookmark(&bookmark)?)
    }

    pub fn jj_commit(&self, message: String) -> Result<(), JayJayError> {
        Ok(self.inner.jj_commit(&message)?)
    }

    pub fn commit_with_submodules(&self, message: String) -> Result<(), JayJayError> {
        Ok(self.inner.commit_with_submodules(&message)?)
    }

    pub fn dirty_submodules(&self) -> Result<Vec<String>, JayJayError> {
        Ok(self.inner.dirty_submodules()?)
    }

    pub fn changed_submodules(&self) -> Result<Vec<String>, JayJayError> {
        Ok(self.inner.changed_submodules()?)
    }

    pub fn submodule_statuses(&self) -> Result<Vec<core::GitSubmoduleStatus>, JayJayError> {
        Ok(self.inner.submodule_statuses()?)
    }

    pub fn commit_safe_submodule_updates(
        &self,
        message: String,
        paths: Vec<String>,
    ) -> Result<String, JayJayError> {
        Ok(self.inner.commit_safe_submodule_updates(&message, &paths)?)
    }

    pub fn tracked_git_lfs_files(&self) -> Result<Vec<String>, JayJayError> {
        Ok(self.inner.tracked_git_lfs_files()?)
    }

    pub fn git_lfs_paths(&self, paths: Vec<String>) -> Result<Vec<String>, JayJayError> {
        Ok(self.inner.git_lfs_paths(&paths)?)
    }

    pub fn diff_summary(&self) -> Result<String, JayJayError> {
        Ok(self.inner.diff_summary()?)
    }

    pub fn generate_commit_message(&self, diff_summary: String) -> Option<String> {
        self.inner.generate_commit_message(&diff_summary)
    }

    pub fn jj_config(&self) -> Result<String, JayJayError> {
        Ok(self.inner.jj_config()?)
    }

    pub fn jj_config_path(&self) -> Result<String, JayJayError> {
        Ok(self.inner.jj_config_path()?)
    }

    pub fn check_user_config(&self) -> Option<String> {
        self.inner.check_user_config()
    }

    pub fn op_log(&self) -> Result<Vec<core::OpLogEntry>, JayJayError> {
        Ok(self.inner.op_log()?)
    }

    pub fn op_restore(&self, op_id: String) -> Result<(), JayJayError> {
        Ok(self.inner.op_restore(&op_id)?)
    }

    pub fn compute_native_diff(
        &self,
        path: String,
        old_content: String,
        new_content: String,
        ignore_whitespace: bool,
    ) -> core::diff::FileDiff {
        core::diff::compute_file_diff(&path, &old_content, &new_content, ignore_whitespace)
    }

    pub fn compute_native_diff_full(
        &self,
        path: String,
        old_content: String,
        new_content: String,
        ignore_whitespace: bool,
    ) -> core::diff::FileDiff {
        core::diff::compute_file_diff_full(&path, &old_content, &new_content, ignore_whitespace)
    }

    pub fn apply_diff_selection(
        &self,
        rev: String,
        destination: core::DiffEditDestination,
        selections: Vec<core::DiffEditFileSelection>,
        message: String,
        ignore_whitespace: bool,
    ) -> Result<(), JayJayError> {
        Ok(self.inner.apply_diff_selection(
            &rev,
            destination,
            &selections,
            &message,
            ignore_whitespace,
        )?)
    }
}
