use std::sync::Arc;

mod chrome;
mod context_menu;
mod rows;

use gpui::{
    App, AppContext, Bounds, ClipboardItem, Context, Entity, FocusHandle, Focusable,
    InteractiveElement, IntoElement, ParentElement, Pixels, Point, Render, SharedString, Size,
    Styled, TitlebarOptions, Window, WindowBounds, WindowOptions, div, px, rgb,
};
use jayjay_core::{BookmarkInfo, Repo};

use crate::app::actions::CloseWindow;
use crate::app::config::AppConfigStore;
use crate::app::theme::{Theme, theme};
use crate::repo::revset;
use crate::repo::window::RepoWindow;
use crate::ui::text_area::TextArea;
use chrome::{header, placeholder, placeholder_err};
use context_menu::render_context_menu as render_bookmark_context_menu;
use context_menu::{BookmarkContextAction, BookmarkContextMenuState, bookmark_menu_items};
use rows::bookmark_list;

pub struct BookmarkManagerView {
    repo: Arc<Repo>,
    parent: Entity<RepoWindow>,
    bookmarks: Arc<Vec<BookmarkInfo>>,
    filter: Entity<TextArea>,
    loading: bool,
    error: Option<SharedString>,
    context_menu: Option<BookmarkContextMenuState>,
    focus_handle: FocusHandle,
}

impl BookmarkManagerView {
    pub fn open(
        repo: Arc<Repo>,
        parent: Entity<RepoWindow>,
        bookmarks: Arc<Vec<BookmarkInfo>>,
        cx: &mut App,
    ) {
        let bounds = Bounds::centered(
            None,
            Size {
                width: px(760.),
                height: px(560.),
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
                        let filter =
                            cx.new(|cx| TextArea::new("", "Filter bookmarks", false, 32., cx));
                        Self {
                            repo,
                            parent,
                            bookmarks,
                            filter,
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
                crate::app::theme::observe_window_appearance(window, cx);
                let focus = view.filter.focus_handle(cx);
                window.focus(&focus, cx);
            });
        }
    }

    fn reload(&mut self, cx: &mut Context<Self>) {
        let repo = self.repo.clone();
        self.loading = true;
        self.error = None;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { repo.list_bookmarks() })
                .await;
            let _ = this.update(cx, move |view, cx| {
                view.loading = false;
                match result {
                    Ok(bookmarks) => view.bookmarks = Arc::new(bookmarks),
                    Err(error) => view.error = Some(format!("{error}").into()),
                }
                view.refresh_parent(cx);
                cx.notify();
            });
        })
        .detach();
    }

    fn refresh_parent(&self, cx: &mut Context<Self>) {
        let parent = self.parent.clone();
        parent.update(cx, |view, cx| {
            let vm = view.vm.clone();
            vm.update(cx, |vm, cx| vm.refresh(false, cx));
        });
    }

    pub(super) fn track_bookmark(&mut self, name: String, remote: String, cx: &mut Context<Self>) {
        let repo = self.repo.clone();
        self.loading = true;
        self.error = None;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { repo.track_bookmark(&name, &remote) })
                .await;
            let _ = this.update(cx, move |view, cx| {
                view.loading = false;
                if let Err(error) = result {
                    view.error = Some(format!("{error}").into());
                    cx.notify();
                } else {
                    view.reload(cx);
                }
            });
        })
        .detach();
    }

    pub(super) fn reveal(&self, change_id: String, cx: &mut Context<Self>) {
        self.parent
            .update(cx, |view, cx| view.reveal_change_id(&change_id, cx));
    }

    pub(super) fn show_diff(&self, bookmark: BookmarkInfo, cx: &mut Context<Self>) {
        if bookmark.change_id.is_empty() {
            return;
        }
        let request = revset::BookmarkDiffRequest {
            base: revset::trunk_endpoint(),
            head: revset::bookmark_endpoint_for_info(&bookmark),
            head_change_id: bookmark.change_id.clone(),
        };
        self.parent.update(cx, |view, cx| {
            view.vm
                .update(cx, |vm, cx| vm.compare_bookmark_diff(request.clone(), cx));
        });
    }

    pub(super) fn open_context_menu(
        &mut self,
        anchor: Point<Pixels>,
        bookmark: BookmarkInfo,
        cx: &mut Context<Self>,
    ) {
        self.context_menu = Some(BookmarkContextMenuState {
            anchor,
            items: bookmark_menu_items(&bookmark),
        });
        cx.notify();
    }

    fn close_context_menu(&mut self, cx: &mut Context<Self>) {
        if self.context_menu.take().is_some() {
            cx.notify();
        }
    }

    fn dispatch_context_action(&mut self, action: BookmarkContextAction, cx: &mut Context<Self>) {
        self.context_menu = None;
        match action {
            BookmarkContextAction::Reveal(change_id) => self.reveal(change_id, cx),
            BookmarkContextAction::ShowDiff(bookmark) => self.show_diff(bookmark, cx),
            BookmarkContextAction::CopyName(name) => {
                cx.write_to_clipboard(ClipboardItem::new_string(name));
            }
            BookmarkContextAction::Track { name, remote } => self.track_bookmark(name, remote, cx),
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
        let query = self.filter.read(cx).text().trim().to_ascii_lowercase();
        let mut bookmarks: Vec<_> = self.bookmarks.iter().cloned().collect();
        bookmarks.sort_by(|a, b| a.name.cmp(&b.name));
        if !query.is_empty() {
            bookmarks.retain(|bookmark| {
                bookmark.name.to_ascii_lowercase().contains(&query)
                    || bookmark.description.to_ascii_lowercase().contains(&query)
                    || bookmark
                        .available_remotes
                        .iter()
                        .any(|remote| remote.to_ascii_lowercase().contains(&query))
            });
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
            .relative()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(t.detail_bg))
            .text_color(rgb(t.fg))
            .child(header(count, self.filter.clone(), &t))
            .child(body);
        if let Some(menu) = context_menu_overlay {
            root = root.child(menu);
        }
        root
    }
}
