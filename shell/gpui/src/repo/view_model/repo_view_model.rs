use super::selection::SelectionCache;
use super::{GraphData, LoadedDiff, LoadingState, SvgPreviewContent};
use crate::diff::{DetailMode, DiffViewMode};
use gpui::SharedString;
use jayjay_core::compare::CompareState;
use jayjay_core::dag::OrderedSelection;
use jayjay_core::diff::{ConflictLineKind, FileDiff};
use jayjay_core::{
    AnnotationLine, ChangeInfo, DiffHunk, DiffProjection, DiffStats, FileDiffStats, PrInfo, Repo,
    RevsetFilterState, RevsetVocabulary,
};
use jayjay_markdown::MarkdownDocument;
use jayjay_review::ReviewNoteStatus;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub struct RepoViewModel {
    pub repo: Option<Arc<Repo>>,
    pub(crate) repo_path: SharedString,
    pub(crate) repo_root_path: SharedString,
    pub error: Option<SharedString>,
    pub selected: Option<usize>,
    pub(super) selected_changes: OrderedSelection,
    pub files: Option<Arc<Vec<DiffHunk>>>,
    pub(crate) conflicted_paths: Arc<HashSet<String>>,
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
    pub(super) diff_preloads_in_flight: HashSet<String>,
    pub(super) diff_load_failures: HashSet<String>,
    pub change_stats: Option<DiffStats>,
    pub file_stats: Arc<HashMap<String, FileDiffStats>>,
    pub working_copy_stats: Option<DiffStats>,
    pub current_operation_description: String,
    pub view_mode: DiffViewMode,
    pub(crate) ignore_whitespace: bool,
    pub revset_filter: RevsetFilterState,
    pub can_load_more: bool,
    /// Refs, tags and aliases the revset field completes from; loaded with the graph, not on each keystroke.
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
    /// Mirrored from the window's overlay state; while true, FS-triggered refreshes are remembered in `loading.pending_auto_refresh` instead of run.
    pub refresh_suspended: bool,
    /// Every file's notes for the selected change (`include_resolved: true`); scoped down to a single hunk elsewhere.
    pub review_notes: Vec<ReviewNoteStatus>,
    /// Recomputed only where `review_notes` is written (`load_review_notes`), not on every render — every file-list render reads it via `active_note_counts`.
    pub(super) active_note_counts_cache: Arc<HashMap<String, usize>>,
    /// One-shot, consumed synchronously by `select_change` so a superseded call can't leak it into an unrelated later selection; set by mutations (e.g. abandon-selected-lines) before the `refresh()` that reloads the file list.
    pub(super) pending_file_selection: Option<String>,
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
        }) || self.current_diff.as_ref().is_some_and(|diff| {
            diff.lines
                .iter()
                .any(|line| line.conflict_kind != ConflictLineKind::None)
        })
    }

    pub(super) fn clear_diff_cache_state(&mut self) {
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
