use std::borrow::Cow;
use std::path::PathBuf;

use gpui::{
    App, AppContext, AssetSource, Bounds, Focusable, Point, SharedString, Size, TitlebarOptions,
    WindowBounds, WindowOptions, px, size,
};

use jayjay_gpui::app::config::{AppConfig, AppConfigStore};
use jayjay_gpui::app::theme::{Theme, observe_window_appearance};
use jayjay_gpui::repo::RepoWindow;
use jayjay_gpui::windows::repo_list::RepoListWindow;

const LUCIDE_FONT: &[u8] = include_bytes!("../assets/fonts/Lucide.ttf");
const REFRESH_CW_SVG: &[u8] = include_bytes!("../assets/icons/refresh-cw.svg");
const LOGO_SVG: &[u8] = include_bytes!("../assets/icons/logo.svg");

struct GpuiAssets;

impl AssetSource for GpuiAssets {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        match path {
            jayjay_gpui::ui::icons::REFRESH_CW_SVG => Ok(Some(Cow::Borrowed(REFRESH_CW_SVG))),
            jayjay_gpui::ui::icons::LOGO_SVG => Ok(Some(Cow::Borrowed(LOGO_SVG))),
            _ => Ok(None),
        }
    }

    fn list(&self, _path: &str) -> gpui::Result<Vec<SharedString>> {
        Ok(Vec::new())
    }
}

fn resolve_repo_path() -> PathBuf {
    let raw = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."));
    raw.canonicalize().unwrap_or(raw)
}

fn main() {
    // CLI commands must run before any GPUI/window init, so `jayjay-gpui review notes` works headlessly with no display server.
    let arguments: Vec<String> = std::env::args().collect();
    if let Some(code) = jayjay_gpui::cli::run_and_exit_if_needed(&arguments[1..]) {
        std::process::exit(code);
    }

    jayjay_gpui::app::cli_install::repair_broken_link();

    let path = resolve_repo_path();
    let title: String = match path.file_name().and_then(|s| s.to_str()) {
        Some(name) if !name.is_empty() => format!("JayJay (Alpha) — {name}"),
        _ => "JayJay (Alpha)".to_string(),
    };

    let app = gpui_platform::application().with_assets(GpuiAssets);
    app.run(move |cx: &mut App| {
        match cx.text_system().add_fonts(vec![Cow::Borrowed(LUCIDE_FONT)]) {
            Ok(()) => eprintln!(
                "[jayjay-gpui] registered Lucide font ({} bytes)",
                LUCIDE_FONT.len()
            ),
            Err(e) => eprintln!("[jayjay-gpui] failed to register Lucide: {e}"),
        }

        let cfg = AppConfig::load();
        jayjay_gpui::app::telemetry::maybe_ping(cfg.telemetry.enabled);
        let initial_appearance = cfg.appearance;
        let show_onboarding = !cfg.onboarding.completed;
        cx.set_global(Theme::for_appearance(
            cfg.appearance,
            cx.window_appearance(),
        ));

        cx.bind_keys(jayjay_gpui::app::actions::app_key_bindings());

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

        cx.set_global(AppConfigStore::new(cfg));
        jayjay_gpui::app::repositories::install(cx);
        if !show_onboarding {
            jayjay_gpui::app::config::update(cx, |c| c.record_opened_repo(&path));
        }
        jayjay_gpui::app::menus::install(cx);
        let window_handle = cx.open_window(
            WindowOptions {
                window_bounds: Some(initial_window_bounds),
                titlebar: Some(TitlebarOptions {
                    title: Some(title.clone().into()),
                    appears_transparent: true,
                    traffic_light_position: Some(Point {
                        x: px(12.),
                        y: px(14.),
                    }),
                }),
                ..Default::default()
            },
            move |window, cx| {
                cx.set_global(Theme::for_appearance(
                    initial_appearance,
                    window.appearance(),
                ));
                cx.new(|cx| {
                    cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
                    cx.observe_global::<AppConfigStore>(|_, cx| cx.notify())
                        .detach();
                    cx.observe_global::<jayjay_gpui::app::repositories::StoreHandle>(|_, cx| {
                        cx.notify()
                    })
                    .detach();
                    let mut view = if show_onboarding {
                        RepoWindow::new_with_onboarding(path.clone(), cx)
                    } else {
                        RepoWindow::new(path.clone(), cx)
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
            observe_window_appearance(window, cx);
            let handle = view.focus_handle(cx);
            window.focus(&handle, cx);

            window.on_window_should_close(cx, |window, cx| {
                let wb = window.window_bounds();
                let (bounds, maximized) = match wb {
                    WindowBounds::Windowed(b) => (b, false),
                    WindowBounds::Maximized(b) => (b, true),
                    WindowBounds::Fullscreen(b) => (b, false),
                };
                jayjay_gpui::app::config::update(cx, move |c| {
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
    });
}
