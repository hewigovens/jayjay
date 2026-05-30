use std::path::PathBuf;

use gpui::{
    App, AppContext, Bounds, Focusable, Point, Size, TitlebarOptions, WindowBounds, WindowOptions,
    px,
};

use super::view::LogView;

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
                y: px(14.),
            }),
        }),
        ..Default::default()
    };
    let handle = cx.open_window(opts, move |_, cx| {
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
    if let Ok(handle) = handle {
        let _ = handle.update(cx, |view, window, cx| {
            crate::app::theme::observe_window_appearance(window, cx);
            let focus = view.focus_handle(cx);
            window.focus(&focus, cx);
        });
    }
}
