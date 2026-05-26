//! `RepoViewModel`: state + async loaders for a single repo window.
//!
//! Mirrors SwiftUI's `Repo/ViewModel/Core/RepoViewModel.swift` split:
//! - this module — struct, constructors, lifecycle, accessors
//! - `selection` — `select_change` / `select_file` (user-driven changes)
//! - `loaders` — async background tasks (diff, annotate, PR, refresh, avatar)

mod loaders;
mod selection;

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
    /// Bumped by `select_change`; async file-load tail commits only when still current.
    pub change_gen: u64,
    /// Bumped by `load_diff_async`.
    pub diff_gen: u64,
    /// Bumped by `load_annotate`.
    pub annotate_gen: u64,
    /// Set while `refresh()` is running; FS-triggered refreshes bail to avoid the snapshot-echo loop.
    pub refreshing: bool,
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
    pub graph: GraphData,
    pub loading: LoadingState,
}

impl RepoViewModel {
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
            graph: GraphData {
                changes: Arc::new(changes),
                entries: Arc::new(entries),
                dag_layout,
                bookmarks,
                workspaces,
            },
            loading: LoadingState::default(),
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
            graph: GraphData::default(),
            loading: LoadingState::default(),
        }
    }

    pub fn boot(&mut self, cx: &mut Context<Self>) {
        if let Some(ix) = self.selected {
            self.select_change(ix, cx);
        }
    }

    pub fn selected_change(&self) -> Option<&ChangeInfo> {
        self.selected.and_then(|ix| self.graph.changes.get(ix))
    }

    pub fn selected_hunk(&self) -> Option<&DiffHunk> {
        self.files
            .as_ref()
            .and_then(|f| self.selected_file_ix.and_then(|ix| f.get(ix)))
    }
}
