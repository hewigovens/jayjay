//! `RepoViewModel`: state + async loaders for a single repo window.

mod loaders;
pub(crate) mod mutations;
mod mutations_changes;
mod mutations_conflict_editor;
mod mutations_file_editor;
mod mutations_files;
mod refresh_indicator;
mod selection;
mod tasks;

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

use gpui::{Context, SharedString};
use jayjay_core::dag::DagLayout;
use jayjay_core::diff::FileDiff;
use jayjay_core::{
    AnnotationLine, BookmarkInfo, ChangeInfo, DEFAULT_REVSET_DEPTH, DiffHunk, DiffProjection,
    DiffStats, GraphEntry, PrInfo, Repo, WorkspaceInfo, build_default_revset,
};
use jayjay_markdown::MarkdownDocument;
use jayjay_review::ReviewNoteStatus;

use crate::diff::{DetailMode, DiffViewMode};
use crate::repo::revset::CompareState;
use crate::ui::ordered_selection::OrderedSelection;

struct OpenedRepo {
    repo: Arc<Repo>,
    repo_root_path: String,
    entries: Vec<GraphEntry>,
    bookmarks: Vec<BookmarkInfo>,
    workspaces: Vec<WorkspaceInfo>,
    pr_host_name: Option<String>,
}

/// All graph-level data refreshed together by `refresh()` / `load_more()`.
pub struct GraphData {
    pub changes: Arc<Vec<ChangeInfo>>,
    pub entries: Arc<Vec<GraphEntry>>,
    pub(crate) dag_layout: Arc<DagLayout>,
    pub(crate) bookmarks: Arc<Vec<BookmarkInfo>>,
    pub workspaces: Arc<Vec<WorkspaceInfo>>,
}

impl Default for GraphData {
    fn default() -> Self {
        Self {
            changes: Arc::new(Vec::new()),
            entries: Arc::new(Vec::new()),
            dag_layout: Arc::new(DagLayout::default()),
            bookmarks: Arc::new(Vec::new()),
            workspaces: Arc::new(Vec::new()),
        }
    }
}

/// Per-section loading flags, stale-click generation counters, and FS-watcher gates.
#[derive(Default)]
pub struct LoadingState {
    pub(crate) files: bool,
    pub diff: bool,
    pub(crate) annotate: bool,
    pub more: bool,
    pr: bool,
    pub refresh_indicator: bool,
    /// Bumped by `select_change`; async file-load tail commits only when still current.
    change_gen: u64,
    pub diff_gen: u64,
    annotate_gen: u64,
    /// Bumped by `refresh_pr_info` and `select_change`; drops out-of-order PR fetches.
    pub pr_gen: u64,
    /// Bumped by `load_review_notes`; drops a reconciliation reply superseded by a newer one.
    review_notes_gen: u64,
    /// True while any refresh/mutation runs; FS-triggered refreshes bail to avoid the snapshot-echo loop.
    pub refreshing: bool,
    /// Count of in-flight refresh/mutation tasks. `refreshing == (in_flight > 0)` keeps the gate set until all finish.
    pub in_flight: u32,
    /// Bumped each time `refresh()` starts; the completion discards data from a superseded run.
    pub(crate) refresh_gen: u64,
    /// Bumped by `refresh_workspaces`; an older list must not overwrite a newer one.
    workspaces_gen: u64,
    /// An owed auto-refresh: set when an FS event arrives mid-refresh or while refreshes are suspended; run by the completion or when the gate clears.
    pub pending_auto_refresh: bool,
    refresh_indicator_gen: u64,
    refresh_minimum_elapsed: bool,
}

pub struct RepoViewModel {
    pub repo: Option<Arc<Repo>>,
    pub(crate) repo_path: SharedString,
    pub(crate) repo_root_path: SharedString,
    pub error: Option<SharedString>,
    pub selected: Option<usize>,
    selected_changes: OrderedSelection<usize>,
    pub files: Option<Arc<Vec<DiffHunk>>>,
    pub selected_file_ix: Option<usize>,
    pub current_diff: Option<Arc<FileDiff>>,
    pub current_projection: Option<DiffProjection>,
    pub current_svg_preview: Option<Arc<SvgPreviewContent>>,
    /// Post-change document only — the rich preview renders a single after view.
    pub current_markdown_preview: Option<Arc<MarkdownDocument>>,
    /// The (old, new) content `current_diff` was computed from.
    pub current_diff_old_content: Option<Arc<str>>,
    pub current_diff_new_content: Option<Arc<str>>,
    pub current_diff_supports_file_editor: bool,
    pub diff_cache: HashMap<String, LoadedDiff>,
    diff_preloads_in_flight: HashSet<String>,
    diff_load_failures: HashSet<String>,
    pub change_stats: Option<DiffStats>,
    pub working_copy_stats: Option<DiffStats>,
    pub current_operation_description: String,
    pub view_mode: DiffViewMode,
    pub(crate) ignore_whitespace: bool,
    pub revset: SharedString,
    pub(crate) revset_depth: u32,
    pub can_load_more: bool,
    pub(crate) detail_mode: DetailMode,
    pub(crate) annotate_lines: Option<Arc<Vec<AnnotationLine>>>,
    avatar_in_flight: HashSet<String>,
    pub pr_info: Option<PrInfo>,
    pub(crate) pr_host_name: Option<SharedString>,
    pub compare: Option<CompareState>,
    pub graph: GraphData,
    pub loading: LoadingState,
    /// Stamped when we start a jj write so the FS echo from our own mutation is ignored.
    pub last_internal_mutation_at: Option<std::time::Instant>,
    /// Mirrored from the window's overlay state; while true, FS-triggered refreshes are remembered in `loading.pending_auto_refresh` instead of run.
    pub refresh_suspended: bool,
    /// Every file's notes for the selected change (`include_resolved: true`); scoped down to a single hunk elsewhere.
    pub review_notes: Vec<ReviewNoteStatus>,
    /// Recomputed only where `review_notes` is written (`load_review_notes`), not on every render — every file-list render reads it via `active_note_counts`.
    active_note_counts_cache: Arc<HashMap<String, usize>>,
    /// One-shot, consumed synchronously by `select_change` so a superseded call can't leak it into an unrelated later selection; set by mutations (e.g. abandon-selected-lines) before the `refresh()` that reloads the file list.
    pending_file_selection: Option<String>,
}

#[derive(Clone)]
pub struct LoadedDiff {
    pub diff: Arc<FileDiff>,
    pub projection: Option<DiffProjection>,
    pub svg_preview: Option<Arc<SvgPreviewContent>>,
    pub markdown_preview: Option<Arc<MarkdownDocument>>,
    /// The exact (old, new) strings `diff` was computed from; must be retained rather than re-read, since file content may have changed by the time an abandon-selected-lines action runs.
    pub old_content: Option<Arc<str>>,
    pub new_content: Option<Arc<str>>,
    pub supports_file_editor: bool,
}

pub(in crate::repo) enum DiffLoadState {
    Missing,
    Failed,
    Loaded(LoadedDiff),
}

#[derive(Clone)]
pub struct SvgPreviewContent {
    pub(crate) old: Option<String>,
    pub new: Option<String>,
}

impl RepoViewModel {
    pub(crate) fn present_error(&mut self, error: impl std::fmt::Display) {
        self.error = Some(format!("{error}").into());
    }

    pub(crate) fn clear_error(&mut self) {
        self.error = None;
    }

    pub fn new(path: PathBuf) -> Self {
        let repo_path: SharedString = path.display().to_string().into();
        let depth = DEFAULT_REVSET_DEPTH;
        let revset = build_default_revset(depth);
        match Self::open_blocking(path, &revset) {
            Ok(loaded) => Self::ready(repo_path, revset.into(), depth, loaded),
            Err(e) => Self::error(repo_path, format!("{e}")),
        }
    }

    /// Pair with [`RepoViewModel::open_async`], which does the heavy open + graph load off the main thread.
    pub fn opening(path: PathBuf) -> Self {
        Self::empty(path.display().to_string().into())
    }

    /// Keeps window-open off the UI thread, since open/revset eval is slow on large checkouts.
    pub fn open_async(&mut self, cx: &mut Context<Self>) {
        let path = PathBuf::from(self.repo_path.as_ref());
        let depth = self.revset_depth;
        let revset = self.revset.to_string();
        let ready_revset = self.revset.clone();
        self.begin_refreshing(cx);
        Self::background_update(
            cx,
            async move { Self::open_blocking(path, &revset) },
            move |vm, opened, cx| {
                vm.finish_repo_task(cx);
                match opened {
                    Ok(loaded) => {
                        *vm = Self::ready(vm.repo_path.clone(), ready_revset, depth, loaded);
                        vm.boot(cx);
                    }
                    Err(e) => vm.present_error(e),
                }
                cx.notify();
            },
        );
    }

    fn open_blocking(path: PathBuf, revset: &str) -> jayjay_core::CoreResult<OpenedRepo> {
        let repo_root_path = jayjay_core::workspace_primary_root(&path.to_string_lossy())
            .unwrap_or_else(|| path.to_string_lossy().into_owned());
        let repo = Repo::open(&path)?;
        let entries = repo.log_graph(revset)?;
        let bookmarks = repo.list_bookmarks().unwrap_or_default();
        let workspaces = repo.workspace_list().unwrap_or_default();
        let pr_host_name = repo.pr_host_name();
        Ok(OpenedRepo {
            repo: Arc::new(repo),
            repo_root_path,
            entries,
            bookmarks,
            workspaces,
            pr_host_name,
        })
    }

    fn ready(
        repo_path: SharedString,
        revset: SharedString,
        revset_depth: u32,
        loaded: OpenedRepo,
    ) -> Self {
        let OpenedRepo {
            repo,
            repo_root_path,
            entries,
            bookmarks,
            workspaces,
            pr_host_name,
        } = loaded;
        let selected = entries
            .iter()
            .position(|e| e.change.is_working_copy)
            .or(if entries.is_empty() { None } else { Some(0) });
        let mut selected_changes = OrderedSelection::default();
        if let Some(selected) = selected {
            selected_changes.replace(selected);
        }
        let dag_layout = Arc::new(DagLayout::compute(&entries));
        let changes: Vec<ChangeInfo> = entries.iter().map(|e| e.change.clone()).collect();
        Self {
            repo: Some(repo),
            repo_path,
            repo_root_path: repo_root_path.into(),
            error: None,
            selected,
            selected_changes,
            files: None,
            selected_file_ix: None,
            current_diff: None,
            current_projection: None,
            current_svg_preview: None,
            current_markdown_preview: None,
            current_diff_old_content: None,
            current_diff_new_content: None,
            current_diff_supports_file_editor: false,
            diff_cache: HashMap::new(),
            diff_preloads_in_flight: HashSet::new(),
            diff_load_failures: HashSet::new(),
            change_stats: None,
            working_copy_stats: None,
            current_operation_description: String::new(),
            view_mode: DiffViewMode::Unified,
            ignore_whitespace: false,
            revset,
            revset_depth,
            can_load_more: changes.len() >= revset_depth as usize,
            detail_mode: DetailMode::Diff,
            annotate_lines: None,
            avatar_in_flight: HashSet::new(),
            pr_info: None,
            pr_host_name: pr_host_name.map(SharedString::from),
            compare: None,
            graph: GraphData {
                changes: Arc::new(changes),
                entries: Arc::new(entries),
                dag_layout,
                bookmarks: Arc::new(bookmarks),
                workspaces: Arc::new(workspaces),
            },
            loading: LoadingState::default(),
            last_internal_mutation_at: None,
            refresh_suspended: false,
            review_notes: Vec::new(),
            active_note_counts_cache: Arc::new(HashMap::new()),
            pending_file_selection: None,
        }
    }

    /// A repo-less view model — base for the error and still-opening states.
    fn empty(repo_path: SharedString) -> Self {
        Self {
            repo: None,
            repo_root_path: repo_path.clone(),
            repo_path,
            error: None,
            selected: None,
            selected_changes: OrderedSelection::default(),
            files: None,
            selected_file_ix: None,
            current_diff: None,
            current_projection: None,
            current_svg_preview: None,
            current_markdown_preview: None,
            current_diff_old_content: None,
            current_diff_new_content: None,
            current_diff_supports_file_editor: false,
            diff_cache: HashMap::new(),
            diff_preloads_in_flight: HashSet::new(),
            diff_load_failures: HashSet::new(),
            change_stats: None,
            working_copy_stats: None,
            current_operation_description: String::new(),
            view_mode: DiffViewMode::Unified,
            ignore_whitespace: false,
            revset: build_default_revset(DEFAULT_REVSET_DEPTH).into(),
            revset_depth: DEFAULT_REVSET_DEPTH,
            can_load_more: false,
            detail_mode: DetailMode::Diff,
            annotate_lines: None,
            avatar_in_flight: HashSet::new(),
            pr_info: None,
            pr_host_name: None,
            compare: None,
            graph: GraphData::default(),
            loading: LoadingState::default(),
            last_internal_mutation_at: None,
            refresh_suspended: false,
            review_notes: Vec::new(),
            active_note_counts_cache: Arc::new(HashMap::new()),
            pending_file_selection: None,
        }
    }

    fn error(repo_path: SharedString, msg: String) -> Self {
        let mut vm = Self::empty(repo_path);
        vm.error = Some(msg.into());
        vm
    }

    pub fn boot(&mut self, cx: &mut Context<Self>) {
        cx.on_app_quit(|vm, _| {
            if let Some(repo) = &vm.repo {
                repo.cancel_running_jj_processes();
            }
            async {}
        })
        .detach();
        // Snapshot small repos on open so the WC is current; huge checkouts defer (snapshot is slow).
        if self
            .repo
            .as_ref()
            .is_some_and(|repo| !repo.working_copy_is_large())
        {
            self.refresh(false, cx);
        } else if let Some(ix) = self.selected {
            self.select_change(ix, cx);
        }
    }

    pub fn selected_change(&self) -> Option<&ChangeInfo> {
        self.selected.and_then(|ix| self.graph.changes.get(ix))
    }

    /// The shared gate for change-scoped file operations (multi-select, batch menu): `None` in compare mode, where the displayed interdiff's files are not the selected change's files.
    pub(crate) fn selected_change_for_file_ops(&self) -> Option<&ChangeInfo> {
        if self.compare.is_some() || self.has_multiple_change_selection() {
            return None;
        }
        self.selected_change()
    }

    pub(crate) fn working_copy_change(&self) -> Option<&ChangeInfo> {
        self.graph.changes.iter().find(|c| c.is_working_copy)
    }

    pub(crate) fn selected_revision(&self) -> Option<String> {
        self.selected_change()
            .map(crate::repo::revset::change_revision)
    }

    pub fn selected_hunk(&self) -> Option<&DiffHunk> {
        self.files
            .as_ref()
            .and_then(|f| self.selected_file_ix.and_then(|ix| f.get(ix)))
    }

    fn clear_diff_cache_state(&mut self) {
        self.diff_cache.clear();
        self.diff_preloads_in_flight.clear();
        self.diff_load_failures.clear();
    }

    /// The shared gate every review surface (marks, notes) uses: a bare `is_working_copy` check would wrongly pass in compare mode, where the displayed diff is an interdiff and review state doesn't apply.
    pub(crate) fn shows_review_controls(&self) -> bool {
        self.selected_change().is_some_and(|c| c.is_working_copy)
            && self.compare.is_none()
            && !self.has_multiple_change_selection()
    }
}
