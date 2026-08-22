use std::sync::Arc;

mod chrome;
mod context_menu;
mod rows;

use gpui::{
    App, AppContext, Bounds, Context, Entity, FocusHandle, Focusable, InteractiveElement,
    IntoElement, ParentElement, Pixels, Point, Render, SharedString, Size, Styled, TitlebarOptions,
    Window, WindowBounds, WindowOptions, div, px, rgb,
};
use jayjay_core::{BookmarkInfo, CoreResult, Repo};

use crate::app::actions::{CloseWindow, Dismiss};
use crate::app::config::AppConfigStore;
use crate::app::theme::{Theme, observe_window_appearance, theme};
use crate::repo::revset;
use crate::repo::view_model::RepoViewModel;
use crate::repo::window::RepoWindow;
use crate::ui::text_area::TextArea;
use chrome::{BookmarkStats, footer, header, placeholder, placeholder_err, stats_bar};
use context_menu::render_context_menu as render_bookmark_context_menu;
use context_menu::{BookmarkContextAction, BookmarkContextMenuState, bookmark_menu_items};
use rows::bookmark_list;

pub struct BookmarkManagerView {
    repo: Arc<Repo>,
    parent: Entity<RepoWindow>,
    vm: Entity<RepoViewModel>,
    bookmarks: Arc<Vec<BookmarkInfo>>,
    pr_host_name: Option<SharedString>,
    filter: Entity<TextArea>,
    show_deleted: bool,
    loading: bool,
    error: Option<SharedString>,
    context_menu: Option<BookmarkContextMenuState>,
    focus_handle: FocusHandle,
}

impl BookmarkManagerView {
    pub(crate) fn open(parent: Entity<RepoWindow>, vm: Entity<RepoViewModel>, cx: &mut App) {
        let Some(repo) = vm.read(cx).repo.clone() else {
            return;
        };
        let pr_host_name = vm.read(cx).pr_host_name.clone();
        let bookmarks = vm.read(cx).graph.bookmarks.clone();
        let bounds = Bounds::centered(
            None,
            Size {
                width: px(600.),
                height: px(480.),
            },
            cx,
        );
        let handle = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(TitlebarOptions {
                        title: Some("Bookmark Manager".into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |_, cx| {
                    cx.new(|cx| {
                        cx.observe_global::<AppConfigStore>(|_, cx| cx.notify())
                            .detach();
                        cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
                        cx.observe(&vm, |view: &mut Self, vm, cx| {
                            view.bookmarks = vm.read(cx).graph.bookmarks.clone();
                            cx.notify();
                        })
                        .detach();
                        let filter =
                            cx.new(|cx| TextArea::new("", "Filter bookmarks", false, 32., cx));
                        TextArea::subscribe_updates(&filter, cx);
                        Self {
                            repo,
                            parent,
                            vm,
                            bookmarks,
                            pr_host_name,
                            filter,
                            show_deleted: false,
                            loading: false,
                            error: None,
                            context_menu: None,
                            focus_handle: cx.focus_handle(),
                        }
                    })
                },
            )
            .ok();
        if let Some(handle) = handle {
            let _ = handle.update(cx, |view, window, cx| {
                observe_window_appearance(window, cx);
                let focus = view.filter.focus_handle(cx);
                window.focus(&focus, cx);
            });
        }
    }

    fn run_bookmark_action(
        &mut self,
        write: impl FnOnce(Arc<Repo>) -> CoreResult<()> + Send + 'static,
        on_success: impl FnOnce(&mut Self, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) {
        let task = self.vm.update(cx, |vm, cx| vm.bookmark_write(write, cx));
        self.loading = true;
        self.error = None;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, move |view, cx| {
                view.loading = false;
                match result {
                    Ok(()) => on_success(view, cx),
                    Err(error) => view.error = Some(format!("{error}").into()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn open_pull_request(&mut self, name: String, cx: &mut Context<Self>) {
        let repo = self.repo.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { repo.pull_request_open_url(&name) })
                .await;
            let _ = this.update(cx, move |view, cx| match result {
                Ok(url) => cx.open_url(&url),
                Err(error) => {
                    view.error = Some(format!("{error}").into());
                    cx.notify();
                }
            });
        })
        .detach();
    }

    fn reveal(&self, change_id: String, cx: &mut Context<Self>) {
        self.parent
            .update(cx, |view, cx| view.reveal_change_id(&change_id, cx));
    }

    fn show_diff(&self, bookmark: BookmarkInfo, cx: &mut Context<Self>) {
        if bookmark.change_id.is_empty() {
            return;
        }
        let request = revset::BookmarkDiffRequest {
            base: revset::trunk_endpoint(),
            head: revset::bookmark_endpoint_for_info(&bookmark),
            head_change_id: bookmark.change_id.id.clone(),
        };
        self.parent.update(cx, |view, cx| {
            view.vm
                .update(cx, |vm, cx| vm.compare_bookmark_diff(request.clone(), cx));
        });
    }

    fn open_context_menu(
        &mut self,
        anchor: Point<Pixels>,
        bookmark: BookmarkInfo,
        cx: &mut Context<Self>,
    ) {
        self.context_menu = Some(BookmarkContextMenuState {
            anchor,
            items: bookmark_menu_items(&bookmark, self.pr_host_name.as_deref()),
        });
        cx.notify();
    }

    fn close_context_menu(&mut self, cx: &mut Context<Self>) {
        if self.context_menu.take().is_some() {
            cx.notify();
        }
    }

    fn toggle_deleted(&mut self, cx: &mut Context<Self>) {
        self.show_deleted ^= true;
        cx.notify();
    }

    fn dispatch_context_action(&mut self, action: BookmarkContextAction, cx: &mut Context<Self>) {
        self.context_menu = None;
        match action {
            BookmarkContextAction::Reveal(change_id) => self.reveal(change_id, cx),
            BookmarkContextAction::ShowDiff(bookmark) => self.show_diff(bookmark, cx),
            BookmarkContextAction::Track { name, remote } => self.run_bookmark_action(
                move |repo| repo.track_bookmark(&name, &remote),
                |_, _| {},
                cx,
            ),
            BookmarkContextAction::Push(name) => {
                self.parent.update(cx, |view, cx| {
                    view.git_push_bookmark(name, cx);
                });
            }
            BookmarkContextAction::Resolve(name) => self.run_bookmark_action(
                move |repo| repo.move_bookmark(&name, "@"),
                |view, cx| view.parent.update(cx, |view, cx| view.git_fetch_origin(cx)),
                cx,
            ),
            BookmarkContextAction::OpenPullRequest(name) => self.open_pull_request(name, cx),
            BookmarkContextAction::Delete(name) => {
                self.run_bookmark_action(move |repo| repo.delete_bookmark(&name), |_, _| {}, cx)
            }
            BookmarkContextAction::Forget(name) => {
                self.run_bookmark_action(move |repo| repo.forget_bookmark(&name), |_, _| {}, cx)
            }
        }
        cx.notify();
    }
}

impl Focusable for BookmarkManagerView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for BookmarkManagerView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = theme(cx).clone();
        let context_menu_overlay = self
            .context_menu
            .as_ref()
            .map(|state| render_bookmark_context_menu(state, &t, &cx.entity()));
        let query = self.filter.read(cx).text().trim().to_lowercase();
        let stats = BookmarkStats::from_bookmarks(&self.bookmarks);
        let mut bookmarks: Vec<_> = self
            .bookmarks
            .iter()
            .filter(|bookmark| self.show_deleted || !bookmark.is_deleted)
            .cloned()
            .collect();
        bookmarks.sort_by_cached_key(|bookmark| bookmark.name.to_lowercase());
        if !query.is_empty() {
            bookmarks.retain(|bookmark| bookmark.name.to_lowercase().contains(&query));
        }
        let count = bookmarks.len();
        let body = if self.loading {
            placeholder("Loading bookmarks...", &t)
        } else if let Some(error) = self.error.clone() {
            placeholder_err(&error, &t)
        } else if count == 0 {
            placeholder("No bookmarks found.", &t)
        } else {
            bookmark_list(Arc::new(bookmarks), &t, cx)
        };

        let mut root = div()
            .track_focus(&self.focus_handle)
            .key_context("BookmarkManagerView")
            .on_action(cx.listener(|view, _: &CloseWindow, window, cx| {
                if view.context_menu.is_some() {
                    view.close_context_menu(cx);
                } else {
                    window.remove_window();
                }
            }))
            .on_action(cx.listener(|view, _: &Dismiss, window, cx| {
                if view.context_menu.is_some() {
                    view.close_context_menu(cx);
                } else {
                    window.remove_window();
                }
            }))
            .relative()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(t.detail_bg))
            .text_color(rgb(t.fg))
            .child(header(self.filter.clone(), &t))
            .child(stats_bar(stats, self.show_deleted, &t, cx))
            .child(body)
            .child(footer(&t, cx));
        if let Some(menu) = context_menu_overlay {
            root = root.child(menu);
        }
        root
    }
}
