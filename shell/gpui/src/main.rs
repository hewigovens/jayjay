use std::borrow::Cow;
use std::path::PathBuf;

use gpui::{
    App, AppContext, AssetSource, Bounds, Point, SharedString, Size, TitlebarOptions, WindowBounds,
    WindowOptions, px, size,
};

use jayjay_gpui::app::config::{self, AppConfig, AppConfigStore};
use jayjay_gpui::app::theme::Theme;
use jayjay_gpui::repo::RepoWindow;
use jayjay_gpui::windows::repo_list::RepoListWindow;

const LUCIDE_FONT: &[u8] = include_bytes!("../assets/fonts/Lucide.ttf");

struct GpuiAssets;

impl AssetSource for GpuiAssets {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        Ok(jayjay_gpui::ui::icons::SVG_ASSETS
            .iter()
            .find(|(name, _)| *name == path)
            .map(|(_, bytes)| Cow::Borrowed(*bytes)))
    }

    fn list(&self, _path: &str) -> gpui::Result<Vec<SharedString>> {
        Ok(Vec::new())
    }
}

fn resolve_repo_path(
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

fn main() {
    // CLI commands must run before any GPUI/window init, so `jayjay-gpui review notes` works headlessly with no display server.
    let arguments: Vec<String> = std::env::args().collect();
    if let Some(code) = jayjay_gpui::cli::run_and_exit_if_needed(&arguments[1..]) {
        std::process::exit(code);
    }
    let external_tool =
        match jayjay_gpui::external_tool::parse_external_tool_invocation(&arguments[1..]) {
            Ok(invocation) => invocation,
            Err(message) => {
                eprintln!("error: {message}");
                std::process::exit(1);
            }
        };

    jayjay_gpui::app::cli_install::repair_broken_link();

    let cfg = AppConfig::load();

    let app = gpui_platform::application().with_assets(GpuiAssets);
    app.run(move |cx: &mut App| {
        match cx.text_system().add_fonts(vec![Cow::Borrowed(LUCIDE_FONT)]) {
            Ok(()) => eprintln!(
                "[jayjay-gpui] registered Lucide font ({} bytes)",
                LUCIDE_FONT.len()
            ),
            Err(e) => eprintln!("[jayjay-gpui] failed to register Lucide: {e}"),
        }

        if external_tool.is_none() {
            jayjay_gpui::app::telemetry::maybe_ping(cfg.telemetry.enabled);
        }
        cx.set_global(
            Theme::for_appearance(cfg.appearance, cx.window_appearance())
                .with_font_size(cfg.font_size()),
        );

        cx.bind_keys(jayjay_gpui::app::actions::app_key_bindings());

        if let Some(invocation) = external_tool.clone() {
            cx.set_global(AppConfigStore::new(cfg));
            if let Err(error) = jayjay_gpui::external_tool::open_external_tool(invocation, cx) {
                eprintln!("[jayjay-gpui] failed to open external tool: {error}");
                std::process::exit(1);
            }
            cx.activate(true);
            return;
        }

        let recent_repos = cfg.recent_repos.clone();
        cx.set_global(AppConfigStore::new(cfg));
        jayjay_gpui::app::repositories::install(cx);
        jayjay_gpui::app::menus::install(cx);
        cx.spawn(async move |cx| {
            let path = cx
                .background_spawn(async move {
                    resolve_repo_path(
                        arguments.get(1).map(PathBuf::from),
                        std::env::current_dir().ok(),
                        &recent_repos,
                    )
                })
                .await;
            cx.update(|cx| open_startup_window(path, cx));
        })
        .detach();
    });
}

fn open_startup_window(path: Option<PathBuf>, cx: &mut App) {
    let Some(path) = path else {
        RepoListWindow::open(cx);
        cx.activate(true);
        return;
    };
    let cfg = config::current(cx);
    let initial_appearance = cfg.appearance;
    let initial_font_size = cfg.font_size();
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

    let title = match path.file_name().and_then(|s| s.to_str()) {
        Some(name) if !name.is_empty() => format!("JayJay (Beta) — {name}"),
        _ => "JayJay (Beta)".to_string(),
    };
    let window_handle = cx.open_window(
        WindowOptions {
            window_bounds: Some(initial_window_bounds),
            titlebar: Some(TitlebarOptions {
                title: Some(title.into()),
                appears_transparent: true,
                traffic_light_position: Some(Point {
                    x: px(12.),
                    y: px(14.),
                }),
            }),
            ..jayjay_gpui::app::window_options()
        },
        move |window, cx| {
            cx.set_global(
                Theme::for_appearance(initial_appearance, window.appearance())
                    .with_font_size(initial_font_size),
            );
            cx.new(|cx| {
                cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
                cx.observe_global::<AppConfigStore>(|_, cx| cx.notify())
                    .detach();
                cx.observe_global::<jayjay_gpui::app::repositories::StoreHandle>(|_, cx| {
                    cx.notify()
                })
                .detach();
                let mut view = if show_onboarding {
                    RepoWindow::new_with_onboarding(path, cx)
                } else {
                    RepoWindow::new(path, cx)
                };
                view.boot(cx);
                view
            })
        },
    );
    let window_handle = match window_handle {
        Ok(handle) => handle,
        Err(error) => {
            eprintln!("[jayjay-gpui] failed to open window: {error}");
            cx.quit();
            return;
        }
    };
    let _ = window_handle.update(cx, |view, window, cx| {
        view.attach_to_window(window, cx);
        window.on_window_should_close(cx, |window, cx| {
            let wb = window.window_bounds();
            let (bounds, maximized) = match wb {
                WindowBounds::Windowed(b) => (b, false),
                WindowBounds::Maximized(b) => (b, true),
                WindowBounds::Fullscreen(b) => (b, false),
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
