use std::path::{Path, PathBuf};
use std::sync::Arc;

use jayjay_core::overview::OverviewSnapshot;
use jayjay_core::{
    AnnotationLine, BisectState, BisectVerdict, BookmarkInfo, ChangeDetail, ChangeInfo, CliStatus,
    CloneRequest, ConflictEditorData, DiffEditDestination, DiffEditFileSelection, DiffExcerpt,
    DiffHunk, DiffProjectionMode, DiffStats, EvologEntry, EvologRow, FetchResult, FileDiffStats,
    FileEditorData, FixSummary, GitSubmoduleStatus, GraphEntry, InsertPosition, JjCommand,
    JjCommandResult, MutationEffect, OpLogEntry, PrInfo, PullRequestImportPreview, RebaseMode,
    RecentRevsetStore, Repo, RevsetPreset, Stack, StackedPrResult, SubmitStackLayer, SyncToken,
    TagInfo, ToolsConfig, UndoState, WorkspaceInfo, WorkspacePresence,
    diff::{self, CollapsedDiff, FileDiff, ReviewFileSnapshot},
    review_display_group_map_from_hunk, review_snapshot_from_hunk,
};
use jayjay_primitives::{ConvergeChoices, ConvergeOutcome, ConvergePlan, ReviewNoteStatus};

use crate::dag::{DagSelectionGraph, GraphWithLayout, layout_data};
use jayjay_core::JayError;
use jayjay_core::dag::DagLayout;

#[uniffi::export]
fn take_mutation_operation_id() -> Option<String> {
    jayjay_core::take_mutation_operation_id()
}

#[uniffi::export]
fn compose_bisect_range(good: String, bad: String, range: String) -> Result<String, JayError> {
    jayjay_core::compose_bisect_range(&good, &bad, &range)
}

#[uniffi::export]
fn default_revset() -> String {
    jayjay_core::DEFAULT_REVSET.to_owned()
}

#[uniffi::export]
fn default_revset_with_depth(depth: u32) -> String {
    jayjay_core::build_default_revset(depth)
}

#[uniffi::export]
fn default_revset_depth(revset: String) -> Option<u32> {
    jayjay_core::default_revset_depth(&revset)
}

#[uniffi::export]
fn fix_summary_message(summary: FixSummary) -> String {
    summary.message()
}

#[uniffi::export]
fn branch_name_slug(text: String) -> String {
    jayjay_core::branch_name_slug(&text)
}

#[uniffi::export]
fn ancestors_revset(change_id: String) -> String {
    jayjay_core::ancestors_revset(&change_id)
}

#[uniffi::export]
fn bookmark_filter_revset(name: String, remote: Option<String>) -> String {
    jayjay_core::bookmark_filter_revset(&name, remote.as_deref())
}

#[uniffi::export]
fn revset_filter(revset: String) -> jayjay_core::RevsetFilter {
    jayjay_core::RevsetFilter::of(&revset)
}

#[uniffi::export]
fn bookmark_filter_targets(
    bookmarks: Vec<jayjay_core::BookmarkInfo>,
) -> Vec<jayjay_core::BookmarkFilterTarget> {
    bookmarks
        .iter()
        .flat_map(jayjay_core::BookmarkFilterTarget::for_bookmark)
        .collect()
}

#[uniffi::export]
fn local_bookmark_filter_target(name: String) -> jayjay_core::BookmarkFilterTarget {
    jayjay_core::BookmarkFilterTarget::local(&name)
}

#[uniffi::export]
fn default_revset_preset() -> RevsetPreset {
    jayjay_core::default_revset_preset()
}

#[uniffi::export]
fn workspace_filter_revset(name: String) -> String {
    jayjay_core::workspace_filter_revset(&name)
}

#[uniffi::export]
fn typed_revset(
    text: String,
    bookmarks: Vec<jayjay_core::BookmarkInfo>,
    default_revset: RevsetPreset,
) -> String {
    jayjay_core::typed_revset(&text, &bookmarks, &default_revset)
}

#[uniffi::export]
fn revset_filter_apply(
    mut state: jayjay_core::RevsetFilterState,
    revset: String,
) -> jayjay_core::RevsetFilterState {
    state.apply(&revset);
    state
}

#[uniffi::export]
fn revset_filter_show_ancestors(
    mut state: jayjay_core::RevsetFilterState,
    change_id: String,
) -> jayjay_core::RevsetFilterState {
    state.show_ancestors(&change_id);
    state
}

#[uniffi::export]
fn recent_revsets(repository_path: String) -> Vec<String> {
    RecentRevsetStore::load().recent(Path::new(&repository_path))
}

#[uniffi::export]
fn recent_revsets_remember(repository_path: String, revset: String) -> Vec<String> {
    RecentRevsetStore::load().remember(Path::new(&repository_path), &revset)
}

#[uniffi::export]
fn recent_revsets_forget(repository_path: String, revset: String) -> Vec<String> {
    RecentRevsetStore::load().forget(Path::new(&repository_path), &revset)
}

#[uniffi::export]
fn revset_filter_back(mut state: jayjay_core::RevsetFilterState) -> jayjay_core::RevsetFilterState {
    state.back();
    state
}

#[uniffi::export]
fn revset_suggestions(
    query: String,
    state: jayjay_core::RevsetFilterState,
    bookmarks: Vec<jayjay_core::BookmarkInfo>,
    workspaces: Vec<String>,
) -> Vec<jayjay_core::RevsetSuggestion> {
    state.suggestions(&query, &bookmarks, &workspaces)
}

#[uniffi::export]
fn revset_presets() -> Vec<RevsetPreset> {
    jayjay_core::revset_presets()
}

#[uniffi::export]
fn revset_completions(
    text: String,
    cursor: u32,
    vocabulary: jayjay_core::RevsetVocabulary,
) -> Vec<jayjay_core::RevsetCompletion> {
    jayjay_core::revset_completions(&text, cursor, &vocabulary)
}

#[uniffi::export]
fn evolog_rows(
    entries: Vec<EvologEntry>,
    hide_snapshots: bool,
    expanded_runs: Vec<u32>,
) -> Vec<EvologRow> {
    jayjay_core::evolog_rows(&entries, hide_snapshots, &expanded_runs)
}

#[uniffi::export]
fn is_snapshot_operation(operation: String) -> bool {
    jayjay_core::is_snapshot_operation(&operation)
}

#[uniffi::export]
fn check_jj_environment() -> CliStatus {
    jayjay_core::check_jj_environment()
}

#[uniffi::export]
fn init_jj_git_repo(path: String) -> Result<(), JayError> {
    jayjay_core::init_jj_git_repo(&PathBuf::from(path))
}

#[uniffi::export]
fn clone_directory_name(source: String) -> String {
    CloneRequest::directory_name(&source)
}

#[uniffi::export]
fn default_clone_location(recent_repositories: Vec<String>) -> String {
    CloneRequest::default_location(&recent_repositories)
        .display()
        .to_string()
}

#[uniffi::export]
fn clone_location(folder: String, name: String) -> String {
    CloneRequest::location(&folder, &name)
}

#[derive(uniffi::Record)]
pub struct CloneLocationParts {
    pub folder: String,
    pub name: String,
}

#[uniffi::export]
fn split_clone_location(location: String) -> CloneLocationParts {
    let (folder, name) = CloneRequest::split_location(&location);
    CloneLocationParts {
        folder: folder.to_owned(),
        name: name.to_owned(),
    }
}

/// Returns the new repository's path.
#[uniffi::export]
fn clone_git_repo(
    source: String,
    location: String,
    sync: Arc<JayJaySyncToken>,
) -> Result<String, JayError> {
    let request = CloneRequest::new(&source, &location)?;
    request.run(&sync.inner)?;
    Ok(request.destination.display().to_string())
}

#[uniffi::export]
fn is_valid_bookmark_name(name: String) -> bool {
    jayjay_core::is_valid_bookmark_name(&name)
}

#[uniffi::export]
fn is_valid_workspace_name(name: String) -> bool {
    jayjay_core::is_valid_workspace_name(&name)
}

#[uniffi::export]
fn enclosing_workspace_root(path: String) -> Option<String> {
    jayjay_core::enclosing_workspace_root(Path::new(&path))
        .and_then(|root| root.into_os_string().into_string().ok())
}

#[uniffi::export]
fn workspace_primary_root(path: String) -> Option<String> {
    jayjay_core::workspace_primary_root(&path)
}

#[uniffi::export]
fn jj_command_body(query: String) -> Option<String> {
    JjCommand::from_palette_query(&query).map(JjCommand::into_raw)
}

#[uniffi::export]
fn review_canonical_snapshot(old_content: String, new_content: String) -> ReviewFileSnapshot {
    jayjay_core::diff::canonical_review_snapshot(&old_content, &new_content)
}

#[uniffi::export]
fn review_snapshot_from_diff_hunk(hunk: DiffHunk) -> ReviewFileSnapshot {
    review_snapshot_from_hunk(&hunk)
}

#[uniffi::export]
fn review_display_group_map_from_diff_hunk(
    hunk: DiffHunk,
    ignore_whitespace: bool,
) -> Vec<Vec<u32>> {
    review_display_group_map_from_hunk(&hunk, ignore_whitespace)
}

#[uniffi::export]
fn parse_jj_command_args(command: String) -> Option<Vec<String>> {
    JjCommand::new(command).parse_args()
}

/// Walks the same fallback paths jj does; macOS `.app` bundles get stripped PATH from launchd, so this avoids relying on shell PATH.
#[uniffi::export]
fn find_binary(name: String) -> Option<String> {
    jayjay_core::find_existing_binary(&name)
}

#[uniffi::export]
fn login_shell_path() -> Option<String> {
    jayjay_core::login_shell_path()
}

#[uniffi::export]
fn login_shell() -> String {
    jayjay_core::login_shell()
}

#[uniffi::export]
fn open_in_editor(
    repo_path: String,
    file_path: String,
    line: Option<u32>,
    external_editor: String,
    custom_editor_command: String,
    terminal: String,
    custom_terminal_command: String,
) -> bool {
    jayjay_core::open_in_editor(
        &repo_path,
        &file_path,
        line,
        &ToolsConfig {
            external_editor,
            custom_editor_command,
            terminal,
            custom_terminal_command,
        },
    )
}

#[uniffi::export]
fn editor_opens_at_line(external_editor: String) -> bool {
    jayjay_core::editor_opens_at_line(&external_editor)
}

#[uniffi::export]
fn open_in_terminal(
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

#[derive(uniffi::Object)]
pub struct JayJaySyncToken {
    pub(crate) inner: SyncToken,
}

#[uniffi::export]
impl JayJaySyncToken {
    #[uniffi::constructor]
    fn standalone() -> Arc<Self> {
        Arc::new(Self {
            inner: SyncToken::standalone(),
        })
    }

    fn cancel(&self) {
        self.inner.cancel();
    }
}

#[uniffi::export]
impl JayJayRepo {
    #[uniffi::constructor]
    fn open(path: String) -> Result<Arc<Self>, JayError> {
        let repo = Repo::open(&PathBuf::from(&path))?;
        Ok(Arc::new(Self { inner: repo }))
    }

    fn repository_context(&self) -> Result<jayjay_core::RepositoryContext, JayError> {
        self.inner.repository_context()
    }

    fn is_remote(&self) -> bool {
        self.inner.is_remote()
    }

    fn ssh_state(&self) -> jayjay_core::ssh::ConnectionState {
        self.inner.connection_state()
    }

    fn ssh_platform(&self) -> Option<jayjay_core::ssh::HostPlatform> {
        self.inner.remote_platform()
    }

    fn ssh_error(&self) -> Option<String> {
        self.inner.connection_error()
    }

    fn reconnect_ssh(&self) -> Result<(), JayError> {
        self.inner.reconnect()
    }

    fn disconnect_ssh(&self) -> Result<(), JayError> {
        self.inner.disconnect()
    }

    fn poll_remote_changes(&self) -> Result<bool, JayError> {
        self.inner.poll_remote_changes()
    }

    fn primary_root(&self) -> String {
        self.inner.primary_root().to_string_lossy().into_owned()
    }

    fn path(&self) -> String {
        self.inner.path().display().to_string()
    }

    fn run_jj_command(&self, command: String) -> Result<JjCommandResult, JayError> {
        JjCommand::new(command).run_in_repo(&self.inner)
    }

    fn refresh_working_copy(&self) -> Result<(), JayError> {
        self.inner.refresh_working_copy()
    }

    fn working_copy_is_large(&self) -> bool {
        self.inner.working_copy_is_large()
    }

    fn has_unignored_working_copy_paths(&self, paths: Vec<String>) -> Result<bool, JayError> {
        self.inner.has_unignored_working_copy_paths(&paths)
    }

    fn log(&self, revset: String) -> Result<Vec<ChangeInfo>, JayError> {
        self.inner.log(&revset)
    }

    fn check_revset(&self, revset: String) -> Result<(), JayError> {
        self.inner.check_revset(&revset)
    }

    fn default_revset_preset(&self) -> RevsetPreset {
        self.inner.default_revset_preset()
    }

    fn log_graph(&self, revset: String) -> Result<Vec<GraphEntry>, JayError> {
        self.inner.log_graph(&revset)
    }

    /// One crossing for the graph and its layout, so the shell never sends the entries back.
    fn log_graph_with_layout(&self, revset: String) -> Result<GraphWithLayout, JayError> {
        let entries = self.inner.log_graph(&revset)?;
        let dag_layout = DagLayout::compute(&entries);
        let selection = Arc::new(DagSelectionGraph::from_entries(&entries, &dag_layout));
        let layout = layout_data(dag_layout);
        Ok(GraphWithLayout {
            entries,
            layout,
            selection,
        })
    }

    fn show(&self, rev: String) -> Result<ChangeDetail, JayError> {
        self.inner.show(&rev)
    }

    /// Fast: file list without content.
    fn show_summary(&self, rev: String) -> Result<ChangeDetail, JayError> {
        self.inner.show_summary(&rev)
    }

    fn show_file(&self, rev: String, path: String) -> Result<DiffHunk, JayError> {
        self.inner.show_file(&rev, &path)
    }

    fn show_file_raw(&self, rev: String, path: String) -> Result<DiffHunk, JayError> {
        self.inner.show_file_raw(&rev, &path)
    }

    fn show_file_rename(
        &self,
        rev: String,
        old_path: String,
        new_path: String,
    ) -> Result<DiffHunk, JayError> {
        self.inner.show_file_rename(&rev, &old_path, &new_path)
    }

    fn show_file_rename_raw(
        &self,
        rev: String,
        old_path: String,
        new_path: String,
    ) -> Result<DiffHunk, JayError> {
        self.inner.show_file_rename_raw(&rev, &old_path, &new_path)
    }

    fn review_file_snapshot(
        &self,
        rev: String,
        path: String,
        old_path: Option<String>,
    ) -> Result<ReviewFileSnapshot, JayError> {
        self.inner
            .review_file_snapshot(&rev, &path, old_path.as_deref())
    }

    /// Fast: file list between two arbitrary revisions (no content).
    fn interdiff_summary(
        &self,
        from_rev: String,
        to_rev: String,
    ) -> Result<ChangeDetail, JayError> {
        self.inner.interdiff_summary(&from_rev, &to_rev)
    }

    fn interdiff_file(
        &self,
        from_rev: String,
        to_rev: String,
        path: String,
    ) -> Result<DiffHunk, JayError> {
        self.inner.interdiff_file(&from_rev, &to_rev, &path)
    }

    fn interdiff_file_raw(
        &self,
        from_rev: String,
        to_rev: String,
        path: String,
    ) -> Result<DiffHunk, JayError> {
        self.inner.interdiff_file_raw(&from_rev, &to_rev, &path)
    }

    fn interdiff_file_rename_with_mode(
        &self,
        from_rev: String,
        to_rev: String,
        old_path: String,
        new_path: String,
        projection_mode: DiffProjectionMode,
    ) -> Result<DiffHunk, JayError> {
        self.inner.interdiff_file_rename_with_mode(
            &from_rev,
            &to_rev,
            &old_path,
            &new_path,
            projection_mode,
        )
    }

    fn workspace_list(&self) -> Result<Vec<WorkspaceInfo>, JayError> {
        self.inner.workspace_list()
    }

    fn overview_snapshot(&self) -> Result<OverviewSnapshot, JayError> {
        self.inner.overview_snapshot()
    }

    fn workspace_add(&self, dest: String, name: String, rev: String) -> Result<String, JayError> {
        self.inner.workspace_add(&dest, &name, &rev)
    }

    fn repository_store_path(&self) -> String {
        self.inner
            .repository_store_path()
            .to_string_lossy()
            .into_owned()
    }

    fn workspace_name(&self) -> String {
        self.inner.workspace_name().to_owned()
    }

    fn sync_token(&self) -> Arc<JayJaySyncToken> {
        Arc::new(JayJaySyncToken {
            inner: self.inner.sync_token(),
        })
    }

    fn cancel_running_jj_processes(&self) {
        self.inner.cancel_running_jj_processes();
    }

    fn update_stale_workspace(&self) -> Result<(), JayError> {
        self.inner.update_stale_workspace()
    }

    fn workspace_forget(
        &self,
        name: String,
        expected_root: Option<String>,
    ) -> Result<(), JayError> {
        self.inner.workspace_forget(&name, expected_root.as_deref())
    }

    fn workspace_forget_and_delete(
        &self,
        name: String,
        expected_root: String,
    ) -> Result<Option<String>, JayError> {
        self.inner
            .workspace_forget_and_delete(&name, &expected_root)
    }

    fn workspace_presence(&self) -> WorkspacePresence {
        self.inner.workspace_presence()
    }

    fn pull_request_info(&self, bookmark: String) -> Option<PrInfo> {
        self.inner.pull_request_info(&bookmark)
    }

    fn pull_request_open_url(&self, bookmark: String) -> Result<String, JayError> {
        self.inner.pull_request_open_url(&bookmark)
    }

    fn pull_request_import_preview(
        &self,
        url: String,
        sync: Arc<JayJaySyncToken>,
    ) -> Result<PullRequestImportPreview, JayError> {
        self.inner.pull_request_import_preview(&url, &sync.inner)
    }

    fn pull_request_import(
        &self,
        url: String,
        previewed_head_commit_id: String,
        workspace_name: String,
        workspace_dest: String,
        sync: Arc<JayJaySyncToken>,
    ) -> Result<String, JayError> {
        self.inner.pull_request_import(
            &url,
            &previewed_head_commit_id,
            &workspace_name,
            &workspace_dest,
            &sync.inner,
        )
    }

    fn trunk_name(&self) -> String {
        self.inner.trunk_name()
    }

    fn pr_host_name(&self) -> Option<String> {
        self.inner.pr_host_name()
    }

    fn diff_stats(&self, rev: String) -> Result<DiffStats, JayError> {
        self.inner.diff_stats(&rev)
    }

    fn diff_file_stats(
        &self,
        rev: String,
        ignore_whitespace: bool,
    ) -> Result<Vec<FileDiffStats>, JayError> {
        self.inner.diff_file_stats(&rev, ignore_whitespace)
    }

    fn annotate_file(&self, rev: String, path: String) -> Result<Vec<AnnotationLine>, JayError> {
        self.inner.annotate_file(&rev, &path)
    }

    fn file_history(&self, path: String) -> Result<Vec<ChangeInfo>, JayError> {
        self.inner.file_history(&path)
    }

    fn evolog(&self, rev: String) -> Result<Vec<EvologEntry>, JayError> {
        self.inner.evolog(&rev)
    }

    fn resolve_list(&self, rev: String) -> Result<Vec<String>, JayError> {
        self.inner.resolve_list(&rev)
    }

    fn resolve_use_ours(&self, rev: String, path: String) -> Result<(), JayError> {
        self.inner.resolve_use_ours(&rev, &path)
    }

    fn resolve_use_theirs(&self, rev: String, path: String) -> Result<(), JayError> {
        self.inner.resolve_use_theirs(&rev, &path)
    }

    fn resolve_with_tool(&self, rev: String, path: String, tool: String) -> Result<(), JayError> {
        self.inner.resolve_with_tool(&rev, &path, &tool)
    }

    fn resolve_with_merge_editor(&self, rev: String, path: String) -> Result<(), JayError> {
        self.inner.resolve_with_merge_editor(&rev, &path)
    }

    fn merge_editor_name(&self) -> Option<String> {
        self.inner.merge_editor_name()
    }

    fn conflict_editor(&self, rev: String, path: String) -> Result<ConflictEditorData, JayError> {
        self.inner.conflict_editor(&rev, &path)
    }

    fn apply_conflict_editor(
        &self,
        rev: String,
        data: ConflictEditorData,
        content: String,
    ) -> Result<(), JayError> {
        self.inner.apply_conflict_editor(&rev, &data, &content)
    }

    fn file_content(&self, rev: String, path: String) -> Result<String, JayError> {
        self.inner.file_content(&rev, &path)
    }

    fn working_copy_file_editor(&self, path: String) -> Result<FileEditorData, JayError> {
        self.inner.working_copy_file_editor(&path)
    }

    fn apply_working_copy_file_editor(
        &self,
        data: FileEditorData,
        content: String,
    ) -> Result<(), JayError> {
        self.inner.apply_working_copy_file_editor(&data, &content)
    }

    fn restore_files(&self, rev: String, paths: Vec<String>) -> Result<(), JayError> {
        self.inner.restore_files(&rev, None, &paths)
    }

    fn restore_version(&self, rev: String, version: String) -> Result<(), JayError> {
        self.inner.restore_version(&rev, &version)
    }

    fn move_to_working_copy(&self, rev: String, paths: Vec<String>) -> Result<(), JayError> {
        self.inner.move_to_working_copy(&rev, &paths)
    }

    fn delete_files(&self, paths: Vec<String>) -> Result<(), JayError> {
        self.inner.delete_files(&paths)
    }

    fn ignore_and_untrack(&self, paths: Vec<String>) -> Result<(), JayError> {
        self.inner.ignore_and_untrack(&paths)
    }

    fn split(
        &self,
        rev: String,
        paths: Vec<String>,
        message: String,
        parallel: bool,
    ) -> Result<(), JayError> {
        self.inner.split(&rev, &paths, &message, parallel)
    }

    fn describe(&self, rev: String, message: String) -> Result<(), JayError> {
        self.inner.describe(&rev, &message)
    }

    fn new_change(&self, parent: String, message: String) -> Result<(), JayError> {
        self.inner.new_change(&parent, &message)
    }

    fn new_change_inserted(
        &self,
        rev: String,
        position: InsertPosition,
        message: String,
    ) -> Result<(), JayError> {
        self.inner.new_change_inserted(&rev, position, &message)
    }

    fn squash(&self, rev: String, into_rev: Option<String>) -> Result<(), JayError> {
        self.inner.squash(&rev, into_rev.as_deref())
    }

    fn squash_many(&self, revs: Vec<String>) -> Result<String, JayError> {
        self.inner.squash_many(&revs)
    }

    fn parallelize(&self, revs: Vec<String>) -> Result<MutationEffect, JayError> {
        self.inner.parallelize(&revs)
    }

    fn fix_unavailable_reason(&self) -> Option<String> {
        self.inner.fix_unavailable_reason()
    }

    fn fix(&self, revs: Vec<String>) -> Result<FixSummary, JayError> {
        self.inner.fix(&revs)
    }

    fn edit(&self, rev: String) -> Result<(), JayError> {
        self.inner.edit(&rev)
    }

    fn bisect_state(&self) -> Option<BisectState> {
        self.inner.bisect_state()
    }

    fn start_bisect(&self, range: String) -> Result<BisectState, JayError> {
        self.inner.start_bisect(&range)
    }

    fn start_bisect_between(&self, first: String, second: String) -> Result<BisectState, JayError> {
        self.inner.start_bisect_between(&first, &second)
    }

    fn bisect_mark(
        &self,
        commit_id: String,
        verdict: BisectVerdict,
    ) -> Result<BisectState, JayError> {
        self.inner.bisect_mark(&commit_id, verdict)
    }

    fn bisect_stop(&self) -> Result<(), JayError> {
        self.inner.bisect_stop()
    }

    fn shutdown_bisect(&self) -> Result<(), JayError> {
        self.inner.shutdown_bisect()
    }

    fn bisect_restart(&self) -> Result<BisectState, JayError> {
        self.inner.bisect_restart()
    }

    fn bisect_dismiss(&self) -> Result<(), JayError> {
        self.inner.bisect_dismiss()
    }

    fn bisect_run_step(
        &self,
        command: String,
        sync: Arc<JayJaySyncToken>,
    ) -> Result<Option<BisectState>, JayError> {
        self.inner.bisect_run_step(&command, &sync.inner)
    }

    fn bisect_run_to_end(
        &self,
        command: String,
        sync: Arc<JayJaySyncToken>,
    ) -> Result<Option<BisectState>, JayError> {
        self.inner.bisect_run_to_end(&command, &sync.inner)
    }

    fn absorb(&self, rev: String) -> Result<MutationEffect, JayError> {
        self.inner.absorb(&rev)
    }

    fn revert_change(&self, rev: String) -> Result<String, JayError> {
        self.inner.revert_change(&rev)
    }

    fn merge(&self, parent_revs: Vec<String>) -> Result<(), JayError> {
        self.inner.merge(&parent_revs)
    }

    fn duplicate(&self, rev: String) -> Result<String, JayError> {
        self.inner.duplicate(&rev)
    }

    fn converge_preview(&self, rev: String) -> Result<ConvergePlan, JayError> {
        self.inner.converge_preview(&rev)
    }

    fn converge_apply(
        &self,
        plan: ConvergePlan,
        choices: ConvergeChoices,
    ) -> Result<ConvergeOutcome, JayError> {
        self.inner.converge_apply(&plan, &choices)
    }

    fn abandon(&self, rev: String) -> Result<(), JayError> {
        self.inner.abandon(&rev)
    }

    fn abandon_many(&self, revs: Vec<String>) -> Result<(), JayError> {
        self.inner.abandon_many(&revs)
    }

    fn rebase(&self, rev: String, dest: String, mode: RebaseMode) -> Result<String, JayError> {
        self.inner.rebase(&rev, &dest, mode)
    }

    fn rebase_many(&self, revs: Vec<String>, dest: String) -> Result<(), JayError> {
        self.inner.rebase_many(&revs, &dest)
    }

    fn list_bookmarks(&self) -> Result<Vec<BookmarkInfo>, JayError> {
        self.inner.list_bookmarks()
    }

    fn list_tags(&self) -> Result<Vec<TagInfo>, JayError> {
        self.inner.list_tags()
    }

    fn revset_vocabulary(&self, bookmarks: Vec<BookmarkInfo>) -> jayjay_core::RevsetVocabulary {
        self.inner.revset_vocabulary(&bookmarks)
    }

    fn create_bookmark(&self, name: String, rev: String) -> Result<(), JayError> {
        self.inner.create_bookmark(&name, &rev)
    }

    fn move_bookmark(&self, name: String, to_rev: String) -> Result<(), JayError> {
        self.inner.move_bookmark(&name, &to_rev)
    }

    fn delete_bookmark(&self, name: String) -> Result<(), JayError> {
        self.inner.delete_bookmark(&name)
    }

    fn remove_bookmark_from_rev(&self, name: String, rev: String) -> Result<(), JayError> {
        self.inner.remove_bookmark_from_rev(&name, &rev)
    }

    fn forget_bookmark(&self, name: String) -> Result<(), JayError> {
        self.inner.forget_bookmark(&name)
    }

    fn detect_stack(&self, base_rev: String, tip_rev: String) -> Result<Stack, JayError> {
        self.inner.detect_stack(&base_rev, &tip_rev)
    }

    fn submit_stack(&self, layers: Vec<SubmitStackLayer>) -> Result<StackedPrResult, JayError> {
        self.inner.submit_stack(layers)
    }

    fn rename_bookmark(&self, old_name: String, new_name: String) -> Result<(), JayError> {
        self.inner.rename_bookmark(&old_name, &new_name)
    }

    fn track_bookmark(&self, name: String, remote: String) -> Result<(), JayError> {
        self.inner.track_bookmark(&name, &remote)
    }

    fn forget_stale_bookmarks(&self) -> Result<u32, JayError> {
        self.inner.forget_stale_bookmarks()
    }

    fn git_push(&self, bookmark: String, sync: Arc<JayJaySyncToken>) -> Result<String, JayError> {
        self.inner.git_push(&bookmark, &sync.inner)
    }

    fn create_tag(&self, name: String, rev: String) -> Result<(), JayError> {
        self.inner.create_tag(&name, &rev)
    }

    fn delete_tag(&self, name: String) -> Result<(), JayError> {
        self.inner.delete_tag(&name)
    }

    fn delete_tag_and_push(
        &self,
        name: String,
        sync: Arc<JayJaySyncToken>,
    ) -> Result<String, JayError> {
        self.inner.delete_tag_and_push(&name, &sync.inner)
    }

    fn git_push_tag(&self, tag: String, sync: Arc<JayJaySyncToken>) -> Result<String, JayError> {
        self.inner.git_push_tag(&tag, &sync.inner)
    }

    fn remote_web_url(&self) -> Option<String> {
        self.inner.remote_web_url()
    }

    fn git_fetch(&self, sync: Arc<JayJaySyncToken>) -> Result<FetchResult, JayError> {
        self.inner.git_fetch(&sync.inner)
    }

    fn upload(&self, revision: String, sync: Arc<JayJaySyncToken>) -> Result<String, JayError> {
        self.inner.upload(&revision, &sync.inner)
    }

    fn update_from_gerrit(
        &self,
        revision: String,
        sync: Arc<JayJaySyncToken>,
    ) -> Result<String, JayError> {
        self.inner.update_from_gerrit(&revision, &sync.inner)
    }

    fn git_pull_bookmark(
        &self,
        bookmark: String,
        sync: Arc<JayJaySyncToken>,
    ) -> Result<FetchResult, JayError> {
        self.inner.git_pull_bookmark(&bookmark, &sync.inner)
    }

    fn jj_commit(&self, message: String) -> Result<(), JayError> {
        self.inner.jj_commit(&message)
    }

    fn submodule_statuses(&self) -> Result<Vec<GitSubmoduleStatus>, JayError> {
        self.inner.submodule_statuses()
    }

    fn commit_safe_submodule_updates(
        &self,
        message: String,
        paths: Vec<String>,
    ) -> Result<String, JayError> {
        self.inner.commit_safe_submodule_updates(&message, &paths)
    }

    fn git_lfs_paths(&self, paths: Vec<String>) -> Result<Vec<String>, JayError> {
        self.inner.git_lfs_paths(&paths)
    }

    fn diff_excerpt(&self) -> Result<Option<DiffExcerpt>, JayError> {
        self.inner.diff_excerpt()
    }

    fn op_log(&self) -> Result<Vec<OpLogEntry>, JayError> {
        self.inner.op_log()
    }

    fn op_restore(&self, op_id: String) -> Result<(), JayError> {
        self.inner.op_restore(&op_id)
    }

    fn undo(&self, operation_id: String) -> Result<Option<String>, JayError> {
        self.inner.undo(&operation_id)
    }

    fn redo(&self, operation_id: String) -> Result<Option<String>, JayError> {
        self.inner.redo(&operation_id)
    }

    fn undo_state(&self) -> UndoState {
        self.inner.undo_state()
    }

    fn review_notes(
        &self,
        rev: String,
        include_resolved: bool,
    ) -> Result<Vec<ReviewNoteStatus>, JayError> {
        self.inner.review_notes(&rev, include_resolved)
    }

    fn is_at_operation_head(&self) -> Result<bool, JayError> {
        self.inner.is_at_operation_head()
    }

    fn current_operation_description(&self) -> String {
        self.inner.current_operation_description()
    }

    fn compute_native_diff(
        &self,
        path: String,
        old_content: String,
        new_content: String,
        ignore_whitespace: bool,
    ) -> FileDiff {
        let mut diff =
            diff::compute_file_diff(&path, &old_content, &new_content, ignore_whitespace);
        diff.detect_moves(&old_content, &new_content);
        diff
    }

    fn compute_native_diff_full(
        &self,
        path: String,
        old_content: String,
        new_content: String,
        ignore_whitespace: bool,
    ) -> FileDiff {
        diff::compute_file_diff_full(&path, &old_content, &new_content, ignore_whitespace)
    }

    fn compute_native_diff_full_plain(
        &self,
        path: String,
        old_content: String,
        new_content: String,
        ignore_whitespace: bool,
    ) -> FileDiff {
        diff::compute_file_diff_full_plain(&path, &old_content, &new_content, ignore_whitespace)
    }

    fn collapse_diff_with_mapping(&self, diff: FileDiff) -> CollapsedDiff {
        diff::collapse_context_with_mapping(&diff)
    }

    fn apply_diff_selection(
        &self,
        rev: String,
        destination: DiffEditDestination,
        selections: Vec<DiffEditFileSelection>,
        message: String,
        ignore_whitespace: bool,
    ) -> Result<(), JayError> {
        self.inner
            .apply_diff_selection(&rev, destination, &selections, &message, ignore_whitespace)
    }
}
