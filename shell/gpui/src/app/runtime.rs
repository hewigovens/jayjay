use std::borrow::Cow;

use gpui::{App, AppContext};

use super::config::{self, AppConfig, AppConfigStore};
use super::gpui_assets::GpuiAssets;
use super::startup_window::{open_startup_window, resolve_repo_path};
use super::theme::Theme;
use crate::startup::GuiLaunch;

const LUCIDE_FONT: &[u8] = include_bytes!("../../assets/fonts/Lucide.ttf");

pub(crate) fn run(launch: GuiLaunch) {
    super::cli_install::repair_broken_link();
    let cfg = AppConfig::load();
    let app = gpui_platform::application().with_assets(GpuiAssets);
    app.run(move |cx: &mut App| {
        if let Err(error) = cx.text_system().add_fonts(vec![Cow::Borrowed(LUCIDE_FONT)]) {
            eprintln!("[jayjay-gpui] failed to register Lucide: {error}");
        }
        cx.set_global(
            Theme::for_appearance(cfg.appearance, cx.window_appearance())
                .with_font_size(cfg.font_size()),
        );
        cx.bind_keys(super::actions::app_key_bindings());

        cx.set_global(AppConfigStore::new(cfg));
        match launch {
            GuiLaunch::ExternalTool(invocation) => {
                if let Err(error) = crate::external_tool::open_external_tool(invocation, cx) {
                    eprintln!("[jayjay-gpui] failed to open external tool: {error}");
                    std::process::exit(1);
                }
                cx.activate(true);
            }
            GuiLaunch::Repository { path, .. } => {
                let cfg = config::current(cx);
                super::telemetry::maybe_ping(cfg.telemetry.enabled);
                super::repositories::install(cx);
                super::menus::install(cx);
                cx.spawn(async move |cx| {
                    let path = cx
                        .background_spawn(async move {
                            resolve_repo_path(path, std::env::current_dir().ok(), &cfg.recent_repos)
                        })
                        .await;
                    cx.update(|cx| open_startup_window(path, cx));
                })
                .detach();
            }
        }
    });
}
