use std::path::PathBuf;

use gpui::{App, Bounds, Point, Size, WindowBounds, px, size};

use super::config;
use crate::repo::RepoWindow;
use crate::windows::repo_list::RepoListWindow;

pub(super) fn resolve_repo_path(
    explicit: Option<PathBuf>,
    cwd: Option<PathBuf>,
    recent_repos: &[String],
) -> Option<PathBuf> {
    if let Some(path) = explicit {
        return Some(path.canonicalize().unwrap_or(path));
    }
    cwd.into_iter()
        .chain(recent_repos.iter().map(PathBuf::from))
        .find_map(|path| {
            jayjay_core::workspace_root(&path)
                .map(|root| jayjay_core::repositories::normalize_repository_path(&root))
        })
}

pub(super) fn open_startup_window(path: Option<PathBuf>, cx: &mut App) {
    let Some(path) = path else {
        RepoListWindow::open(cx);
        cx.activate(true);
        return;
    };
    let cfg = config::current(cx);
    let show_onboarding = !cfg.onboarding.completed;
    let initial_bounds = if cfg.window.is_set() {
        Bounds {
            origin: Point {
                x: px(cfg.window.x),
                y: px(cfg.window.y),
            },
            size: Size {
                width: px(cfg.window.width),
                height: px(cfg.window.height),
            },
        }
    } else {
        Bounds::centered(None, size(px(1080.), px(720.)), cx)
    };
    let initial_window_bounds = if cfg.window.maximized {
        WindowBounds::Maximized(initial_bounds)
    } else {
        WindowBounds::Windowed(initial_bounds)
    };

    let window_handle = match RepoWindow::open(path, initial_window_bounds, show_onboarding, cx) {
        Ok(handle) => handle,
        Err(error) => {
            eprintln!("[jayjay-gpui] failed to open window: {error}");
            cx.quit();
            return;
        }
    };
    let _ = window_handle.update(cx, |_, window, cx| {
        window.on_window_should_close(cx, |window, cx| {
            let (bounds, maximized) = match window.window_bounds() {
                WindowBounds::Maximized(bounds) => (bounds, true),
                WindowBounds::Windowed(bounds) | WindowBounds::Fullscreen(bounds) => {
                    (bounds, false)
                }
            };
            config::update(cx, move |c| {
                c.window.x = f32::from(bounds.origin.x);
                c.window.y = f32::from(bounds.origin.y);
                c.window.width = f32::from(bounds.size.width);
                c.window.height = f32::from(bounds.size.height);
                c.window.maximized = maximized;
            });
            RepoListWindow::open_if_last_repo_window(cx);
            true
        });
    });
    cx.activate(true);
}

#[cfg(test)]
mod tests {
    use super::resolve_repo_path;
    use jj_test::LinearFixture;

    #[test]
    fn startup_repo_precedence_and_invalid_recents() {
        let first = LinearFixture::build();
        let second = LinearFixture::build();
        let invalid = tempfile::tempdir().unwrap();
        let broken = tempfile::tempdir().unwrap();
        std::fs::create_dir(broken.path().join(".jj")).unwrap();
        let recent = vec![
            invalid.path().join("missing").display().to_string(),
            invalid.path().display().to_string(),
            broken.path().display().to_string(),
            second.path.display().to_string(),
            first.path.display().to_string(),
        ];
        let cwd = Some(first.path.clone());
        assert_eq!(
            resolve_repo_path(Some(invalid.path().to_owned()), cwd.clone(), &recent),
            Some(invalid.path().canonicalize().unwrap()),
        );
        assert_eq!(
            resolve_repo_path(None, cwd, &recent),
            Some(first.path.canonicalize().unwrap())
        );
        let nested = first.path.join("src/nested");
        std::fs::create_dir_all(&nested).unwrap();
        assert_eq!(
            resolve_repo_path(None, Some(nested), &recent),
            Some(first.path.canonicalize().unwrap())
        );
        for cwd in [Some(invalid.path().to_owned()), None] {
            assert_eq!(
                resolve_repo_path(None, cwd.clone(), &recent),
                Some(second.path.canonicalize().unwrap())
            );
            assert_eq!(resolve_repo_path(None, cwd, &recent[..3]), None);
        }
    }
}
