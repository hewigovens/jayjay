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
use crate::ui::overlay::{PromptSlots, PromptStyle, TextPrompt};
use crate::ui::text_area::{Newline, TextArea};
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
    rename: Option<TextPrompt>,
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
                    ..crate::app::window_options()
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
                            rename: None,
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
                Ok(url) => crate::app::links::open_url(cx, &url),
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

    fn open_rename_modal(&mut self, name: String, cx: &mut Context<Self>) {
        self.error = None;
        self.rename = Some(TextPrompt::single_line(
            "Rename Bookmark",
            name.clone(),
            name,
            "New name",
            "Rename",
            cx,
        ));
        cx.notify();
    }

    fn close_rename_modal(&mut self, cx: &mut Context<Self>) {
        if self.rename.take().is_some() {
            self.error = None;
            cx.notify();
        }
    }

    fn ready_rename(&self, cx: &App) -> Option<(String, String)> {
        let rename = self.rename.as_ref()?;
        if self.loading {
            return None;
        }
        let new_name = rename.text(cx);
        let new_name = new_name.trim();
        if new_name.is_empty()
            || new_name == rename.subtitle.as_ref()
            || !jayjay_core::is_valid_bookmark_name(new_name)
        {
            return None;
        }
        Some((rename.subtitle.to_string(), new_name.to_string()))
    }

    fn submit_rename(&mut self, cx: &mut Context<Self>) {
        let Some((old_name, new_name)) = self.ready_rename(cx) else {
            return;
        };
        self.run_bookmark_action(
            move |repo| repo.rename_bookmark(&old_name, &new_name),
            |view, cx| view.close_rename_modal(cx),
            cx,
        );
    }

    fn dismiss_overlay(&mut self, cx: &mut Context<Self>) -> bool {
        if self.rename.is_some() {
            self.close_rename_modal(cx);
            true
        } else if self.context_menu.is_some() {
            self.close_context_menu(cx);
            true
        } else {
            false
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
            BookmarkContextAction::Rename(name) => self.open_rename_modal(name, cx),
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
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(rename) = self.rename.as_mut() {
            rename.take_focus(window, cx);
        }
        let t = theme(cx).clone();
        let context_menu_overlay = self
            .context_menu
            .as_ref()
            .map(|state| render_bookmark_context_menu(state, &t, &cx.entity()));
        let can_submit_rename = self.ready_rename(cx).is_some();
        let rename_error = self.error.clone();
        let rename_modal_overlay = self.rename.as_ref().map(|rename| {
            rename.overlay(
                &PromptStyle {
                    input_id: Some("bookmark-rename-input"),
                    primary_enabled: can_submit_rename,
                    ..PromptStyle::new(360., "bookmark-rename-cancel", "bookmark-rename-primary")
                },
                &t,
                cx,
                PromptSlots::new(
                    [],
                    rename_error.map(|message| {
                        div()
                            .id("bookmark-rename-error")
                            .debug_selector(|| "bookmark-rename-error".to_owned())
                            .text_size(px(12.))
                            .text_color(rgb(t.error_fg))
                            .child(message)
                            .into_any_element()
                    }),
                ),
                |view, cx| view.close_rename_modal(cx),
                |view, cx| view.submit_rename(cx),
            )
        });
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
        let body = if self.rename.is_none() && self.loading {
            placeholder("Loading bookmarks...", &t)
        } else if self.rename.is_none()
            && let Some(error) = self.error.clone()
        {
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
                if !view.dismiss_overlay(cx) {
                    window.remove_window();
                }
            }))
            .on_action(cx.listener(|view, _: &Dismiss, window, cx| {
                if !view.dismiss_overlay(cx) {
                    window.remove_window();
                }
            }))
            .on_action(cx.listener(|view, _: &Newline, _, cx| {
                if view.rename.is_some() {
                    view.submit_rename(cx);
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
        if let Some(modal) = rename_modal_overlay {
            root = root.child(modal);
        }
        root
    }
}
