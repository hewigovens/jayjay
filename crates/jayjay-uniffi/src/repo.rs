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

    pub fn restore_files(&self, rev: String, paths: Vec<String>) -> Result<(), JayJayError> {
        Ok(self.inner.restore_files(&rev, &paths)?)
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
    ) -> Result<(), JayJayError> {
        Ok(self.inner.split(&rev, &paths, &message)?)
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

    pub fn git_push(&self, bookmark: String) -> Result<String, JayJayError> {
        Ok(self.inner.git_push(&bookmark)?)
    }

    pub fn git_fetch(&self, remote: String) -> Result<String, JayJayError> {
        Ok(self.inner.git_fetch(&remote)?)
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
}
