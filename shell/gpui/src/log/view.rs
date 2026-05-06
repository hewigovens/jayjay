use std::cell::Cell;
use std::path::PathBuf;
use std::rc::Rc;

use gpui::{
    App, AppContext, Bounds, Context, Entity, FocusHandle, Focusable, Pixels, SharedString,
    UniformListScrollHandle,
};

use crate::app::fs_watcher::{FsEvent, IsRelevantWcChange, RepoFsWatcher};
use crate::diff::DiffSelection;
use crate::repo::view_model::RepoViewModel;
use crate::ui::context_menu::ContextMenuState;

// Written by a canvas overlay during prepaint, read by mouse handlers.
pub type PanelBoundsSlot = Rc<Cell<Option<Bounds<Pixels>>>>;

pub struct LogView {
    pub vm: Entity<RepoViewModel>,
    pub focus_handle: FocusHandle,
    pub active_pane: ActivePane,
    pub layout: LayoutState,
    pub find: FindState,
    pub diff: DiffPanelState,
    pub scrolls: ScrollHandles,
    pub feedback: FeedbackState,
    pub collapsed_dirs: std::collections::HashSet<String>,
    pub context_menu: Option<ContextMenuState>,
    pub fs_watcher: Option<RepoFsWatcher>,
    pub review_store: jayjay_core::review::ReviewStore,
}

#[derive(Default)]
pub struct LayoutState {
    pub sidebar_width: f32,
    pub file_column_width: f32,
    pub description_height: f32,
    pub drag: Option<ColumnDrag>,
}

#[derive(Default)]
pub struct FindState {
    pub query: Option<String>,
    pub matches: Vec<usize>,
    pub current: usize,
}

pub struct DiffPanelState {
    pub selection: Option<DiffSelection>,
    pub unified_bounds: PanelBoundsSlot,
    pub sbs_old_bounds: PanelBoundsSlot,
    pub sbs_new_bounds: PanelBoundsSlot,
}

impl Default for DiffPanelState {
    fn default() -> Self {
        Self {
            selection: None,
            unified_bounds: Rc::new(Cell::new(None)),
            sbs_old_bounds: Rc::new(Cell::new(None)),
            sbs_new_bounds: Rc::new(Cell::new(None)),
        }
    }
}

pub struct ScrollHandles {
    pub changes: UniformListScrollHandle,
    pub files: UniformListScrollHandle,
    pub diff: UniformListScrollHandle,
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
pub struct FeedbackState {
    pub recently_copied: Option<SharedString>,
    pub toast: Option<SharedString>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivePane {
    Sidebar,
    FileColumn,
}

#[derive(Debug, Clone, Copy)]
pub struct ColumnDrag {
    pub target: DragTarget,
    /// Pointer coord at drag start; axis (x/y) depends on `target`.
    pub start_pos: f32,
    pub start_size: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum DragTarget {
    Sidebar,
    FileColumn,
    Description,
}

pub const SIDEBAR_MIN: f32 = 240.;
pub const SIDEBAR_MAX: f32 = 600.;
pub const FILE_COLUMN_MIN: f32 = 200.;
pub const FILE_COLUMN_MAX: f32 = 480.;
pub const DESCRIPTION_MIN: f32 = 24.;
pub const DESCRIPTION_MAX: f32 = 360.;

impl LogView {
    pub fn new(path: PathBuf, cx: &mut Context<Self>) -> Self {
        let review_store = jayjay_core::review::ReviewStore::load();
        let vm = cx.new(|_| RepoViewModel::new(path));
        cx.observe(&vm, |this, _vm, cx| {
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
            context_menu: None,
            fs_watcher: None,
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
        let vm = self.vm.clone();
        vm.update(cx, |vm, cx| vm.boot(cx));
        self.start_fs_watcher(cx);
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
                    vm.update(cx, |vm, cx| vm.refresh(true, cx));
                });
            }
        })
        .detach();
    }
}

impl Focusable for LogView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
