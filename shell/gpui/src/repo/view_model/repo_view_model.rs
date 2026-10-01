use super::selection::SelectionCache;
use super::{DiffCache, GraphData, LoadingState, NotesState, ShownDiff, StatsState};
use crate::diff::{DetailMode, DiffViewMode};
use gpui::SharedString;
use jayjay_core::compare::CompareState;
use jayjay_core::dag::OrderedSelection;
use jayjay_core::diff::ConflictLineKind;
use jayjay_core::{
    AnnotationLine, ChangeInfo, DiffHunk, DiffStats, PrInfo, Repo, RevsetFilterState,
    RevsetVocabulary,
};
use std::cell::RefCell;
use std::collections::HashSet;
use std::sync::Arc;

pub struct RepoViewModel {
    pub repo: Option<Arc<Repo>>,
    pub(crate) fix_unavailable_reason: Option<SharedString>,
    pub(crate) repo_path: SharedString,
    pub(crate) repo_root_path: SharedString,
    pub error: Option<SharedString>,
    pub selected: Option<usize>,
    pub(super) selected_changes: OrderedSelection,
    pub files: Option<Arc<Vec<DiffHunk>>>,
    pub(crate) conflicted_paths: Arc<HashSet<String>>,
    pub selected_file_ix: Option<usize>,
    pub shown: ShownDiff,
    pub diff_cache: DiffCache,
    pub stats: StatsState,
    pub working_copy_stats: Option<DiffStats>,
    pub current_operation_description: String,
    pub view_mode: DiffViewMode,
    pub(crate) ignore_whitespace: bool,
    pub revset_filter: RevsetFilterState,
    pub can_load_more: bool,
    /// Loaded with the graph, not on each keystroke.
    pub(crate) vocabulary: RevsetVocabulary,
    pub(crate) detail_mode: DetailMode,
    pub(crate) annotate_lines: Option<Arc<Vec<AnnotationLine>>>,
    pub(super) avatar_in_flight: HashSet<String>,
    pub pr_info: Option<PrInfo>,
    pub(crate) pr_host_name: Option<SharedString>,
    pub compare: Option<CompareState>,
    pub graph: GraphData,
    pub loading: LoadingState,
    /// Stamped when we start a jj write so the FS echo from our own mutation is ignored.
    pub last_internal_mutation_at: Option<std::time::Instant>,
    /// While true, FS-triggered refreshes are remembered in `loading.pending_auto_refresh` instead of run.
    pub refresh_suspended: bool,
    pub notes: NotesState,
    /// One-shot, consumed synchronously by `select_change` so a superseded call can't leak it into an unrelated selection.
    pub(super) pending_file_selection: Option<String>,
    /// Shown once the next refresh lands, since re-selecting the refreshed change clears `error`; a new action drops it.
    pub(in crate::repo) pending_error: Option<SharedString>,
    pub(super) selection_cache: RefCell<Option<SelectionCache>>,
}

impl RepoViewModel {
    pub(crate) fn present_error(&mut self, error: impl std::fmt::Display) {
        self.error = Some(crate::app::error_text(error));
    }

    pub(crate) fn clear_error(&mut self) {
        self.error = None;
    }

    pub fn selected_change(&self) -> Option<&ChangeInfo> {
        self.selected.and_then(|ix| self.graph.changes.get(ix))
    }

    /// `None` in compare mode, where the displayed interdiff's files are not the selected change's files.
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
            .map(|change| change.selection_revision().to_owned())
    }

    pub fn selected_hunk(&self) -> Option<&DiffHunk> {
        self.files
            .as_ref()
            .and_then(|f| self.selected_file_ix.and_then(|ix| f.get(ix)))
    }

    pub(crate) fn selected_file_has_conflict(&self) -> bool {
        self.selected_hunk().is_some_and(|hunk| {
            self.conflicted_paths.contains(&hunk.path) || hunk.is_conflict_only_placeholder()
        }) || self.shown.diff.as_ref().is_some_and(|diff| {
            diff.lines
                .iter()
                .any(|line| line.conflict_kind != ConflictLineKind::None)
        })
    }

    pub(super) fn clear_diff_cache_state(&mut self) {
        self.diff_cache.loaded.clear();
        self.diff_cache.preloads_in_flight.clear();
        self.diff_cache.load_failures.clear();
    }

    /// A bare `is_working_copy` check would wrongly pass in compare mode, where review state doesn't apply.
    pub(crate) fn shows_review_controls(&self) -> bool {
        self.selected_change().is_some_and(|c| c.is_working_copy)
            && self.compare.is_none()
            && !self.has_multiple_change_selection()
    }
}
