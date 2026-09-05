use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;

use gpui::{
    App, AppContext, Bounds, Context, Entity, FocusHandle, Focusable, Pixels, ScrollHandle,
    ScrollStrategy, SharedString, UniformListScrollHandle, Window, point,
};

use jayjay_review::NoteEntry;

use crate::app::fs_watcher::{FsEvent, IsRelevantWcChange, RepoFsWatcher};
use crate::diff::{DiffSelection, DiffWrapCache, FileTreeCache, GutterLineSelection};
use crate::repo::view_model::RepoViewModel;
#[cfg(not(target_os = "macos"))]
use crate::ui::app_menu::AppMenuState;
use crate::ui::context_menu::ContextMenuState;
use crate::ui::input::LineInput;
use crate::ui::onboarding::{OnboardingCompleted, OnboardingView};
use crate::ui::overlay::TextPrompt;
use crate::ui::text_area::TextArea;

use super::bookmark_picker::BookmarkPickerState;
use super::commit_ai::CommitAiState;
use super::commit_box::CommitBoxState;
use super::confirmation::Confirmation;
use super::dag_drag::DagRebaseRequest;
use super::repo_switcher::RepoSwitcherState;
use super::stacked_pr::StackedPrState;
use super::{ConflictEditorState, ContextExpansionState, DiffEditState, FileEditorState};

// Written by a canvas overlay during prepaint, read by mouse handlers.
pub type PanelBoundsSlot = Rc<Cell<Option<Bounds<Pixels>>>>;

pub struct RepoWindow {
    pub(crate) vm: Entity<RepoViewModel>,
    pub(crate) focus_handle: FocusHandle,
    pub(crate) active_pane: ActivePane,
    pub(crate) layout: LayoutState,
    pub(crate) file_column: FileColumnUiState,
    pub(crate) file_filter_focus: FocusHandle,
    pub(crate) find: FindState,
    pub(crate) revset_filter: Option<LineInput>,
    pub(crate) revset_filter_focus: FocusHandle,
    pub(crate) diff: DiffPanelState,
    pub(crate) diff_edit: DiffEditState,
    pub(crate) conflict_editor: ConflictEditorState,
    pub(crate) file_editor: FileEditorState,
    pub(crate) scrolls: ScrollHandles,
    pub(crate) feedback: FeedbackState,
    pub(crate) sync_activity: SyncActivity,
    pub(crate) collapsed_dirs: std::collections::HashSet<String>,
    pub(crate) file_tree_cache: FileTreeCacheSlot,
    #[cfg(not(target_os = "macos"))]
    pub(crate) app_menu: Option<AppMenuState>,
    pub(crate) context_menu: Option<ContextMenuState>,
    pub(crate) bookmark_picker: Option<BookmarkPickerState>,
    pub(crate) confirmation: Option<Confirmation>,
    pub(crate) repo_switcher: Option<RepoSwitcherState>,
    pub(crate) onboarding: Option<Entity<OnboardingView>>,
    pub(crate) summary_input: Entity<TextArea>,
    pub(crate) description_input: Entity<TextArea>,
    pub(crate) commit_box: CommitBoxState,
    pub(crate) commit_ai: CommitAiState,
    pub(crate) text_modal: Option<TextModalState>,
    pub(crate) pending_rebase: Option<DagRebaseRequest>,
    pub(crate) stacked_pr: Option<StackedPrState>,
    pub(crate) stacked_pr_provider: std::sync::Arc<dyn crate::repo::StackedPrProvider>,
    fs_watcher: Option<RepoFsWatcher>,
    /// True once the watcher's start preconditions are met (repo open + `.jj`), even when the real OS watcher is suppressed under test; lets tests assert the decision.
    fs_watcher_armed: bool,
    pub(crate) review_store: super::review::SharedReviewStore,
}

#[derive(Default)]
pub(crate) struct SyncActivity {
    pub(crate) fetching: bool,
    pub(crate) pushing: bool,
}

#[derive(Default)]
pub(crate) struct LayoutState {
    pub(crate) sidebar_width: f32,
    pub(crate) file_column_width: f32,
    pub(crate) description_height: f32,
    pub(crate) drag: Option<ColumnDrag>,
}

#[derive(Default)]
pub(crate) struct FileColumnUiState {
    pub(crate) hide_reviewed: bool,
    pub(crate) notes_only: bool,
    pub(crate) filter: Option<LineInput>,
    pub(crate) multi_select: super::file_select::FileMultiSelect,
}

#[derive(Default)]
pub(crate) struct FindState {
    pub(crate) query: Option<LineInput>,
    pub(crate) matches: Vec<usize>,
    pub(crate) current: usize,
}

pub(crate) type DiffWrapCacheSlot = Rc<RefCell<DiffWrapCache>>;

pub(crate) type FileTreeCacheSlot = Rc<RefCell<FileTreeCache>>;

pub(crate) struct DiffPanelState {
    pub(crate) selection: Option<DiffSelection>,
    /// Gutter's line-range selection; mutually exclusive with `selection` (starting one clears the other).
    pub(crate) gutter_selection: Option<GutterLineSelection>,
    pub(crate) rich_preview: Option<DiffRichPreviewSelection>,
    pub(crate) unified_bounds: PanelBoundsSlot,
    pub(crate) sbs_old_bounds: PanelBoundsSlot,
    pub(crate) sbs_new_bounds: PanelBoundsSlot,
    pub(crate) markdown_scroll: ScrollHandle,
    /// Markdown preview pane's rendered width, used to size table columns by content.
    pub(crate) markdown_bounds: PanelBoundsSlot,
    pub(crate) wrap_cache: DiffWrapCacheSlot,
    pub(crate) context_expansion: ContextExpansionState,
    /// `sync_review_notes`'s change-detection key: reviewable-files fingerprint + last raw note list, so a store write or a diff refresh (identity change) both trigger re-reconciliation.
    pub(crate) review_notes_sync_key: Option<(u64, Vec<NoteEntry>)>,
}

impl Default for DiffPanelState {
    fn default() -> Self {
        Self {
            selection: None,
            gutter_selection: None,
            rich_preview: None,
            unified_bounds: Rc::new(Cell::new(None)),
            sbs_old_bounds: Rc::new(Cell::new(None)),
            sbs_new_bounds: Rc::new(Cell::new(None)),
            markdown_scroll: ScrollHandle::new(),
            markdown_bounds: Rc::new(Cell::new(None)),
            wrap_cache: Rc::new(RefCell::new(DiffWrapCache::default())),
            context_expansion: ContextExpansionState::default(),
            review_notes_sync_key: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DiffRichPreviewSelection {
    pub(crate) kind: DiffRichPreviewKind,
    pub(crate) path: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DiffRichPreviewKind {
    Projection,
    Markdown,
    Svg,
}

impl DiffRichPreviewSelection {
    pub(crate) fn is_active(&self, kind: DiffRichPreviewKind, path: &str) -> bool {
        self.kind == kind && self.path == path
    }
}

pub(crate) struct ScrollHandles {
    pub(crate) changes: UniformListScrollHandle,
    pub(crate) files: UniformListScrollHandle,
    pub(crate) tree_files: ScrollHandle,
    pub(crate) diff: UniformListScrollHandle,
}

impl Default for ScrollHandles {
    fn default() -> Self {
        Self {
            changes: UniformListScrollHandle::new(),
            files: UniformListScrollHandle::new(),
            tree_files: ScrollHandle::new(),
            diff: UniformListScrollHandle::new(),
        }
    }
}

#[derive(Default)]
pub(crate) struct FeedbackState {
    pub(crate) recently_copied: Option<SharedString>,
    pub(crate) toast: Option<SharedString>,
    pub(crate) pending_push_bookmark: Option<SharedString>,
}

pub(crate) struct TextModalState {
    pub(crate) prompt: TextPrompt,
    pub(crate) action: TextModalAction,
    /// Read-only diff excerpt shown above the input; only the review-note composer sets this, and its presence is also the render-time signal that gates the `"NoteComposer"` key context so mod+Return doesn't grow a new shortcut on every other text modal.
    pub(crate) context: Option<TextModalContext>,
    /// Optional labeled toggle rendered below the input; only the split-files modal sets this (SwiftUI's "Parallel split" checkbox).
    pub(crate) checkbox: Option<TextModalCheckbox>,
    /// Optional monospace path list rendered below the checkbox; only the split-files modal sets this.
    pub(crate) file_list: Option<Vec<SharedString>>,
}

impl TextModalState {
    pub(crate) fn new(prompt: TextPrompt, action: TextModalAction) -> Self {
        Self {
            prompt,
            action,
            context: None,
            checkbox: None,
            file_list: None,
        }
    }
}

pub(crate) struct TextModalContext {
    pub(crate) lines: Vec<super::note_composer::NoteContextLine>,
    pub(crate) input: Entity<TextArea>,
}

pub(crate) struct TextModalCheckbox {
    pub(crate) label: SharedString,
    pub(crate) checked: bool,
}

#[derive(Clone)]
pub(crate) enum TextModalAction {
    EditDescription {
        rev: String,
    },
    DiffEditDescription {
        session: u64,
    },
    CreateBookmark {
        rev: String,
    },
    ReviewNote(super::note_composer::NoteComposerTarget),
    /// Carries the already-validated parent directory the workspace will be created under.
    CreateWorkspace(std::path::PathBuf),
    SplitFiles(std::sync::Arc<super::file_actions::SelectedFilesRequest>),
}

impl TextModalAction {
    pub(super) fn submits_on_enter(&self) -> bool {
        matches!(
            self,
            Self::CreateBookmark { .. } | Self::CreateWorkspace(_) | Self::SplitFiles(_)
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivePane {
    Sidebar,
    FileColumn,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ColumnDrag {
    pub(crate) target: DragTarget,
    /// Pointer coord at drag start; axis (x/y) depends on `target`.
    pub(crate) start_pos: f32,
    pub(crate) start_size: f32,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum DragTarget {
    Sidebar,
    FileColumn,
    Description,
}

pub(crate) const SIDEBAR_MIN: f32 = 240.;
pub(crate) const SIDEBAR_MAX: f32 = 600.;
pub(crate) const SECONDARY_PANE_DEFAULT: f32 = 260.;
pub(crate) const SECONDARY_PANE_MIN: f32 = 220.;
pub(crate) const SECONDARY_PANE_MAX: f32 = 480.;
/// Both shells fit the sidebar and file column so the preview keeps at least this.
pub(crate) const PREVIEW_MIN: f32 = 420.;
pub(crate) const DESCRIPTION_DEFAULT: f32 = 32.;
pub(crate) const DESCRIPTION_MIN: f32 = 24.;
pub(crate) const DESCRIPTION_MAX: f32 = 180.;
const DESCRIPTION_LEGACY_DEFAULT: f32 = 64.;

pub(crate) fn pane_max(min: f32, max: f32, room: f32) -> f32 {
    max.min(room.max(min))
}

impl RepoWindow {
    pub fn new(path: PathBuf, cx: &mut Context<Self>) -> Self {
        Self::new_internal(path, true, cx)
    }

    pub fn new_with_onboarding(path: PathBuf, cx: &mut Context<Self>) -> Self {
        let mut view = Self::new_internal(path, false, cx);
        let onboarding = cx.new(OnboardingView::new);
        cx.subscribe(&onboarding, |view, _, _: &OnboardingCompleted, cx| {
            view.onboarding = None;
            view.vm.update(cx, |vm, cx| vm.open_async(cx));
            cx.notify();
        })
        .detach();
        view.onboarding = Some(onboarding);
        view
    }

    fn new_internal(path: PathBuf, open_now: bool, cx: &mut Context<Self>) -> Self {
        let review_store = super::review::shared(cx);
        // Open off the main thread (`Repo::open` + initial revset eval are slow on large repos); render a loading pane until it lands.
        let vm_path = path.clone();
        let vm = cx.new(|cx| {
            let mut vm = RepoViewModel::opening(vm_path);
            if open_now {
                vm.open_async(cx);
            }
            vm
        });
        // Summary + optional body combine into jj's one change description (summary\n\nbody).
        let summary_input = cx.new(|cx| TextArea::new("", "Summary", false, 32., cx));
        let description_input =
            cx.new(|cx| TextArea::new("", "Description (optional)", true, 60., cx));
        cx.observe(&vm, |this, _vm, cx| {
            // A repo that opened after `new` (e.g. in-app `jj git init`) has no watcher yet.
            if !this.fs_watcher_armed {
                this.start_fs_watcher(cx);
            }
            this.recompute_find_matches(cx);
            this.reset_context_expansion_if_basis_changed(cx);
            this.clear_notes_only_if_empty(cx);
            this.prune_file_multi_select(cx);
            this.sync_diff_edit_loaded_files(cx);
            this.sync_commit_box_from_working_copy(cx);
            cx.notify();
        })
        .detach();
        let mut view = Self {
            vm,
            focus_handle: cx.focus_handle(),
            active_pane: ActivePane::Sidebar,
            layout: LayoutState {
                sidebar_width: 380.,
                file_column_width: SECONDARY_PANE_DEFAULT,
                description_height: DESCRIPTION_DEFAULT,
                drag: None,
            },
            file_column: FileColumnUiState::default(),
            file_filter_focus: cx.focus_handle(),
            find: FindState::default(),
            revset_filter: None,
            revset_filter_focus: cx.focus_handle(),
            diff: DiffPanelState::default(),
            diff_edit: DiffEditState::default(),
            conflict_editor: ConflictEditorState::default(),
            file_editor: FileEditorState::default(),
            scrolls: ScrollHandles::default(),
            feedback: FeedbackState::default(),
            sync_activity: SyncActivity::default(),
            collapsed_dirs: std::collections::HashSet::new(),
            file_tree_cache: Rc::new(RefCell::new(FileTreeCache::default())),
            #[cfg(not(target_os = "macos"))]
            app_menu: None,
            context_menu: None,
            bookmark_picker: None,
            confirmation: None,
            repo_switcher: None,
            onboarding: None,
            summary_input,
            description_input,
            commit_box: CommitBoxState::default(),
            commit_ai: CommitAiState::default(),
            text_modal: None,
            pending_rebase: None,
            stacked_pr: None,
            stacked_pr_provider: std::sync::Arc::new(crate::repo::CoreStackedPrProvider),
            fs_watcher: None,
            fs_watcher_armed: false,
            review_store,
        };
        // Real AI-CLI detection may spawn a login shell to resolve PATH; keep it out of the deterministic test scheduler (tests inject a mock provider explicitly), same reason the fs watcher is suppressed.
        if !crate::app::fs_watcher::is_watcher_suppressed(cx) {
            view.redetect_commit_ai_provider(cx);
        }
        view
    }

    pub fn boot(&mut self, cx: &mut Context<Self>) {
        let cfg = crate::app::config::current(cx);
        if cfg.layout.sidebar_width > 0. {
            self.layout.sidebar_width = cfg.layout.sidebar_width.clamp(SIDEBAR_MIN, SIDEBAR_MAX);
        }
        if cfg.layout.secondary_pane_width > 0. {
            self.layout.file_column_width = cfg
                .layout
                .secondary_pane_width
                .clamp(SECONDARY_PANE_MIN, SECONDARY_PANE_MAX);
        }
        if cfg.layout.description_height > 0. {
            // Treat the previous default as unset so existing config files migrate to the new default.
            let description_height = if (cfg.layout.description_height - DESCRIPTION_LEGACY_DEFAULT)
                .abs()
                < f32::EPSILON
            {
                DESCRIPTION_DEFAULT
            } else {
                cfg.layout.description_height
            };
            self.layout.description_height =
                description_height.clamp(DESCRIPTION_MIN, DESCRIPTION_MAX);
        }
        // `boot` only restores window layout; the repo is opened async from `RepoWindow::new`.
    }

    /// Every repo window needs this after `open_window`: theme tracking and initial focus.
    pub fn attach_to_window(&self, window: &mut Window, cx: &mut Context<Self>) {
        crate::app::theme::observe_window_appearance(window, cx);
        window.focus(&self.focus_handle, cx);
    }

    pub fn view_model(&self) -> Entity<RepoViewModel> {
        self.vm.clone()
    }

    pub fn summary_input(&self) -> Entity<TextArea> {
        self.summary_input.clone()
    }

    pub fn description_input(&self) -> Entity<TextArea> {
        self.description_input.clone()
    }

    pub fn active_pane(&self) -> ActivePane {
        self.active_pane
    }

    pub fn set_active_pane(&mut self, pane: ActivePane) {
        self.active_pane = pane;
    }

    pub fn set_diff_selection(&mut self, selection: Option<DiffSelection>) {
        self.diff.selection = selection;
    }

    pub fn has_diff_selection(&self) -> bool {
        self.diff.selection.is_some()
    }

    pub fn gutter_selection(&self) -> Option<GutterLineSelection> {
        self.diff.gutter_selection.clone()
    }

    pub fn toast(&self) -> Option<SharedString> {
        self.feedback.toast.clone()
    }

    pub fn revset_filter_text(&self) -> Option<String> {
        self.revset_filter
            .as_ref()
            .map(|input| input.text().to_owned())
    }

    pub fn pending_push_bookmark(&self) -> Option<SharedString> {
        self.feedback.pending_push_bookmark.clone()
    }

    pub fn pending_diff_scroll_target(&self) -> Option<(usize, ScrollStrategy, bool)> {
        self.scrolls
            .diff
            .0
            .borrow()
            .deferred_scroll_to_item
            .map(|target| (target.item_index, target.strategy, target.scroll_strict))
    }

    pub fn set_diff_scroll_offset_y(&mut self, y: Pixels) {
        let base = self.scrolls.diff.0.borrow().base_handle.clone();
        let offset = base.offset();
        base.set_offset(point(offset.x, y));
    }

    pub fn diff_scroll_offset_y(&self) -> Pixels {
        self.scrolls.diff.0.borrow().base_handle.offset().y
    }

    pub fn markdown_preview_scroll_offset_y(&self) -> Pixels {
        self.diff.markdown_scroll.offset().y
    }

    pub fn has_text_modal(&self) -> bool {
        self.text_modal.is_some()
    }

    /// Mirrors `summary_input()`/`description_input()`: `pub` so the separate `tests/` crate can drive the review-note composer without reaching `pub(crate)` state.
    pub fn text_modal_input(&self) -> Option<Entity<TextArea>> {
        self.text_modal.as_ref().map(|m| m.prompt.input.clone())
    }

    pub fn text_modal_context_input(&self) -> Option<Entity<TextArea>> {
        self.text_modal
            .as_ref()
            .and_then(|m| m.context.as_ref())
            .map(|context| context.input.clone())
    }

    /// Mirrors `text_modal_input()`: lets the tests crate assert header hints such as the New Workspace destination.
    pub fn text_modal_subtitle(&self) -> Option<SharedString> {
        self.text_modal.as_ref().map(|m| m.prompt.subtitle.clone())
    }

    /// `None` when the current modal has no checkbox row (only the split-files modal does).
    pub fn text_modal_checkbox_checked(&self) -> Option<bool> {
        self.text_modal
            .as_ref()
            .and_then(|m| m.checkbox.as_ref())
            .map(|c| c.checked)
    }

    /// `None` when the current modal has no file-list section (only the split-files modal does).
    pub fn text_modal_file_list(&self) -> Option<Vec<SharedString>> {
        self.text_modal.as_ref().and_then(|m| m.file_list.clone())
    }

    pub fn notes_only_files(&self) -> bool {
        self.file_column.notes_only
    }

    pub fn fs_watcher_armed(&self) -> bool {
        self.fs_watcher_armed
    }

    pub fn handle_fs_event(&mut self, cx: &mut Context<Self>) {
        self.sync_refresh_gate(cx);
        self.vm.update(cx, |vm, cx| vm.handle_fs_event(cx));
    }

    /// One chokepoint for refresh suspension: render (and the event path) mirror the overlay state into the view model, which owes itself a deferred refresh once the gate clears.
    pub(crate) fn sync_refresh_gate(&self, cx: &mut Context<Self>) {
        let suspended = self.has_refresh_sensitive_interaction();
        if self.vm.read(cx).refresh_suspended != suspended {
            self.vm
                .update(cx, |vm, cx| vm.set_refresh_suspended(suspended, cx));
        }
    }

    fn has_refresh_sensitive_interaction(&self) -> bool {
        self.text_modal.is_some()
            || self.confirmation.is_some()
            || self.pending_rebase.is_some()
            || self.stacked_pr.is_some()
            || self.diff_edit.active
            || self.file_editor.active
            || self.file_editor.preparing
            || self.conflict_editor.active
            || self.conflict_editor.preparing
    }

    pub fn hide_reviewed_files(&self) -> bool {
        self.file_column.hide_reviewed
    }

    pub fn file_filter_visible(&self) -> bool {
        self.file_column.filter.is_some()
    }

    pub fn file_filter_query(&self) -> Option<&str> {
        self.file_column.filter.as_ref().map(LineInput::text)
    }

    pub fn mark_unreviewed(&mut self, change_id: &str, path: &str) {
        super::review::mutate(&self.review_store, |store| {
            store.mark_unreviewed(change_id, path);
        });
    }

    fn start_fs_watcher(&mut self, cx: &mut Context<Self>) {
        let (repo_path, repo) = {
            let vm = self.vm.read(cx);
            (vm.repo_path.to_string(), vm.repo.clone())
        };
        let Some(repo) = repo else {
            return;
        };
        if repo_path.is_empty() {
            return;
        }
        let path = std::path::PathBuf::from(&repo_path);
        if !path.join(".jj").exists() {
            return;
        }
        self.fs_watcher_armed = true;
        if crate::app::fs_watcher::is_watcher_suppressed(cx) {
            return;
        }
        let (tx, rx) = flume::unbounded::<FsEvent>();
        let filter: IsRelevantWcChange = std::sync::Arc::new({
            let repo = repo.clone();
            move |paths: &[std::path::PathBuf]| -> bool {
                let strs: Vec<String> = paths
                    .iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect();
                repo.has_unignored_working_copy_paths(&strs).unwrap_or(true)
            }
        });
        let Some(watcher) = RepoFsWatcher::new(&path, tx, filter) else {
            return;
        };
        self.fs_watcher = Some(watcher);

        cx.spawn(async move |this, cx| {
            while let Ok(_event) = rx.recv_async().await {
                let _ = this.update(cx, |view, cx| {
                    view.handle_fs_event(cx);
                });
            }
        })
        .detach();
    }
}

impl Focusable for RepoWindow {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
