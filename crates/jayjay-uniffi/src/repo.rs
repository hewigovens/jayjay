use std::path::PathBuf;
use std::sync::Arc;

use jayjay_core::{
    AnnotationLine, BookmarkInfo, ChangeDetail, ChangeInfo, CliStatus, DiffEditDestination,
    DiffEditFileSelection, DiffHunk, DiffStats, EvologEntry, FetchResult, FileTreeEntry,
    GitSubmoduleStatus, GraphEntry, JjCommand, JjCommandResult, OpLogEntry, PrInfo, Repo,
    ToolsConfig, WorkspaceInfo,
    diff::{self, CollapsedDiff, FileDiff},
};

use crate::error::JayJayError;

#[uniffi::export]
pub fn build_file_tree(paths: Vec<String>) -> Vec<FileTreeEntry> {
    jayjay_core::file_tree::build_file_tree(&paths)
}

#[uniffi::export]
pub fn detect_ai_provider() -> String {
    jayjay_core::detect_ai_provider()
}

#[uniffi::export]
pub fn commit_message_prompt() -> String {
    jayjay_core::COMMIT_MESSAGE_PROMPT.to_owned()
}

#[uniffi::export]
pub fn default_revset() -> String {
    jayjay_core::DEFAULT_REVSET.to_owned()
}

#[uniffi::export]
pub fn default_revset_with_depth(depth: u32) -> String {
    jayjay_core::build_default_revset(depth)
}

#[uniffi::export]
pub fn check_jj_environment() -> CliStatus {
    jayjay_core::check_jj_environment()
}

#[uniffi::export]
pub fn check_gh_environment() -> CliStatus {
    jayjay_core::check_gh_environment()
}

#[uniffi::export]
pub fn jj_command_body(query: String) -> Option<String> {
    JjCommand::from_palette_query(&query).map(JjCommand::into_raw)
}

/// Fuzzy-rank `candidates` against `query`; returns matching indices, best first.
#[uniffi::export]
pub fn fuzzy_rank(query: String, candidates: Vec<String>) -> Vec<u32> {
    jayjay_core::fuzzy::rank(&query, &candidates)
}

/// True when diff content is editable text, not a placeholder (binary, submodule, LFS, image, etc.).
#[uniffi::export]
pub fn is_editable_diff_text(text: String) -> bool {
    jayjay_core::placeholder::is_editable_text(&text)
}

/// True when diff content is a Git LFS pointer/object placeholder.
#[uniffi::export]
pub fn is_git_lfs_placeholder(text: String) -> bool {
    jayjay_core::placeholder::is_git_lfs_placeholder(&text)
}

/// True when diff content is a Git submodule placeholder.
#[uniffi::export]
pub fn is_git_submodule_placeholder(text: String) -> bool {
    jayjay_core::placeholder::is_git_submodule_placeholder(&text)
}

/// Canonical review-store path, so the SwiftUI shell persists to the same file as the Rust core.
#[uniffi::export]
pub fn review_store_path() -> Option<String> {
    jayjay_core::review::ReviewStore::store_path().map(|p| p.to_string_lossy().into_owned())
}

/// Outcome of a palette history recall: the query to show and the cursor index.
#[derive(uniffi::Record, Debug, Clone)]
pub struct PaletteRecall {
    pub query: String,
    pub history_index: Option<u32>,
}

/// Push `command` onto `history` newest-first, deduped, capped at the limit.
#[uniffi::export]
pub fn palette_record_history(command: String, history: Vec<String>) -> Vec<String> {
    jayjay_core::palette::record(&command, &history)
}

/// Walk the palette history cursor one step (`older` toward older entries, else newer).
#[uniffi::export]
pub fn palette_recall_history(
    history: Vec<String>,
    history_index: Option<u32>,
    older: bool,
) -> Option<PaletteRecall> {
    jayjay_core::palette::recall(&history, history_index.map(|ix| ix as usize), older).map(
        |recall| PaletteRecall {
            query: recall.query,
            history_index: recall.index.map(|ix| ix as u32),
        },
    )
}

#[uniffi::export]
pub fn parse_jj_command_args(command: String) -> Option<Vec<String>> {
    JjCommand::new(command).parse_args()
}

#[uniffi::export]
pub fn run_jj_command_in_repo_path(
    repo_path: String,
    command: String,
) -> Result<JjCommandResult, JayJayError> {
    Ok(JjCommand::new(command).run_in_path(&PathBuf::from(repo_path))?)
}

/// Resolve a CLI binary by walking the same fallback paths jj does. Returns
/// the absolute path when found, `nil` otherwise. macOS `.app` bundles get
/// stripped PATH from launchd, so this avoids relying on shell PATH.
#[uniffi::export]
pub fn find_binary(name: String) -> Option<String> {
    jayjay_core::find_existing_binary(&name)
}

#[uniffi::export]
pub fn login_shell_path() -> Option<String> {
    jayjay_core::login_shell_path()
}

#[uniffi::export]
pub fn login_shell() -> String {
    jayjay_core::login_shell()
}

/// Open a file in the user-configured external editor. Returns false on
/// missing binary / spawn failure.
#[uniffi::export]
pub fn open_in_editor(
    repo_path: String,
    file_path: String,
    external_editor: String,
    custom_editor_command: String,
    terminal: String,
    custom_terminal_command: String,
) -> bool {
    jayjay_core::open_in_editor(
        &repo_path,
        &file_path,
        &ToolsConfig {
            external_editor,
            custom_editor_command,
            terminal,
            custom_terminal_command,
        },
    )
}

/// Open the user-configured terminal at `repo_path`, optionally running a
/// command after `cd`-ing in.
#[uniffi::export]
pub fn open_in_terminal(
    repo_path: String,
    command: Option<String>,
    terminal: String,
    custom_terminal_command: String,
) -> bool {
    jayjay_core::open_in_terminal(
        &repo_path,
        command.as_deref(),
        &ToolsConfig {
            terminal,
            custom_terminal_command,
            ..Default::default()
        },
    )
}

#[derive(uniffi::Object)]
pub struct JayJayRepo {
    inner: Repo,
}

#[uniffi::export]
impl JayJayRepo {
    #[uniffi::constructor]
    pub fn open(path: String) -> Result<Arc<Self>, JayJayError> {
        let repo = Repo::open(&PathBuf::from(&path))?;
        Ok(Arc::new(Self { inner: repo }))
    }

    pub fn path(&self) -> String {
        self.inner.path().display().to_string()
    }

    pub fn refresh_working_copy(&self) -> Result<(), JayJayError> {
        Ok(self.inner.refresh_working_copy()?)
    }

    pub fn working_copy_is_large(&self) -> bool {
        self.inner.working_copy_is_large()
    }

    pub fn has_unignored_working_copy_paths(
        &self,
        paths: Vec<String>,
    ) -> Result<bool, JayJayError> {
        Ok(self.inner.has_unignored_working_copy_paths(&paths)?)
    }

    pub fn log(&self, revset: String) -> Result<Vec<ChangeInfo>, JayJayError> {
        Ok(self.inner.log(&revset)?)
    }

    pub fn log_graph(&self, revset: String) -> Result<Vec<GraphEntry>, JayJayError> {
        Ok(self.inner.log_graph(&revset)?)
    }

    pub fn show(&self, rev: String) -> Result<ChangeDetail, JayJayError> {
        Ok(self.inner.show(&rev)?)
    }

    /// Fast: file list without content.
    pub fn show_summary(&self, rev: String) -> Result<ChangeDetail, JayJayError> {
        Ok(self.inner.show_summary(&rev)?)
    }

    /// Single file with content.
    pub fn show_file(&self, rev: String, path: String) -> Result<DiffHunk, JayJayError> {
        Ok(self.inner.show_file(&rev, &path)?)
    }

    /// Renamed file: old content from old_path, new content from new_path.
    pub fn show_file_rename(
        &self,
        rev: String,
        old_path: String,
        new_path: String,
    ) -> Result<DiffHunk, JayJayError> {
        Ok(self.inner.show_file_rename(&rev, &old_path, &new_path)?)
    }

    /// Fast: file list between two arbitrary revisions (no content).
    pub fn interdiff_summary(
        &self,
        from_rev: String,
        to_rev: String,
    ) -> Result<ChangeDetail, JayJayError> {
        Ok(self.inner.interdiff_summary(&from_rev, &to_rev)?)
    }

    /// Single file content between two arbitrary revisions.
    pub fn interdiff_file(
        &self,
        from_rev: String,
        to_rev: String,
        path: String,
    ) -> Result<DiffHunk, JayJayError> {
        Ok(self.inner.interdiff_file(&from_rev, &to_rev, &path)?)
    }

    pub fn workspace_list(&self) -> Result<Vec<WorkspaceInfo>, JayJayError> {
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

    pub fn pull_request_info(&self, bookmark: String) -> Option<PrInfo> {
        self.inner.pull_request_info(&bookmark)
    }

    pub fn pull_request_open_url(&self, bookmark: String) -> Option<String> {
        self.inner.pull_request_open_url(&bookmark)
    }

    pub fn pr_host_name(&self) -> Option<String> {
        self.inner.pr_host_name()
    }

    pub fn diff_stats(&self, rev: String) -> Result<DiffStats, JayJayError> {
        Ok(self.inner.diff_stats(&rev)?)
    }

    pub fn annotate_file(
        &self,
        rev: String,
        path: String,
    ) -> Result<Vec<AnnotationLine>, JayJayError> {
        Ok(self.inner.annotate_file(&rev, &path)?)
    }

    pub fn file_history(&self, path: String) -> Result<Vec<ChangeInfo>, JayJayError> {
        Ok(self.inner.file_history(&path)?)
    }

    pub fn evolog(&self, rev: String) -> Result<Vec<EvologEntry>, JayJayError> {
        Ok(self.inner.evolog(&rev)?)
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

    pub fn revert_change(&self, rev: String) -> Result<(), JayJayError> {
        Ok(self.inner.revert_change(&rev)?)
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

    pub fn list_bookmarks(&self) -> Result<Vec<BookmarkInfo>, JayJayError> {
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

    pub fn forget_stale_bookmarks(&self) -> Result<u32, JayJayError> {
        Ok(self.inner.forget_stale_bookmarks()?)
    }

    pub fn git_push(&self, bookmark: String) -> Result<String, JayJayError> {
        Ok(self.inner.git_push(&bookmark)?)
    }

    pub fn remote_web_url(&self) -> Option<String> {
        self.inner.remote_web_url()
    }

    pub fn git_fetch(&self, remote: String) -> Result<FetchResult, JayJayError> {
        Ok(self.inner.git_fetch(&remote)?)
    }

    pub fn git_pull_bookmark(&self, bookmark: String) -> Result<FetchResult, JayJayError> {
        Ok(self.inner.git_pull_bookmark(&bookmark)?)
    }

    pub fn jj_commit(&self, message: String) -> Result<(), JayJayError> {
        Ok(self.inner.jj_commit(&message)?)
    }

    pub fn submodule_statuses(&self) -> Result<Vec<GitSubmoduleStatus>, JayJayError> {
        Ok(self.inner.submodule_statuses()?)
    }

    pub fn commit_safe_submodule_updates(
        &self,
        message: String,
        paths: Vec<String>,
    ) -> Result<String, JayJayError> {
        Ok(self.inner.commit_safe_submodule_updates(&message, &paths)?)
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

    pub fn check_user_config(&self) -> Option<String> {
        self.inner.check_user_config()
    }

    pub fn op_log(&self) -> Result<Vec<OpLogEntry>, JayJayError> {
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
    ) -> FileDiff {
        diff::compute_file_diff(&path, &old_content, &new_content, ignore_whitespace)
    }

    pub fn compute_native_diff_full(
        &self,
        path: String,
        old_content: String,
        new_content: String,
        ignore_whitespace: bool,
    ) -> FileDiff {
        diff::compute_file_diff_full(&path, &old_content, &new_content, ignore_whitespace)
    }

    pub fn collapse_diff_with_mapping(&self, diff: FileDiff) -> CollapsedDiff {
        diff::collapse_context_with_mapping(&diff)
    }

    pub fn apply_diff_selection(
        &self,
        rev: String,
        destination: DiffEditDestination,
        selections: Vec<DiffEditFileSelection>,
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
