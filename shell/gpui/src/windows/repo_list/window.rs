use gpui::{
    App, AppContext, Bounds, FocusHandle, Focusable, TitlebarOptions, WindowBounds, WindowOptions,
    px, size,
};
use jayjay_core::repositories::RepoListGroups;

use crate::app::config::AppConfigStore;
use crate::app::repositories::{self, StoreHandle};
use crate::app::theme::{Theme, observe_window_appearance};
use crate::repo::RepoWindow;
use crate::ui::logo::Logo;

const WINDOW_WIDTH: f32 = 480.;
const WINDOW_HEIGHT: f32 = 600.;

pub struct RepoListWindow {
    pub(super) focus_handle: FocusHandle,
    pub(super) logo: Logo,
    pub(super) pinned: Vec<String>,
    pub(super) recent: Vec<String>,
    pub(super) groups: RepoListGroups,
    pub(super) grouping_generation: u64,
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
                    ..crate::app::window_options()
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
                            pinned: Vec::new(),
                            recent: Vec::new(),
                            groups: RepoListGroups::default(),
                            grouping_generation: 0,
                        }
                    })
                },
            )
            .ok();

        if let Some(handle) = handle {
            let _ = handle.update(cx, |view, window, cx| {
                observe_window_appearance(window, cx);
                window.focus(&view.focus_handle, cx);
                cx.observe_window_activation(window, |view, window, cx| {
                    if window.is_window_active() {
                        view.regroup(cx);
                    }
                })
                .detach();
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
