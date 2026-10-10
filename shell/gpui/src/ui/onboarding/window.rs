use gpui::{
    App, AppContext, Bounds, Point, TitlebarOptions, WindowBounds, WindowOptions, px, size,
};

use super::{OnboardingCompleted, OnboardingView};
use crate::app::theme::{Theme, observe_window_appearance};

const WINDOW_WIDTH: f32 = 480.;
const WINDOW_HEIGHT: f32 = 460.;

impl OnboardingView {
    pub(crate) fn open_window(on_finish: impl FnOnce(&mut App) + 'static, cx: &mut App) {
        let bounds = Bounds::centered(None, size(px(WINDOW_WIDTH), px(WINDOW_HEIGHT)), cx);
        let handle = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_min_size: Some(bounds.size),
                is_minimizable: false,
                titlebar: Some(TitlebarOptions {
                    title: Some("Welcome to JayJay".into()),
                    appears_transparent: true,
                    traffic_light_position: Some(Point {
                        x: px(crate::platform::REPO_TRAFFIC_LIGHTS.0),
                        y: px(crate::platform::REPO_TRAFFIC_LIGHTS.1),
                    }),
                }),
                ..crate::app::window_options()
            },
            |window, cx| {
                cx.new(|cx| {
                    cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
                    observe_window_appearance(window, cx);
                    Self::new(cx)
                })
            },
        );
        let Some((handle, view)) = handle
            .ok()
            .and_then(|handle| Some((handle, handle.entity(cx).ok()?)))
        else {
            eprintln!("[jayjay-gpui] failed to open onboarding window");
            cx.quit();
            return;
        };
        let mut on_finish = Some(on_finish);
        cx.subscribe(&view, move |_, _: &OnboardingCompleted, cx| {
            if let Some(on_finish) = on_finish.take() {
                on_finish(cx);
            }
            let _ = handle.update(cx, |_, window, _| window.remove_window());
        })
        .detach();
        cx.activate(true);
    }
}
