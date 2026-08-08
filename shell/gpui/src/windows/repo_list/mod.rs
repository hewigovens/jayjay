mod view;

use gpui::{
    App, AppContext, Bounds, Context, FocusHandle, Focusable, InteractiveElement, IntoElement,
    ParentElement, Render, Styled, TitlebarOptions, Window, WindowBounds, WindowOptions, div, px,
    rgb, size,
};

use crate::app::config::{self, AppConfigStore};
use crate::app::repositories::{self, StoreHandle};
use crate::app::theme::{Theme, observe_window_appearance, theme};
use crate::repo::RepoWindow;
use crate::ui::logo::Logo;
use crate::ui::primitives::divider_h;

const WINDOW_WIDTH: f32 = 480.;
const WINDOW_HEIGHT: f32 = 600.;

pub struct RepoListWindow {
    focus_handle: FocusHandle,
    logo: Logo,
}

impl RepoListWindow {
    pub fn open(cx: &mut App) {
        repositories::ensure(cx);
        if let Some(handle) = cx
            .windows()
            .into_iter()
            .find_map(|handle| handle.downcast::<Self>())
        {
            let _ = handle.update(cx, |view, window, cx| {
                window.activate_window();
                window.focus(&view.focus_handle, cx);
            });
            return;
        }

        let window_size = size(px(WINDOW_WIDTH), px(WINDOW_HEIGHT));
        let handle = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                        None,
                        window_size,
                        cx,
                    ))),
                    window_min_size: Some(window_size),
                    titlebar: Some(TitlebarOptions {
                        title: Some("JayJay".into()),
                        appears_transparent: true,
                        traffic_light_position: Some(gpui::Point {
                            x: px(12.),
                            y: px(14.),
                        }),
                    }),
                    ..Default::default()
                },
                |_, cx| {
                    cx.new(|cx| {
                        cx.observe_global::<AppConfigStore>(|_, cx| cx.notify())
                            .detach();
                        cx.observe_global::<StoreHandle>(|_, cx| cx.notify())
                            .detach();
                        cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
                        Self {
                            focus_handle: cx.focus_handle(),
                            logo: Logo::load(cx),
                        }
                    })
                },
            )
            .ok();

        if let Some(handle) = handle {
            let _ = handle.update(cx, |view, window, cx| {
                observe_window_appearance(window, cx);
                window.focus(&view.focus_handle, cx);
            });
        }
    }

    pub fn open_if_last_repo_window(cx: &mut App) {
        let repo_windows = cx
            .windows()
            .iter()
            .filter(|handle| handle.downcast::<RepoWindow>().is_some())
            .count();
        if repo_windows <= 1 {
            Self::open(cx);
        }
    }

    pub(crate) fn close(cx: &mut App) {
        let handles: Vec<_> = cx
            .windows()
            .into_iter()
            .filter_map(|handle| handle.downcast::<Self>())
            .collect();
        for handle in handles {
            let _ = handle.update(cx, |_, window, _| window.remove_window());
        }
    }
}

impl Focusable for RepoListWindow {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for RepoListWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = theme(cx).clone();
        let pinned = repositories::current(cx);
        let pinned_set = pinned.iter().collect::<std::collections::HashSet<_>>();
        let recent = config::current(cx)
            .recent_repos
            .iter()
            .filter(|path| !pinned_set.contains(path))
            .cloned()
            .collect::<Vec<_>>();
        let has_repositories = !pinned.is_empty() || !recent.is_empty();
        let content = if has_repositories {
            div()
                .flex()
                .flex_1()
                .min_h_0()
                .flex_col()
                .child(
                    div()
                        .px(px(30.))
                        .pt(px(30.))
                        .pb(px(22.))
                        .child(view::header(&self.logo, &t)),
                )
                .child(divider_h(&t))
                .child(view::repository_sections(pinned, recent, &t))
        } else {
            div()
                .flex()
                .flex_1()
                .items_center()
                .justify_center()
                .child(view::header(&self.logo, &t))
        };

        div()
            .id("repo-list-window")
            .debug_selector(|| "repo-list-window".to_owned())
            .track_focus(&self.focus_handle)
            .key_context("RepoListWindow")
            .on_action(
                cx.listener(|_, _: &crate::app::actions::CloseWindow, window, _| {
                    window.remove_window();
                }),
            )
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(t.detail_bg))
            .text_color(rgb(t.fg))
            .child(content)
    }
}
