use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;

use gpui::{
    App, AppContext, Bounds, Context, Entity, FocusHandle, Focusable, Pixels, ScrollStrategy,
    SharedString, UniformListScrollHandle, Window, point,
};

use crate::app::fs_watcher::{FsEvent, IsRelevantWcChange, RepoFsWatcher};
use crate::diff::{DiffSelection, DiffWrapCache, FileTreeCache};
use crate::repo::view_model::RepoViewModel;
use crate::ui::context_menu::ContextMenuState;
use crate::ui::input::LineInput;
use crate::ui::text_area::TextArea;

// Written by a canvas overlay during prepaint, read by mouse handlers.
pub type PanelBoundsSlot = Rc<Cell<Option<Bounds<Pixels>>>>;

pub struct RepoWindow {
    pub(crate) vm: Entity<RepoViewModel>,
    pub(crate) focus_handle: FocusHandle,
    pub(crate) active_pane: ActivePane,
    pub(crate) layout: LayoutState,
    pub(crate) find: FindState,
    pub(crate) diff: DiffPanelState,
    pub(crate) scrolls: ScrollHandles,
    pub(crate) feedback: FeedbackState,
    pub(crate) collapsed_dirs: std::collections::HashSet<String>,
    pub(crate) file_tree_cache: FileTreeCacheSlot,
    pub(crate) context_menu: Option<ContextMenuState>,
    pub(crate) commit_input: Entity<TextArea>,
    pub(crate) text_modal: Option<TextModalState>,
    pub(crate) fs_watcher: Option<RepoFsWatcher>,
    /// True once the watcher's start preconditions are met (repo open + `.jj`), even when the
    /// real OS watcher is suppressed under test; lets tests assert the decision.
    pub(crate) fs_watcher_armed: bool,
    pub(crate) review_store: super::review::SharedReviewStore,
}

#[derive(Default)]
pub(crate) struct LayoutState {
    pub(crate) sidebar_width: f32,
    pub(crate) file_column_width: f32,
    pub(crate) description_height: f32,
    pub(crate) drag: Option<ColumnDrag>,
}

#[derive(Default)]
pub(crate) struct FindState {
    pub(crate) query: Option<LineInput>,
    pub(crate) matches: Vec<usize>,
    pub(crate) current: usize,
}

/// Shared wrap cache so render reuses wrapped diff output across frames instead of re-wrapping per `cx.notify()`.
pub(crate) type DiffWrapCacheSlot = Rc<RefCell<DiffWrapCache>>;

/// Shared file-tree cache so tree mode reuses the built tree across frames instead of rebuilding per `cx.notify()`.
pub(crate) type FileTreeCacheSlot = Rc<RefCell<FileTreeCache>>;

pub(crate) struct DiffPanelState {
    pub(crate) selection: Option<DiffSelection>,
    pub(crate) unified_bounds: PanelBoundsSlot,
    pub(crate) sbs_old_bounds: PanelBoundsSlot,
    pub(crate) sbs_new_bounds: PanelBoundsSlot,
    pub(crate) wrap_cache: DiffWrapCacheSlot,
}

impl Default for DiffPanelState {
    fn default() -> Self {
        Self {
            selection: None,
            unified_bounds: Rc::new(Cell::new(None)),
            sbs_old_bounds: Rc::new(Cell::new(None)),
            sbs_new_bounds: Rc::new(Cell::new(None)),
            wrap_cache: Rc::new(RefCell::new(DiffWrapCache::default())),
        }
    }
}

pub(crate) struct ScrollHandles {
    pub(crate) changes: UniformListScrollHandle,
    pub(crate) files: UniformListScrollHandle,
    pub(crate) diff: UniformListScrollHandle,
}

impl Default for ScrollHandles {
    fn default() -> Self {
        Self {
            changes: UniformListScrollHandle::new(),
            files: UniformListScrollHandle::new(),
            diff: UniformListScrollHandle::new(),
        }
    }
}

#[derive(Default)]
pub(crate) struct FeedbackState {
    pub(crate) recently_copied: Option<SharedString>,
    pub(crate) toast: Option<SharedString>,
}

pub(crate) struct TextModalState {
    pub(crate) title: SharedString,
    pub(crate) subtitle: SharedString,
    pub(crate) primary_label: SharedString,
    pub(crate) action: TextModalAction,
    pub(crate) input: Entity<TextArea>,
}

#[derive(Clone)]
pub(crate) enum TextModalAction {
    EditDescription { rev: String },
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
pub(crate) const FILE_COLUMN_MIN: f32 = 200.;
pub(crate) const FILE_COLUMN_MAX: f32 = 480.;
pub(crate) const DESCRIPTION_MIN: f32 = 24.;
pub(crate) const DESCRIPTION_MAX: f32 = 360.;

impl RepoWindow {
    pub fn new(path: PathBuf, cx: &mut Context<Self>) -> Self {
        let review_store = super::review::shared(cx);
        // Open off the main thread (`Repo::open` + initial revset eval are slow on large repos); render a loading pane until it lands.
        let vm = cx.new(|cx| {
            let mut vm = RepoViewModel::opening(path);
            vm.open_async(cx);
            vm
        });
        let commit_input =
            cx.new(|cx| TextArea::new("", "Describe the working-copy change", true, 76., cx));
        cx.observe(&vm, |this, _vm, cx| {
            // A repo that opened after `new` (e.g. in-app `jj git init`) has no watcher yet.
            if !this.fs_watcher_armed {
                this.start_fs_watcher(cx);
            }
            this.recompute_find_matches(cx);
            cx.notify();
        })
        .detach();
        Self {
            vm,
            focus_handle: cx.focus_handle(),
            active_pane: ActivePane::Sidebar,
            layout: LayoutState {
                sidebar_width: 380.,
                file_column_width: 260.,
                description_height: 64.,
                drag: None,
            },
            find: FindState::default(),
            diff: DiffPanelState::default(),
            scrolls: ScrollHandles::default(),
            feedback: FeedbackState::default(),
            collapsed_dirs: std::collections::HashSet::new(),
            file_tree_cache: Rc::new(RefCell::new(FileTreeCache::default())),
            context_menu: None,
            commit_input,
            text_modal: None,
            fs_watcher: None,
            fs_watcher_armed: false,
            review_store,
        }
    }

    pub fn boot(&mut self, cx: &mut Context<Self>) {
        let cfg = crate::app::config::current(cx);
        if cfg.layout.sidebar_width > 0. {
            self.layout.sidebar_width = cfg.layout.sidebar_width.clamp(SIDEBAR_MIN, SIDEBAR_MAX);
        }
        if cfg.layout.description_height > 0. {
            self.layout.description_height = cfg
                .layout
                .description_height
                .clamp(DESCRIPTION_MIN, DESCRIPTION_MAX);
        }
        // `boot` only restores window layout; the repo is opened async from `RepoWindow::new`.
    }

    /// Keep the view model's window-active flag current; it gates the WC-review badge.
    pub fn observe_window_active(&self, window: &mut Window, cx: &mut Context<Self>) {
        let active = window.is_window_active();
        self.vm
            .update(cx, |vm, _| vm.is_repo_window_active = active);
        cx.observe_window_activation(window, |view, window, cx| {
            let active = window.is_window_active();
            view.vm
                .update(cx, |vm, _| vm.is_repo_window_active = active);
        })
        .detach();
    }

    pub fn view_model(&self) -> Entity<RepoViewModel> {
        self.vm.clone()
    }

    pub fn commit_input(&self) -> Entity<TextArea> {
        self.commit_input.clone()
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

    pub fn has_text_modal(&self) -> bool {
        self.text_modal.is_some()
    }

    /// Whether the FS watcher's start preconditions have been met. Exposed for lifecycle tests.
    pub fn fs_watcher_armed(&self) -> bool {
        self.fs_watcher_armed
    }

    pub fn mark_unreviewed(&mut self, change_id: &str, path: &str) {
        self.review_store
            .borrow_mut()
            .mark_unreviewed(change_id, path);
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
        // Preconditions met: record the decision even when the real watcher is suppressed.
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
                    let vm = view.vm.clone();
                    vm.update(cx, |vm, cx| vm.handle_working_copy_change(cx));
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
