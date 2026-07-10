use std::borrow::Cow;
use std::path::PathBuf;

use gpui::{
    App, AppContext, AssetSource, Bounds, Focusable, KeyBinding, Point, SharedString, Size,
    TitlebarOptions, WindowBounds, WindowOptions, px, size,
};

use jayjay_gpui::app::actions::{
    CloseWindow, CopyDiffSelection, Dismiss, OpenBookmarkManager, OpenCommandPalette, OpenFind,
    OpenOperationLog, OpenRepository, OpenSettings, Quit, Refresh, ResetZoom, SaveNoteComposer,
    ShowRepoInFileManager, ZoomIn, ZoomOut,
};
use jayjay_gpui::app::config::{AppConfig, AppConfigStore};
use jayjay_gpui::app::theme::{Theme, observe_window_appearance};
use jayjay_gpui::platform::MOD_KEY;
use jayjay_gpui::repo::RepoWindow;
use jayjay_gpui::ui::text_area;

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

        let mod_key = MOD_KEY;
        let mut key_bindings = vec![
            KeyBinding::new(format!("{mod_key}-o").as_str(), OpenRepository, None),
            KeyBinding::new(format!("{mod_key}-,").as_str(), OpenSettings, None),
            KeyBinding::new(format!("{mod_key}-q").as_str(), Quit, None),
            KeyBinding::new(format!("{mod_key}-w").as_str(), CloseWindow, None),
            KeyBinding::new("escape", Dismiss, None),
            KeyBinding::new(format!("{mod_key}-r").as_str(), Refresh, None),
            KeyBinding::new(format!("{mod_key}-+").as_str(), ZoomIn, None),
            KeyBinding::new(format!("{mod_key}--").as_str(), ZoomOut, None),
            KeyBinding::new(format!("{mod_key}-0").as_str(), ResetZoom, None),
            KeyBinding::new(
                format!("{mod_key}-shift-p").as_str(),
                OpenCommandPalette,
                None,
            ),
            KeyBinding::new(
                format!("{mod_key}-shift-b").as_str(),
                OpenBookmarkManager,
                None,
            ),
            KeyBinding::new(
                format!("{mod_key}-shift-u").as_str(),
                OpenOperationLog,
                None,
            ),
            KeyBinding::new(
                format!("{mod_key}-alt-f").as_str(),
                ShowRepoInFileManager,
                None,
            ),
            KeyBinding::new(format!("{mod_key}-f").as_str(), OpenFind, None),
            KeyBinding::new(format!("{mod_key}-c").as_str(), CopyDiffSelection, None),
            // Scoped to the "NoteComposer" key context, not bare "TextArea", so mod+Return saves the note without binding on every other TextArea (commit box, edit description, ...).
            KeyBinding::new(
                format!("{mod_key}-enter").as_str(),
                SaveNoteComposer,
                Some("NoteComposer"),
            ),
        ];
        key_bindings.extend(text_area::key_bindings(mod_key));
        cx.bind_keys(key_bindings);

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
                true
            });
        });
        cx.activate(true);
    });
}
