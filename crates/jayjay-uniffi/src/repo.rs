use std::path::PathBuf;
use std::sync::Arc;

use jayjay_core::{
    AnnotationLine, BookmarkInfo, ChangeDetail, ChangeInfo, CliStatus, ConflictEditorData,
    DiffEditDestination, DiffEditFileSelection, DiffHunk, DiffStats, EvologEntry, EvologRow,
    FetchResult, FileDiffStats, FileEditorData, GitSubmoduleStatus, GraphEntry, JjCommand,
    JjCommandResult, OpLogEntry, PrInfo, Repo, ReviewNoteOutputFormat, RevsetPreset, Stack,
    StackedPrResult, SubmitStackLayer, SyncToken, ToolsConfig, WorkspaceInfo, WorkspacePresence,
    diff::{self, CollapsedDiff, FileDiff, ReviewFileSnapshot},
    review_display_group_map_from_hunk, review_snapshot_from_hunk,
};
use jayjay_primitives::{
    NoteAnchor, NoteEntry, NoteSide, ReviewFileRollup, ReviewGroupState, ReviewNoteStatus,
    ReviewStoreSummary,
};
use jayjay_review::ReviewStore;

use crate::error::JayJayError;

#[uniffi::export]
fn detect_ai_provider() -> String {
    jayjay_core::detect_ai_provider()
}

#[uniffi::export]
fn commit_message_prompt() -> String {
    jayjay_core::COMMIT_MESSAGE_PROMPT.to_owned()
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
fn revset_presets() -> Vec<RevsetPreset> {
    jayjay_core::revset_presets().to_vec()
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
fn init_jj_git_repo(path: String) -> Result<(), JayJayError> {
    jayjay_core::init_jj_git_repo(&PathBuf::from(path)).map_err(JayJayError::from)
}

#[uniffi::export]
fn review_notes_output(
    repo_path: String,
    format: String,
    include_resolved: bool,
) -> Result<String, JayJayError> {
    let format = review_note_output_format(&format)?;
    jayjay_core::review_notes_output(&PathBuf::from(repo_path), format, include_resolved)
        .map_err(JayJayError::from)
}

#[uniffi::export]
fn add_review_note(
    repo_path: String,
    file: String,
    line: u32,
    side: String,
    message: String,
) -> Result<String, JayJayError> {
    let side = review_note_side(&side)?;
    jayjay_core::add_review_note(&PathBuf::from(repo_path), &file, line, side, &message)
        .map_err(JayJayError::from)
}

#[uniffi::export]
fn resolve_review_note(repo_path: String, id: String) -> Result<String, JayJayError> {
    jayjay_core::resolve_review_note(&PathBuf::from(repo_path), &id).map_err(JayJayError::from)
}

fn review_note_output_format(format: &str) -> Result<ReviewNoteOutputFormat, JayJayError> {
    match format {
        "text" => Ok(ReviewNoteOutputFormat::Text),
        "json" => Ok(ReviewNoteOutputFormat::Json),
        _ => Err(JayJayError::Internal {
            message: format!("unsupported review notes format: {format}"),
        }),
    }
}

fn review_note_side(side: &str) -> Result<NoteSide, JayJayError> {
    match side {
        "new" => Ok(NoteSide::New),
        "old" => Ok(NoteSide::Old),
        _ => Err(JayJayError::Internal {
            message: format!("unsupported review note side: {side}"),
        }),
    }
}

#[uniffi::export]
fn check_gh_environment() -> CliStatus {
    jayjay_core::check_gh_environment()
}

#[uniffi::export]
fn check_glab_environment() -> CliStatus {
    jayjay_core::check_glab_environment()
}

#[uniffi::export]
fn check_origin_environment() -> CliStatus {
    jayjay_core::check_origin_environment()
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
fn workspace_primary_root(path: String) -> Option<String> {
    jayjay_core::workspace_primary_root(&path)
}

#[uniffi::export]
fn jj_command_body(query: String) -> Option<String> {
    JjCommand::from_palette_query(&query).map(JjCommand::into_raw)
}

/// Canonical review-store path, so the SwiftUI shell persists to the same file as the Rust core.
#[uniffi::export]
fn review_store_path() -> Option<String> {
    jayjay_review::ReviewStore::store_path().map(|p| p.to_string_lossy().into_owned())
}

fn review_store(store_path: Option<String>) -> ReviewStore {
    match store_path.filter(|path| !path.is_empty()) {
        Some(path) => ReviewStore::load_from(std::path::PathBuf::from(path)),
        None => ReviewStore::load(),
    }
}

#[uniffi::export]
fn review_is_reviewed(
    change_id: String,
    path: String,
    identity: String,
    store_path: Option<String>,
) -> bool {
    review_store(store_path).is_reviewed(&change_id, &path, &identity)
}

#[uniffi::export]
fn review_mark_reviewed(
    change_id: String,
    path: String,
    identity: String,
    snapshot: Option<ReviewFileSnapshot>,
    store_path: Option<String>,
) {
    review_store(store_path).mark_reviewed_snapshot(
        &change_id,
        &path,
        &identity,
        snapshot.as_ref(),
    );
}

#[uniffi::export]
fn review_mark_unreviewed(change_id: String, path: String, store_path: Option<String>) {
    review_store(store_path).mark_unreviewed(&change_id, &path);
}

#[uniffi::export]
fn review_toggle_reviewed(
    change_id: String,
    path: String,
    identity: String,
    snapshot: Option<ReviewFileSnapshot>,
    store_path: Option<String>,
) {
    let mut store = review_store(store_path);
    match snapshot.as_ref() {
        Some(snapshot) => store.toggle_snapshot(&change_id, &path, &identity, snapshot),
        None => store.toggle(&change_id, &path, &identity),
    }
}

/// Batch mark lookup with one store read, so refreshes see external writers (other windows, GPUI, the CLI) without a per-file disk load.
#[uniffi::export]
fn review_reviewed_paths(
    change_id: String,
    paths: Vec<String>,
    identities: Vec<String>,
    store_path: Option<String>,
) -> Vec<String> {
    let store = review_store(store_path);
    paths
        .into_iter()
        .zip(identities)
        .filter_map(|(path, identity)| {
            store
                .is_reviewed(&change_id, &path, &identity)
                .then_some(path)
        })
        .collect()
}

#[uniffi::export]
fn review_is_hunk_reviewed(
    change_id: String,
    path: String,
    identity: String,
    hunk_index: u32,
    store_path: Option<String>,
) -> bool {
    review_store(store_path).is_hunk_reviewed(&change_id, &path, &identity, hunk_index)
}

/// One file's marks in a single call, so shells can cache them and answer per-gutter-line lookups without re-reading the store from disk each time.
#[uniffi::export]
fn review_file_marks(
    change_id: String,
    path: String,
    identity: String,
    snapshot: Option<ReviewFileSnapshot>,
    store_path: Option<String>,
) -> jayjay_review::ReviewFileMarks {
    review_store(store_path).file_marks(&change_id, &path, &identity, snapshot.as_ref())
}

#[uniffi::export]
fn review_file_rollups(
    change_id: String,
    paths: Vec<String>,
    identities: Vec<String>,
    snapshots: Vec<Option<ReviewFileSnapshot>>,
    store_path: Option<String>,
) -> Vec<ReviewFileRollup> {
    review_store(store_path).file_rollups(&change_id, &paths, &identities, &snapshots)
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
fn review_display_hunk_states(
    change_id: String,
    path: String,
    identity: String,
    snapshot: ReviewFileSnapshot,
    mapping: Vec<Vec<u32>>,
    store_path: Option<String>,
) -> Vec<ReviewGroupState> {
    review_store(store_path).display_hunk_states(&change_id, &path, &identity, &snapshot, &mapping)
}

#[uniffi::export]
fn review_toggle_display_hunk_snapshot(
    change_id: String,
    path: String,
    identity: String,
    snapshot: ReviewFileSnapshot,
    mapping: Vec<Vec<u32>>,
    display_index: u32,
    store_path: Option<String>,
) {
    review_store(store_path).toggle_display_group_snapshot(
        &change_id,
        &path,
        &identity,
        &snapshot,
        &mapping,
        display_index,
    );
}

#[uniffi::export]
fn review_mark_hunk_reviewed(
    change_id: String,
    path: String,
    identity: String,
    hunk_index: u32,
    snapshot: Option<ReviewFileSnapshot>,
    store_path: Option<String>,
) {
    review_store(store_path).mark_hunk_reviewed_snapshot(
        &change_id,
        &path,
        &identity,
        snapshot.as_ref(),
        hunk_index,
    );
}

#[uniffi::export]
fn review_mark_hunk_unreviewed(
    change_id: String,
    path: String,
    hunk_index: u32,
    identity: Option<String>,
    snapshot: Option<ReviewFileSnapshot>,
    store_path: Option<String>,
) {
    let mut store = review_store(store_path);
    match (identity.as_deref(), snapshot.as_ref()) {
        (Some(identity), Some(snapshot)) => {
            store.mark_hunk_unreviewed_snapshot(&change_id, &path, identity, snapshot, hunk_index)
        }
        _ => store.mark_hunk_unreviewed(&change_id, &path, hunk_index),
    }
}

#[uniffi::export]
fn review_toggle_hunk(
    change_id: String,
    path: String,
    identity: String,
    hunk_index: u32,
    snapshot: Option<ReviewFileSnapshot>,
    store_path: Option<String>,
) {
    let mut store = review_store(store_path);
    match snapshot.as_ref() {
        Some(snapshot) => {
            store.toggle_hunk_snapshot(&change_id, &path, &identity, snapshot, hunk_index)
        }
        None => store.toggle_hunk(&change_id, &path, &identity, hunk_index),
    }
}

#[uniffi::export]
fn review_set_reviewed_hunks(
    change_id: String,
    path: String,
    identity: String,
    hunk_indices: Vec<u32>,
    snapshot: Option<ReviewFileSnapshot>,
    store_path: Option<String>,
) {
    review_store(store_path).set_reviewed_hunks_snapshot(
        &change_id,
        &path,
        &identity,
        snapshot.as_ref(),
        hunk_indices,
    );
}

#[uniffi::export]
fn review_store_summary(store_path: Option<String>) -> ReviewStoreSummary {
    review_store(store_path).summary()
}

#[uniffi::export]
fn review_clear_all(store_path: Option<String>) {
    review_store(store_path).clear_all();
}

#[uniffi::export]
fn review_clear_change(change_id: String, store_path: Option<String>) {
    review_store(store_path).clear_change(&change_id);
}

#[uniffi::export]
fn review_list_notes(
    change_id: String,
    include_resolved: bool,
    store_path: Option<String>,
) -> Vec<NoteEntry> {
    review_store(store_path).list_notes(&change_id, include_resolved)
}

#[uniffi::export]
fn review_add_note(anchor: NoteAnchor, body: String, store_path: Option<String>) -> NoteEntry {
    review_store(store_path).add_note(anchor, &body)
}

#[uniffi::export]
fn review_update_note(id: String, body: String, store_path: Option<String>) -> Option<NoteEntry> {
    review_store(store_path).update_note(&id, &body)
}

#[uniffi::export]
fn review_delete_note(id: String, store_path: Option<String>) -> bool {
    review_store(store_path).delete_note(&id)
}

#[uniffi::export]
fn review_resolve_note(id: String, store_path: Option<String>) -> Option<NoteEntry> {
    review_store(store_path).resolve_note(&id)
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
    inner: SyncToken,
}

#[uniffi::export]
impl JayJaySyncToken {
    fn cancel(&self) {
        self.inner.cancel();
    }
}

#[uniffi::export]
impl JayJayRepo {
    #[uniffi::constructor]
    fn open(path: String) -> Result<Arc<Self>, JayJayError> {
        let repo = Repo::open(&PathBuf::from(&path))?;
        Ok(Arc::new(Self { inner: repo }))
    }

    fn path(&self) -> String {
        self.inner.path().display().to_string()
    }

    fn run_jj_command(&self, command: String) -> Result<JjCommandResult, JayJayError> {
        Ok(JjCommand::new(command).run_in_repo(&self.inner)?)
    }

    fn refresh_working_copy(&self) -> Result<(), JayJayError> {
        Ok(self.inner.refresh_working_copy()?)
    }

    fn working_copy_is_large(&self) -> bool {
        self.inner.working_copy_is_large()
    }

    fn has_unignored_working_copy_paths(&self, paths: Vec<String>) -> Result<bool, JayJayError> {
        Ok(self.inner.has_unignored_working_copy_paths(&paths)?)
    }

    fn log(&self, revset: String) -> Result<Vec<ChangeInfo>, JayJayError> {
        Ok(self.inner.log(&revset)?)
    }

    fn log_graph(&self, revset: String) -> Result<Vec<GraphEntry>, JayJayError> {
        Ok(self.inner.log_graph(&revset)?)
    }

    fn show(&self, rev: String) -> Result<ChangeDetail, JayJayError> {
        Ok(self.inner.show(&rev)?)
    }

    /// Fast: file list without content.
    fn show_summary(&self, rev: String) -> Result<ChangeDetail, JayJayError> {
        Ok(self.inner.show_summary(&rev)?)
    }

    fn show_file(&self, rev: String, path: String) -> Result<DiffHunk, JayJayError> {
        Ok(self.inner.show_file(&rev, &path)?)
    }

    fn show_file_raw(&self, rev: String, path: String) -> Result<DiffHunk, JayJayError> {
        Ok(self.inner.show_file_raw(&rev, &path)?)
    }

    fn show_file_rename(
        &self,
        rev: String,
        old_path: String,
        new_path: String,
    ) -> Result<DiffHunk, JayJayError> {
        Ok(self.inner.show_file_rename(&rev, &old_path, &new_path)?)
    }

    fn show_file_rename_raw(
        &self,
        rev: String,
        old_path: String,
        new_path: String,
    ) -> Result<DiffHunk, JayJayError> {
        Ok(self
            .inner
            .show_file_rename_raw(&rev, &old_path, &new_path)?)
    }

    fn review_file_snapshot(
        &self,
        rev: String,
        path: String,
        old_path: Option<String>,
    ) -> Result<ReviewFileSnapshot, JayJayError> {
        Ok(self
            .inner
            .review_file_snapshot(&rev, &path, old_path.as_deref())?)
    }

    /// Fast: file list between two arbitrary revisions (no content).
    fn interdiff_summary(
        &self,
        from_rev: String,
        to_rev: String,
    ) -> Result<ChangeDetail, JayJayError> {
        Ok(self.inner.interdiff_summary(&from_rev, &to_rev)?)
    }

    fn interdiff_file(
        &self,
        from_rev: String,
        to_rev: String,
        path: String,
    ) -> Result<DiffHunk, JayJayError> {
        Ok(self.inner.interdiff_file(&from_rev, &to_rev, &path)?)
    }

    fn interdiff_file_raw(
        &self,
        from_rev: String,
        to_rev: String,
        path: String,
    ) -> Result<DiffHunk, JayJayError> {
        Ok(self.inner.interdiff_file_raw(&from_rev, &to_rev, &path)?)
    }

    fn workspace_list(&self) -> Result<Vec<WorkspaceInfo>, JayJayError> {
        Ok(self.inner.workspace_list()?)
    }

    fn workspace_add(
        &self,
        dest: String,
        name: String,
        rev: String,
    ) -> Result<String, JayJayError> {
        Ok(self.inner.workspace_add(&dest, &name, &rev)?)
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

    fn workspace_forget(
        &self,
        name: String,
        expected_root: Option<String>,
    ) -> Result<(), JayJayError> {
        Ok(self
            .inner
            .workspace_forget(&name, expected_root.as_deref())?)
    }

    fn workspace_forget_and_delete(
        &self,
        name: String,
        expected_root: String,
    ) -> Result<Option<String>, JayJayError> {
        Ok(self
            .inner
            .workspace_forget_and_delete(&name, &expected_root)?)
    }

    fn workspace_presence(&self) -> WorkspacePresence {
        self.inner.workspace_presence()
    }

    fn pull_request_info(&self, bookmark: String) -> Option<PrInfo> {
        self.inner.pull_request_info(&bookmark)
    }

    fn pull_request_open_url(&self, bookmark: String) -> Result<String, JayJayError> {
        Ok(self.inner.pull_request_open_url(&bookmark)?)
    }

    fn pr_host_name(&self) -> Option<String> {
        self.inner.pr_host_name()
    }

    fn diff_stats(&self, rev: String) -> Result<DiffStats, JayJayError> {
        Ok(self.inner.diff_stats(&rev)?)
    }

    fn diff_file_stats(
        &self,
        rev: String,
        ignore_whitespace: bool,
    ) -> Result<Vec<FileDiffStats>, JayJayError> {
        Ok(self.inner.diff_file_stats(&rev, ignore_whitespace)?)
    }

    fn annotate_file(&self, rev: String, path: String) -> Result<Vec<AnnotationLine>, JayJayError> {
        Ok(self.inner.annotate_file(&rev, &path)?)
    }

    fn file_history(&self, path: String) -> Result<Vec<ChangeInfo>, JayJayError> {
        Ok(self.inner.file_history(&path)?)
    }

    fn evolog(&self, rev: String) -> Result<Vec<EvologEntry>, JayJayError> {
        Ok(self.inner.evolog(&rev)?)
    }

    fn resolve_list(&self, rev: String) -> Result<Vec<String>, JayJayError> {
        Ok(self.inner.resolve_list(&rev)?)
    }

    fn resolve_use_ours(&self, rev: String, path: String) -> Result<(), JayJayError> {
        Ok(self.inner.resolve_use_ours(&rev, &path)?)
    }

    fn resolve_use_theirs(&self, rev: String, path: String) -> Result<(), JayJayError> {
        Ok(self.inner.resolve_use_theirs(&rev, &path)?)
    }

    fn resolve_with_tool(
        &self,
        rev: String,
        path: String,
        tool: String,
    ) -> Result<(), JayJayError> {
        Ok(self.inner.resolve_with_tool(&rev, &path, &tool)?)
    }

    fn conflict_editor(
        &self,
        rev: String,
        path: String,
    ) -> Result<ConflictEditorData, JayJayError> {
        Ok(self.inner.conflict_editor(&rev, &path)?)
    }

    fn apply_conflict_editor(
        &self,
        rev: String,
        data: ConflictEditorData,
        content: String,
    ) -> Result<(), JayJayError> {
        Ok(self.inner.apply_conflict_editor(&rev, &data, &content)?)
    }

    fn file_content(&self, rev: String, path: String) -> Result<String, JayJayError> {
        Ok(self.inner.file_content(&rev, &path)?)
    }

    fn working_copy_file_editor(&self, path: String) -> Result<FileEditorData, JayJayError> {
        Ok(self.inner.working_copy_file_editor(&path)?)
    }

    fn apply_working_copy_file_editor(
        &self,
        data: FileEditorData,
        content: String,
    ) -> Result<(), JayJayError> {
        Ok(self.inner.apply_working_copy_file_editor(&data, &content)?)
    }

    fn restore_files(&self, rev: String, paths: Vec<String>) -> Result<(), JayJayError> {
        Ok(self.inner.restore_files(&rev, None, &paths)?)
    }

    fn move_to_working_copy(&self, rev: String, paths: Vec<String>) -> Result<(), JayJayError> {
        Ok(self.inner.move_to_working_copy(&rev, &paths)?)
    }

    fn delete_files(&self, paths: Vec<String>) -> Result<(), JayJayError> {
        Ok(self.inner.delete_files(&paths)?)
    }

    fn ignore_and_untrack(&self, paths: Vec<String>) -> Result<(), JayJayError> {
        Ok(self.inner.ignore_and_untrack(&paths)?)
    }

    fn split(
        &self,
        rev: String,
        paths: Vec<String>,
        message: String,
        parallel: bool,
    ) -> Result<(), JayJayError> {
        Ok(self.inner.split(&rev, &paths, &message, parallel)?)
    }

    fn describe(&self, rev: String, message: String) -> Result<(), JayJayError> {
        Ok(self.inner.describe(&rev, &message)?)
    }

    fn new_change(&self, parent: String, message: String) -> Result<(), JayJayError> {
        Ok(self.inner.new_change(&parent, &message)?)
    }

    fn squash(&self, rev: String, into_rev: Option<String>) -> Result<(), JayJayError> {
        Ok(self.inner.squash(&rev, into_rev.as_deref())?)
    }

    fn edit(&self, rev: String) -> Result<(), JayJayError> {
        Ok(self.inner.edit(&rev)?)
    }

    fn absorb(&self, rev: String) -> Result<(), JayJayError> {
        Ok(self.inner.absorb(&rev)?)
    }

    fn revert_change(&self, rev: String) -> Result<(), JayJayError> {
        Ok(self.inner.revert_change(&rev)?)
    }

    fn merge(&self, parent_revs: Vec<String>) -> Result<(), JayJayError> {
        Ok(self.inner.merge(&parent_revs)?)
    }

    fn duplicate(&self, rev: String) -> Result<(), JayJayError> {
        Ok(self.inner.duplicate(&rev)?)
    }

    fn abandon(&self, rev: String) -> Result<(), JayJayError> {
        Ok(self.inner.abandon(&rev)?)
    }

    fn rebase(&self, rev: String, dest: String) -> Result<(), JayJayError> {
        self.inner.rebase(&rev, &dest)?;
        Ok(())
    }

    fn list_bookmarks(&self) -> Result<Vec<BookmarkInfo>, JayJayError> {
        Ok(self.inner.list_bookmarks()?)
    }

    fn create_bookmark(&self, name: String, rev: String) -> Result<(), JayJayError> {
        Ok(self.inner.create_bookmark(&name, &rev)?)
    }

    fn move_bookmark(&self, name: String, to_rev: String) -> Result<(), JayJayError> {
        Ok(self.inner.move_bookmark(&name, &to_rev)?)
    }

    fn delete_bookmark(&self, name: String) -> Result<(), JayJayError> {
        Ok(self.inner.delete_bookmark(&name)?)
    }

    fn remove_bookmark_from_rev(&self, name: String, rev: String) -> Result<(), JayJayError> {
        Ok(self.inner.remove_bookmark_from_rev(&name, &rev)?)
    }

    fn forget_bookmark(&self, name: String) -> Result<(), JayJayError> {
        Ok(self.inner.forget_bookmark(&name)?)
    }

    fn detect_stack(&self, base_rev: String, tip_rev: String) -> Result<Stack, JayJayError> {
        Ok(self.inner.detect_stack(&base_rev, &tip_rev)?)
    }

    fn submit_stack(&self, layers: Vec<SubmitStackLayer>) -> Result<StackedPrResult, JayJayError> {
        Ok(self.inner.submit_stack(layers)?)
    }

    fn rename_bookmark(&self, old_name: String, new_name: String) -> Result<(), JayJayError> {
        Ok(self.inner.rename_bookmark(&old_name, &new_name)?)
    }

    fn track_bookmark(&self, name: String, remote: String) -> Result<(), JayJayError> {
        Ok(self.inner.track_bookmark(&name, &remote)?)
    }

    fn forget_stale_bookmarks(&self) -> Result<u32, JayJayError> {
        Ok(self.inner.forget_stale_bookmarks()?)
    }

    fn git_push(
        &self,
        bookmark: String,
        sync: Arc<JayJaySyncToken>,
    ) -> Result<String, JayJayError> {
        Ok(self.inner.git_push(&bookmark, &sync.inner)?)
    }

    fn remote_web_url(&self) -> Option<String> {
        self.inner.remote_web_url()
    }

    fn git_fetch(
        &self,
        remote: String,
        sync: Arc<JayJaySyncToken>,
    ) -> Result<FetchResult, JayJayError> {
        Ok(self.inner.git_fetch(&remote, &sync.inner)?)
    }

    fn git_pull_bookmark(
        &self,
        bookmark: String,
        sync: Arc<JayJaySyncToken>,
    ) -> Result<FetchResult, JayJayError> {
        Ok(self.inner.git_pull_bookmark(&bookmark, &sync.inner)?)
    }

    fn jj_commit(&self, message: String) -> Result<(), JayJayError> {
        Ok(self.inner.jj_commit(&message)?)
    }

    fn submodule_statuses(&self) -> Result<Vec<GitSubmoduleStatus>, JayJayError> {
        Ok(self.inner.submodule_statuses()?)
    }

    fn commit_safe_submodule_updates(
        &self,
        message: String,
        paths: Vec<String>,
    ) -> Result<String, JayJayError> {
        Ok(self.inner.commit_safe_submodule_updates(&message, &paths)?)
    }

    fn git_lfs_paths(&self, paths: Vec<String>) -> Result<Vec<String>, JayJayError> {
        Ok(self.inner.git_lfs_paths(&paths)?)
    }

    fn diff_summary(&self) -> Result<String, JayJayError> {
        Ok(self.inner.diff_summary()?)
    }

    fn generate_commit_message(&self, diff_summary: String) -> Option<String> {
        self.inner.generate_commit_message(&diff_summary)
    }

    fn check_user_config(&self) -> Option<String> {
        self.inner.check_user_config()
    }

    fn op_log(&self) -> Result<Vec<OpLogEntry>, JayJayError> {
        Ok(self.inner.op_log()?)
    }

    fn op_restore(&self, op_id: String) -> Result<(), JayJayError> {
        Ok(self.inner.op_restore(&op_id)?)
    }

    fn review_notes(
        &self,
        rev: String,
        include_resolved: bool,
    ) -> Result<Vec<ReviewNoteStatus>, JayJayError> {
        Ok(self.inner.review_notes(&rev, include_resolved)?)
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
        diff::compute_file_diff(&path, &old_content, &new_content, ignore_whitespace)
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
