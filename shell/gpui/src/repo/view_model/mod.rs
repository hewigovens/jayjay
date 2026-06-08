//! `RepoViewModel`: state + async loaders for a single repo window.
//!
//! Mirrors SwiftUI's `Repo/ViewModel/Core/RepoViewModel.swift` split:
//! - this module — struct, constructors, lifecycle, accessors
//! - `selection` — `select_change` / `select_file` (user-driven changes)
//! - `loaders` — async background tasks (diff, annotate, PR, refresh, avatar)

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
    AnnotationLine, BookmarkInfo, ChangeInfo, DEFAULT_REVSET_DEPTH, DiffHunk, DiffStats,
    GraphEntry, PrInfo, Repo, WorkspaceInfo, build_default_revset,
};

use crate::diff::{DetailMode, DiffViewMode};
use crate::repo::revset::CompareState;

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

/// Per-section "is loading" booleans + stale-click generation counters +
/// the FS-watcher gates. Grouped so the top-level VM stays scannable.
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
    /// Set while `refresh()` is running; FS-triggered refreshes bail to avoid the snapshot-echo loop.
    pub refreshing: bool,
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
    pub diff_cache: HashMap<String, Option<Arc<FileDiff>>>,
    pub change_stats: Option<DiffStats>,
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

impl RepoViewModel {
    pub fn present_error(&mut self, error: impl std::fmt::Display) {
        self.error = Some(format!("{error}").into());
    }

    pub fn clear_error(&mut self) {
        self.error = None;
    }

    pub fn new(path: PathBuf) -> Self {
        let repo_path: SharedString = path.display().to_string().into();
        let initial_depth = DEFAULT_REVSET_DEPTH;
        match Repo::open(&path) {
            Ok(repo) => match repo.log_graph(&build_default_revset(initial_depth)) {
                Ok(entries) => {
                    let selected = entries
                        .iter()
                        .position(|e| e.change.is_working_copy)
                        .or(if entries.is_empty() { None } else { Some(0) });
                    Self::ready(Arc::new(repo), repo_path, entries, selected, initial_depth)
                }
                Err(e) => Self::error(repo_path, format!("{e}")),
            },
            Err(e) => Self::error(repo_path, format!("{e}")),
        }
    }

    fn ready(
        repo: Arc<Repo>,
        repo_path: SharedString,
        entries: Vec<GraphEntry>,
        selected: Option<usize>,
        revset_depth: u32,
    ) -> Self {
        let dag_layout = Arc::new(DagLayout::compute(&entries));
        let changes: Vec<ChangeInfo> = entries.iter().map(|e| e.change.clone()).collect();
        let bookmarks = Arc::new(repo.list_bookmarks().unwrap_or_default());
        let workspaces = Arc::new(repo.workspace_list().unwrap_or_default());
        let pr_host_name = repo.pr_host_name().map(SharedString::from);
        Self {
            repo: Some(repo),
            repo_path,
            error: None,
            selected,
            files: None,
            selected_file_ix: None,
            current_diff: None,
            diff_cache: HashMap::new(),
            change_stats: None,
            view_mode: DiffViewMode::Unified,
            ignore_whitespace: false,
            revset_depth,
            detail_mode: DetailMode::Diff,
            annotate_lines: None,
            avatar_in_flight: HashSet::new(),
            pr_info: None,
            pr_host_name,
            compare: None,
            graph: GraphData {
                changes: Arc::new(changes),
                entries: Arc::new(entries),
                dag_layout,
                bookmarks,
                workspaces,
            },
            loading: LoadingState::default(),
            is_repo_window_active: true,
            last_internal_mutation_at: None,
        }
    }

    fn error(repo_path: SharedString, msg: String) -> Self {
        Self {
            repo: None,
            repo_path,
            error: Some(msg.into()),
            selected: None,
            files: None,
            selected_file_ix: None,
            current_diff: None,
            diff_cache: HashMap::new(),
            change_stats: None,
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
