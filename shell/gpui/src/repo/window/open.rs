use std::path::{Path, PathBuf};

use gpui::{
    AnyWindowHandle, App, AppContext, Bounds, Point, Size, TitlebarOptions, WindowBounds,
    WindowOptions, px,
};
use jayjay_core::repositories::normalize_repository_path;

use crate::app::config;
use crate::app::theme::Theme;
use crate::windows::repo_list::RepoListWindow;

use super::view::RepoWindow;

pub fn open_repo_window(path: PathBuf, cx: &mut App) {
    let path = normalize_repository_path(&path);
    config::update(cx, |config| config.record_opened_repo(&path));
    if activate_normalized_repo_window(&path, cx) {
        RepoListWindow::close(cx);
        return;
    }
    let title = match path.file_name().and_then(|s| s.to_str()) {
        Some(name) if !name.is_empty() => format!("JayJay (Beta) — {name}"),
        _ => "JayJay (Beta)".to_string(),
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
        ..crate::app::window_options()
    };
    let handle = cx.open_window(opts, move |_, cx| {
        cx.new(|cx| {
            cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
            cx.observe_global::<crate::app::config::AppConfigStore>(|_, cx| cx.notify())
                .detach();
            cx.observe_global::<crate::app::repositories::StoreHandle>(|_, cx| cx.notify())
                .detach();
            let mut view = RepoWindow::new(path.clone(), cx);
            view.boot(cx);
            view
        })
    });
    if let Ok(handle) = handle {
        let _ = handle.update(cx, |view, window, cx| {
            view.attach_to_window(window, cx);
            window.on_window_should_close(cx, |_, cx| {
                RepoListWindow::open_if_last_repo_window(cx);
                true
            });
        });
        cx.defer(RepoListWindow::close);
    }
}

pub(crate) fn activate_repo_window(path: &Path, cx: &mut App) -> bool {
    let normalized = normalize_repository_path(path);
    activate_normalized_repo_window(&normalized, cx)
}

fn activate_normalized_repo_window(normalized: &Path, cx: &mut App) -> bool {
    let handle = cx.windows().into_iter().find_map(|handle| {
        let handle = handle.downcast::<RepoWindow>()?;
        let view = handle.read(cx).ok()?;
        let open_path = PathBuf::from(view.vm.read(cx).repo_path.as_ref());
        (normalize_repository_path(&open_path) == normalized).then_some(handle)
    });
    let Some(handle) = handle else { return false };
    let _ = handle.update(cx, |view, window, cx| {
        window.activate_window();
        window.focus(&view.focus_handle, cx);
    });
    true
}

pub(crate) fn close_repo_window_at(path: &Path, cx: &mut App) {
    let normalized = normalize_repository_path(path);
    let handles: Vec<_> = cx
        .windows()
        .into_iter()
        .filter_map(|handle| {
            let handle = handle.downcast::<RepoWindow>()?;
            let view = handle.read(cx).ok()?;
            let open_path = PathBuf::from(view.vm.read(cx).repo_path.as_ref());
            (normalize_repository_path(&open_path) == normalized).then_some(handle)
        })
        .collect();
    for handle in handles {
        let _ = handle.update(cx, |_, window, _| window.remove_window());
    }
}

pub(crate) fn open_repo_paths(
    current_window: AnyWindowHandle,
    current_path: &str,
    cx: &App,
) -> Vec<String> {
    let mut paths = vec![current_path.to_owned()];
    for handle in cx.windows() {
        if handle == current_window {
            continue;
        }
        let Some(handle) = handle.downcast::<RepoWindow>() else {
            continue;
        };
        let Ok(view) = handle.read(cx) else { continue };
        paths.push(view.vm.read(cx).repo_path.to_string());
    }
    let mut seen = std::collections::HashSet::new();
    paths.retain(|path| seen.insert(path.clone()));
    paths
}
