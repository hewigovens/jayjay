use std::path::{Path, PathBuf};
use std::sync::Arc;

use gpui::{
    App, AppContext, Bounds, ClickEvent, Context, Entity, FocusHandle, Focusable,
    InteractiveElement, IntoElement, ParentElement, Render, ScrollHandle, SharedString,
    StatefulInteractiveElement, Styled, TitlebarOptions, WeakEntity, Window, WindowBounds,
    WindowOptions, div, px, rgb, size,
};
use jayjay_core::overview::OverviewSnapshot;
use jayjay_core::{CoreResult, Repo};

use crate::app::actions::{CloseWindow, Dismiss, OpenFind};
use crate::app::config::AppConfigStore;
use crate::app::theme::{Theme, observe_window_appearance, theme_for_window};
use crate::repo::view_model::RepoViewModel;
use crate::repo::window::RepoWindow;
use crate::ui::popup_menu::{PopupMenu, render_popup_menu};
use crate::ui::primitives::{placeholder, placeholder_err};
use crate::ui::text_area::Newline;

mod actions;
mod canvas;
mod chrome;
mod lane_panel;
mod menu;
mod panel;
mod placement;
mod selection;

use actions::Confirmation;
use chrome::LaneFilter;
use menu::OverviewAction;
use placement::{Geometry, Placement};
use selection::Selection;

/// Reads through its own repo handle so loading at head never moves the repo window's; mutations go through the repo window's view model.
pub struct OverviewView {
    parent: WeakEntity<RepoWindow>,
    vm: Entity<RepoViewModel>,
    repo_path: SharedString,
    load: LoadState,
    snapshot: Option<Arc<OverviewSnapshot>>,
    lane_ids: Vec<String>,
    selection: Selection,
    filter: LaneFilter,
    menu: Option<PopupMenu<OverviewAction>>,
    confirmation: Option<Confirmation>,
    action_error: Option<SharedString>,
    scroll: ScrollHandle,
    reveal_selection: bool,
    focus_handle: FocusHandle,
}

struct LoadState {
    repo: Option<Arc<Repo>>,
    generation: u64,
    /// Identity of the repo window's graph data at the last load; a refresh there replaces it.
    graph_seen: usize,
    error: Option<SharedString>,
}

fn graph_identity(vm: &RepoViewModel) -> usize {
    Arc::as_ptr(&vm.graph.entries) as usize
}

impl OverviewView {
    pub(crate) fn open(parent: Entity<RepoWindow>, vm: Entity<RepoViewModel>, cx: &mut App) {
        let repo_path = vm.read(cx).repo_path.clone();
        let existing = cx.windows().into_iter().find_map(|handle| {
            let handle = handle.downcast::<Self>()?;
            (handle.read(cx).ok()?.repo_path == repo_path).then_some(handle)
        });
        if let Some(existing) = existing {
            let _ = existing.update(cx, |view, window, cx| {
                window.activate_window();
                window.focus(&view.focus_handle, cx);
            });
            return;
        }
        let name = Path::new(repo_path.as_ref())
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        let bounds = Bounds::centered(None, size(px(1280.), px(800.)), cx);
        let handle = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some(format!("Repo Overview — {name}").into()),
                    ..Default::default()
                }),
                ..crate::app::window_options()
            },
            |window, cx| {
                cx.new(|cx| {
                    cx.observe_global::<AppConfigStore>(|_, cx| cx.notify())
                        .detach();
                    cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
                    cx.observe(&vm, |view: &mut Self, vm, cx| {
                        let identity = graph_identity(vm.read(cx));
                        if identity != view.load.graph_seen {
                            view.load.graph_seen = identity;
                            view.load(cx);
                        }
                    })
                    .detach();
                    cx.observe_release_in(&parent, window, |_, _, window, _| {
                        window.remove_window();
                    })
                    .detach();
                    Self {
                        parent: parent.downgrade(),
                        load: LoadState {
                            repo: None,
                            generation: 0,
                            graph_seen: graph_identity(vm.read(cx)),
                            error: None,
                        },
                        vm: vm.clone(),
                        repo_path,
                        snapshot: None,
                        lane_ids: Vec::new(),
                        selection: Selection::default(),
                        filter: LaneFilter::new(cx),
                        menu: None,
                        confirmation: None,
                        action_error: None,
                        scroll: ScrollHandle::new(),
                        reveal_selection: false,
                        focus_handle: cx.focus_handle(),
                    }
                })
            },
        );
        if let Ok(handle) = handle {
            let _ = handle.update(cx, |view, window, cx| {
                observe_window_appearance(window, cx);
                window.focus(&view.focus_handle, cx);
                view.load(cx);
            });
        }
    }

    fn load(&mut self, cx: &mut Context<Self>) {
        self.load.generation += 1;
        let generation = self.load.generation;
        let repo = self.load.repo.clone();
        let path = PathBuf::from(self.repo_path.as_ref());
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    let repo = match repo {
                        Some(repo) => repo,
                        None => Arc::new(Repo::open(&path)?),
                    };
                    let snapshot = repo.overview_snapshot()?;
                    CoreResult::Ok((repo, snapshot))
                })
                .await;
            let _ = this.update(cx, |view, cx| {
                if view.load.generation != generation {
                    return;
                }
                match result {
                    Ok((repo, snapshot)) => {
                        view.load.repo = Some(repo);
                        view.load.error = None;
                        view.apply_snapshot(snapshot);
                    }
                    Err(error) => view.load.error = Some(crate::app::error_text(error)),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn apply_snapshot(&mut self, snapshot: OverviewSnapshot) {
        self.lane_ids = snapshot.lane_ids();
        if self
            .confirmation
            .as_ref()
            .is_some_and(|confirmation| !confirmation.is_current(&snapshot, &self.lane_ids))
        {
            self.confirmation = None;
        }
        self.snapshot = Some(Arc::new(snapshot));
    }
}

impl Focusable for OverviewView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for OverviewView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = theme_for_window(window, cx).clone();
        self.keep_selection_visible(cx);
        let snapshot = self.snapshot.clone();
        let body = match snapshot.as_deref() {
            None => match &self.load.error {
                Some(error) => placeholder_err(error, &t),
                None => placeholder("Loading…", &t),
            },
            Some(snapshot) if snapshot.overview.lanes.is_empty() => placeholder(
                "No mutable changes. Every change is on trunk or immutable.",
                &t,
            ),
            Some(snapshot) => {
                let groups = self.visible_groups(cx);
                let placement = Placement::new(
                    &snapshot.overview.lanes,
                    &groups,
                    Geometry::scaled(t.scaled_font_size(1.)),
                );
                if std::mem::take(&mut self.reveal_selection)
                    && let Some(bounds) = self.selection_bounds(&placement)
                {
                    self.scroll_into_view(bounds);
                }
                let tree = self.render_tree(snapshot, &groups, &placement, &t, cx);
                let panel = match (self.selected_lane_ix(), self.selected_change_offset()) {
                    (Some(lane_ix), Some(offset)) => {
                        let change = snapshot.overview.lanes[lane_ix].changes[offset].clone();
                        Some(self.render_change_panel(lane_ix, &change, &t, cx))
                    }
                    (Some(lane_ix), None) if self.selection.lane_panel => {
                        let lane = snapshot.overview.lanes[lane_ix].clone();
                        Some(self.render_lane_panel(lane_ix, &lane, snapshot, &t, cx))
                    }
                    _ => None,
                };
                div()
                    .flex()
                    .flex_row()
                    .flex_1()
                    .min_h_0()
                    .child(
                        div()
                            .id("overview-canvas")
                            .flex_1()
                            .h_full()
                            .overflow_scroll()
                            .track_scroll(&self.scroll)
                            .bg(rgb(t.sidebar_bg))
                            .on_click(cx.listener(|view, _: &ClickEvent, _, cx| {
                                view.clear_change();
                                view.selection.lane_panel = false;
                                cx.notify();
                            }))
                            .child(tree),
                    )
                    .children(panel)
                    .into_any_element()
            }
        };
        let header = self.header(snapshot.as_deref(), &t, cx);
        let menu = self.menu.as_ref().map(|menu| {
            let dismiss = cx.entity();
            let select = cx.entity();
            render_popup_menu(
                menu,
                "overview-menu",
                &t,
                move |_, cx| {
                    dismiss.update(cx, |view, cx| {
                        view.menu = None;
                        cx.notify();
                    });
                },
                move |action, _, cx| select.update(cx, |view, cx| view.dispatch(action, cx)),
            )
        });
        let confirmation = self.confirmation_overlay(&t, cx);
        div()
            .track_focus(&self.focus_handle)
            .key_context("OverviewView")
            .on_action(cx.listener(|_, _: &CloseWindow, window, _| window.remove_window()))
            .on_action(cx.listener(|view, _: &Dismiss, window, cx| view.dismiss(window, cx)))
            .on_action(cx.listener(|view, _: &OpenFind, window, cx| view.show_filter(window, cx)))
            .on_action(cx.listener(|view, _: &Newline, window, cx| {
                if view.filter_focused(window, cx) {
                    view.submit_filter(window, cx);
                }
            }))
            .on_key_down(cx.listener(Self::handle_key))
            .relative()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(t.sidebar_bg))
            .text_color(rgb(t.fg))
            .child(header)
            .child(body)
            .children(menu)
            .children(confirmation)
    }
}
