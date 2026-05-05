use std::borrow::Cow;
use std::path::PathBuf;

use gpui::{
    App, AppContext, Bounds, Focusable, KeyBinding, Point, Size, TitlebarOptions, WindowBounds,
    WindowOptions, px, size,
};

use jayjay_gpui::app::actions::{
    CloseWindow, CopyDiffSelection, OpenCommandPalette, OpenFind, OpenSettings, Refresh,
};
use jayjay_gpui::app::config::{AppConfig, AppConfigStore};
use jayjay_gpui::app::theme::Theme;
use jayjay_gpui::log::LogView;

const PHOSPHOR_FONT: &[u8] = include_bytes!("../assets/fonts/Phosphor.ttf");

fn resolve_repo_path() -> PathBuf {
    let raw = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."));
    raw.canonicalize().unwrap_or(raw)
}

fn main() {
    let path = resolve_repo_path();
    let title: String = match path.file_name().and_then(|s| s.to_str()) {
        Some(name) if !name.is_empty() => format!("JayJay (Alpha) — {name}"),
        _ => "JayJay (Alpha)".to_string(),
    };

    gpui_platform::application().run(move |cx: &mut App| {
        match cx
            .text_system()
            .add_fonts(vec![Cow::Borrowed(PHOSPHOR_FONT)])
        {
            Ok(()) => eprintln!(
                "[jayjay-gpui] registered Phosphor font ({} bytes)",
                PHOSPHOR_FONT.len()
            ),
            Err(e) => eprintln!("[jayjay-gpui] failed to register Phosphor: {e}"),
        }

        let cfg = AppConfig::load();
        cx.set_global(Theme::for_appearance(cfg.appearance));

        let mod_key = if cfg!(target_os = "macos") {
            "cmd"
        } else {
            "ctrl"
        };
        cx.bind_keys([
            KeyBinding::new(format!("{mod_key}-,").as_str(), OpenSettings, None),
            KeyBinding::new(format!("{mod_key}-w").as_str(), CloseWindow, None),
            KeyBinding::new("escape", CloseWindow, None),
            KeyBinding::new(format!("{mod_key}-r").as_str(), Refresh, None),
            KeyBinding::new(
                format!("{mod_key}-shift-p").as_str(),
                OpenCommandPalette,
                None,
            ),
            KeyBinding::new(format!("{mod_key}-f").as_str(), OpenFind, None),
            KeyBinding::new(format!("{mod_key}-c").as_str(), CopyDiffSelection, None),
        ]);

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
        let window_handle = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(initial_window_bounds),
                    titlebar: Some(TitlebarOptions {
                        title: Some(title.clone().into()),
                        appears_transparent: true,
                        traffic_light_position: Some(Point {
                            x: px(12.),
                            y: px(12.),
                        }),
                    }),
                    ..Default::default()
                },
                |_, cx| {
                    cx.new(|cx| {
                        cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
                        cx.observe_global::<AppConfigStore>(|_, cx| cx.notify())
                            .detach();
                        let mut view = LogView::new(path.clone(), cx);
                        view.boot(cx);
                        view
                    })
                },
            )
            .unwrap();
        let _ = window_handle.update(cx, |view, window, cx| {
            let handle = view.focus_handle(cx);
            window.focus(&handle, cx);

            // Persist window bounds when the user closes the window.
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
