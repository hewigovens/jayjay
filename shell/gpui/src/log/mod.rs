pub mod actions;
pub mod commit_row;
pub mod dag;
pub mod detail;
pub mod diff_select;
pub mod drag;
pub mod find;
pub mod menu;
pub mod nav;
pub mod render;
pub mod sidebar;
pub mod status_bar;

use std::cell::Cell;
use std::path::PathBuf;
use std::rc::Rc;

use gpui::{
    App, AppContext, Bounds, Context, Entity, FocusHandle, Focusable, Pixels, Point, SharedString,
    Size, TitlebarOptions, UniformListScrollHandle, WindowBounds, WindowOptions, px,
};

use crate::app::fs_watcher::RepoFsWatcher;
use crate::diff::DiffSelection;
use crate::repo::view_model::RepoViewModel;
use crate::ui::context_menu::ContextMenuState;

/// Shared bounds slot — written by a `gpui::canvas` overlay during prepaint,
/// read by mouse handlers to compute pixel→column on the same frame.
pub type PanelBoundsSlot = Rc<Cell<Option<Bounds<Pixels>>>>;

pub struct LogView {
    pub vm: Entity<RepoViewModel>,
    pub focus_handle: FocusHandle,
    pub sidebar_width: f32,
    pub file_column_width: f32,
    pub description_height: f32,
    pub drag: Option<ColumnDrag>,
    pub active_pane: ActivePane,
    pub recently_copied: Option<SharedString>,
    pub collapsed_dirs: std::collections::HashSet<String>,
    pub find_query: Option<String>,
    pub find_matches: Vec<usize>,
    pub find_current: usize,
    pub changes_scroll: UniformListScrollHandle,
    pub files_scroll: UniformListScrollHandle,
    pub diff_scroll: UniformListScrollHandle,
    pub context_menu: Option<ContextMenuState>,
    pub fs_watcher: Option<RepoFsWatcher>,
    pub review_store: jayjay_core::review::ReviewStore,
    pub toast: Option<SharedString>,
    pub diff_selection: Option<DiffSelection>,
    pub diff_unified_bounds: PanelBoundsSlot,
    pub diff_sbs_old_bounds: PanelBoundsSlot,
    pub diff_sbs_new_bounds: PanelBoundsSlot,
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
        let review_store = jayjay_core::review::ReviewStore::load(path.clone());
        let vm = cx.new(|_| RepoViewModel::new(path));
        cx.observe(&vm, |this, _vm, cx| {
            this.recompute_find_matches(cx);
            cx.notify();
        })
        .detach();
        Self {
            vm,
            focus_handle: cx.focus_handle(),
            sidebar_width: 380.,
            file_column_width: 260.,
            description_height: 64.,
            drag: None,
            active_pane: ActivePane::Sidebar,
            recently_copied: None,
            collapsed_dirs: std::collections::HashSet::new(),
            find_query: None,
            find_matches: Vec::new(),
            find_current: 0,
            changes_scroll: UniformListScrollHandle::new(),
            files_scroll: UniformListScrollHandle::new(),
            diff_scroll: UniformListScrollHandle::new(),
            context_menu: None,
            fs_watcher: None,
            review_store,
            toast: None,
            diff_selection: None,
            diff_unified_bounds: Rc::new(Cell::new(None)),
            diff_sbs_old_bounds: Rc::new(Cell::new(None)),
            diff_sbs_new_bounds: Rc::new(Cell::new(None)),
        }
    }

    pub fn boot(&mut self, cx: &mut Context<Self>) {
        let cfg = crate::app::config::current(cx);
        if cfg.layout.sidebar_width > 0. {
            self.sidebar_width = cfg.layout.sidebar_width.clamp(SIDEBAR_MIN, SIDEBAR_MAX);
        }
        if cfg.layout.description_height > 0. {
            self.description_height = cfg
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
        let (tx, rx) = flume::unbounded::<crate::app::fs_watcher::FsEvent>();
        let filter: crate::app::fs_watcher::IsRelevantWcChange = std::sync::Arc::new({
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

/// Open a new repo window pointing at `path`. Used by main.rs for the initial
/// window and by the workspace switcher to open a sibling workspace.
pub fn open_repo_window(path: PathBuf, cx: &mut App) {
    let title = match path.file_name().and_then(|s| s.to_str()) {
        Some(name) if !name.is_empty() => format!("JayJay (Alpha) — {name}"),
        _ => "JayJay (Alpha)".to_string(),
    };
    let bounds = Bounds::centered(
        None,
        Size {
            width: px(1080.),
            height: px(720.),
        },
        cx,
    );
    let opts = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: Some(TitlebarOptions {
            title: Some(title.into()),
            appears_transparent: true,
            traffic_light_position: Some(Point {
                x: px(12.),
                y: px(12.),
            }),
        }),
        ..Default::default()
    };
    let _ = cx.open_window(opts, move |_, cx| {
        cx.new(|cx| {
            cx.observe_global::<crate::app::theme::Theme>(|_, cx| cx.notify())
                .detach();
            cx.observe_global::<crate::app::config::AppConfigStore>(|_, cx| cx.notify())
                .detach();
            let mut view = LogView::new(path.clone(), cx);
            view.boot(cx);
            view
        })
    });
}
