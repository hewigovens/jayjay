//! `RepoViewModel`: state + async loaders for a single repo window.
//! Mirrors SwiftUI's `Repo/ViewModel/Core/RepoViewModel.swift` split: this module owns the
//! struct, constructors, lifecycle, and accessors; sibling modules own selection, mutations,
//! loaders, the refresh indicator, and task plumbing.

mod loaders;
mod mutations;
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

use crate::diff::{DetailMode, DiffViewMode};
use crate::repo::revset::CompareState;

/// Result of opening a repo + loading its initial graph off the main thread.
struct OpenedRepo {
    repo: Arc<Repo>,
    entries: Vec<GraphEntry>,
    bookmarks: Vec<BookmarkInfo>,
    workspaces: Vec<WorkspaceInfo>,
    pr_host_name: Option<String>,
}

/// All graph-level data refreshed together by `refresh()` / `load_more()`.
pub struct GraphData {
    pub changes: Arc<Vec<ChangeInfo>>,
    pub entries: Arc<Vec<GraphEntry>>,
    pub dag_layout: Arc<DagLayout>,
    pub bookmarks: Arc<Vec<BookmarkInfo>>,
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
    pub files: bool,
    pub diff: bool,
    pub annotate: bool,
    pub more: bool,
    pub pr: bool,
    pub refresh_indicator: bool,
    /// Bumped by `select_change`; async file-load tail commits only when still current.
    pub change_gen: u64,
    /// Bumped by `load_diff_async`.
    pub diff_gen: u64,
    /// Bumped by `load_annotate`.
    pub annotate_gen: u64,
    /// Bumped by `refresh_pr_info` and `select_change`; drops out-of-order PR fetches.
    pub pr_gen: u64,
    /// True while any refresh/mutation runs; FS-triggered refreshes bail to avoid the snapshot-echo loop.
    pub refreshing: bool,
    /// Count of in-flight refresh/mutation tasks. `refreshing == (in_flight > 0)` keeps the gate set until all finish.
    pub in_flight: u32,
    /// Bumped each time `refresh()` starts; the completion discards data from a superseded run.
    pub refresh_gen: u64,
    /// Set when an FS event arrives mid-refresh; the completion re-runs `refresh()` so the tail isn't lost.
    pub pending_auto_refresh: bool,
    pub refresh_indicator_gen: u64,
    pub refresh_minimum_elapsed: bool,
    /// Set when an auto-triggered refresh is suppressed because the user is reviewing the WC.
    pub wc_changes: bool,
}

pub struct RepoViewModel {
    pub repo: Option<Arc<Repo>>,
    pub repo_path: SharedString,
    pub error: Option<SharedString>,
    pub selected: Option<usize>,
    pub files: Option<Arc<Vec<DiffHunk>>>,
    pub selected_file_ix: Option<usize>,
    pub current_diff: Option<Arc<FileDiff>>,
    pub current_projection: Option<DiffProjection>,
    pub current_svg_preview: Option<Arc<SvgPreviewContent>>,
    pub current_markdown_preview: Option<Arc<MarkdownPreviewContent>>,
    pub diff_cache: HashMap<String, LoadedDiff>,
    pub change_stats: Option<DiffStats>,
    pub working_copy_stats: Option<DiffStats>,
    pub current_operation_description: String,
    pub view_mode: DiffViewMode,
    pub ignore_whitespace: bool,
    pub revset_depth: u32,
    pub detail_mode: DetailMode,
    pub annotate_lines: Option<Arc<Vec<AnnotationLine>>>,
    pub avatar_in_flight: HashSet<String>,
    pub pr_info: Option<PrInfo>,
    pub pr_host_name: Option<SharedString>,
    pub compare: Option<CompareState>,
    pub graph: GraphData,
    pub loading: LoadingState,
    /// True while this repo's window is the active one — gates the WC-review badge.
    pub is_repo_window_active: bool,
    /// Stamped when we start a jj write so the FS echo from our own mutation is ignored.
    pub last_internal_mutation_at: Option<std::time::Instant>,
}

#[derive(Clone)]
pub struct LoadedDiff {
    pub diff: Arc<FileDiff>,
    pub projection: Option<DiffProjection>,
    pub svg_preview: Option<Arc<SvgPreviewContent>>,
    pub markdown_preview: Option<Arc<MarkdownPreviewContent>>,
}

#[derive(Clone)]
pub struct SvgPreviewContent {
    pub old: Option<String>,
    pub new: Option<String>,
}

#[derive(Clone)]
pub struct MarkdownPreviewContent {
    pub old: Option<MarkdownDocument>,
    pub new: Option<MarkdownDocument>,
}

impl RepoViewModel {
    pub fn present_error(&mut self, error: impl std::fmt::Display) {
        self.error = Some(format!("{error}").into());
    }

    pub fn clear_error(&mut self) {
        self.error = None;
    }

    pub fn new(path: PathBuf) -> Self {
        let repo_path: SharedString = path.display().to_string().into();
        let depth = DEFAULT_REVSET_DEPTH;
        match Self::open_blocking(path, depth) {
            Ok(loaded) => Self::ready(repo_path, depth, loaded),
            Err(e) => Self::error(repo_path, format!("{e}")),
        }
    }

    /// A still-opening view model (no repo yet, renders the loading state). Pair with
    /// [`RepoViewModel::open_async`], which does the heavy open + graph load off the main thread.
    pub fn opening(path: PathBuf) -> Self {
        let mut vm = Self::empty(path.display().to_string().into());
        vm.is_repo_window_active = true;
        vm
    }

    /// Open the repo and load the initial graph on a background task, then install and boot.
    /// Keeps window-open off the UI thread, since open/revset eval is slow on large checkouts.
    pub fn open_async(&mut self, cx: &mut Context<Self>) {
        let path = PathBuf::from(self.repo_path.as_ref());
        let depth = self.revset_depth;
        // Drive the refresh indicator state machine like `refresh` does.
        self.begin_refreshing(cx);
        Self::background_update(
            cx,
            async move { Self::open_blocking(path, depth) },
            move |vm, opened, cx| {
                vm.finish_refreshing(cx);
                match opened {
                    Ok(loaded) => {
                        let active = vm.is_repo_window_active;
                        *vm = Self::ready(vm.repo_path.clone(), depth, loaded);
                        vm.is_repo_window_active = active;
                        vm.boot(cx);
                    }
                    Err(e) => vm.present_error(e),
                }
                cx.notify();
            },
        );
    }

    fn open_blocking(path: PathBuf, depth: u32) -> jayjay_core::CoreResult<OpenedRepo> {
        let repo = Repo::open(&path)?;
        let entries = repo.log_graph(&build_default_revset(depth))?;
        let bookmarks = repo.list_bookmarks().unwrap_or_default();
        let workspaces = repo.workspace_list().unwrap_or_default();
        let pr_host_name = repo.pr_host_name();
        Ok(OpenedRepo {
            repo: Arc::new(repo),
            entries,
            bookmarks,
            workspaces,
            pr_host_name,
        })
    }

    fn ready(repo_path: SharedString, revset_depth: u32, loaded: OpenedRepo) -> Self {
        let OpenedRepo {
            repo,
            entries,
            bookmarks,
            workspaces,
            pr_host_name,
        } = loaded;
        let selected = entries
            .iter()
            .position(|e| e.change.is_working_copy)
            .or(if entries.is_empty() { None } else { Some(0) });
        let dag_layout = Arc::new(DagLayout::compute(&entries));
        let changes: Vec<ChangeInfo> = entries.iter().map(|e| e.change.clone()).collect();
        Self {
            repo: Some(repo),
            repo_path,
            error: None,
            selected,
            files: None,
            selected_file_ix: None,
            current_diff: None,
            current_projection: None,
            current_svg_preview: None,
            current_markdown_preview: None,
            diff_cache: HashMap::new(),
            change_stats: None,
            working_copy_stats: None,
            current_operation_description: String::new(),
            view_mode: DiffViewMode::Unified,
            ignore_whitespace: false,
            revset_depth,
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
            is_repo_window_active: true,
            last_internal_mutation_at: None,
        }
    }

    /// A repo-less view model — base for the error and still-opening states.
    fn empty(repo_path: SharedString) -> Self {
        Self {
            repo: None,
            repo_path,
            error: None,
            selected: None,
            files: None,
            selected_file_ix: None,
            current_diff: None,
            current_projection: None,
            current_svg_preview: None,
            current_markdown_preview: None,
            diff_cache: HashMap::new(),
            change_stats: None,
            working_copy_stats: None,
            current_operation_description: String::new(),
            view_mode: DiffViewMode::Unified,
            ignore_whitespace: false,
            revset_depth: DEFAULT_REVSET_DEPTH,
            detail_mode: DetailMode::Diff,
            annotate_lines: None,
            avatar_in_flight: HashSet::new(),
            pr_info: None,
            pr_host_name: None,
            compare: None,
            graph: GraphData::default(),
            loading: LoadingState::default(),
            is_repo_window_active: false,
            last_internal_mutation_at: None,
        }
    }

    fn error(repo_path: SharedString, msg: String) -> Self {
        let mut vm = Self::empty(repo_path);
        vm.error = Some(msg.into());
        vm
    }

    pub fn boot(&mut self, cx: &mut Context<Self>) {
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

    pub fn selected_revision(&self) -> Option<String> {
        self.selected_change()
            .map(crate::repo::revset::change_revision)
    }

    pub fn selected_hunk(&self) -> Option<&DiffHunk> {
        self.files
            .as_ref()
            .and_then(|f| self.selected_file_ix.and_then(|ix| f.get(ix)))
    }
}
