use std::borrow::Cow;
use std::time::Instant;

use gpui::{App, AppContext};

use super::config::{self, AppConfig, AppConfigStore};
use super::gpui_assets::GpuiAssets;
use super::stall_monitor::StallMonitor;
use super::startup_window::{open_startup_window, resolve_repo_path};
use super::theme::Theme;
use crate::startup::GuiLaunch;

const OCTICONS_FONT: &[u8] = include_bytes!("../../assets/fonts/Octicons.ttf");
const SETI_FONT: &[u8] = include_bytes!("../../../../assets/fonts/Seti.ttf");

pub(crate) fn run(launch: GuiLaunch) {
    let launched_at = Instant::now();
    jayjay_core::LogFile::install("gpui", env!("CARGO_PKG_VERSION"));
    let cfg = AppConfig::load();
    jayjay_i18n::set_preferred_languages(&cfg.preferred_languages());
    jayjay_i18n::set_number_locale(&crate::platform::numeric_locale());
    super::cli_install::repair_broken_link();
    let app = gpui_platform::application().with_assets(GpuiAssets);
    app.run(move |cx: &mut App| {
        log::info!("platform ready at {}ms", launched_at.elapsed().as_millis());
        let mut stall_monitor = StallMonitor::new(launched_at);
        apply_debug_mode(&mut stall_monitor, cfg.debug_mode(), cx);
        let fonts = vec![Cow::Borrowed(OCTICONS_FONT), Cow::Borrowed(SETI_FONT)];
        if let Err(error) = cx.text_system().add_fonts(fonts) {
            log::error!("failed to register the icon fonts: {error}");
        }
        cx.set_global(
            Theme::for_appearance(cfg.appearance, cx.window_appearance(), &cfg.diff.palette)
                .with_typography(&cfg),
        );
        cx.bind_keys(super::actions::app_key_bindings());

        let fonts_started = Instant::now();
        cx.set_global(AppConfigStore::new(cfg));
        log::info!(
            "fonts resolved in {}ms",
            fonts_started.elapsed().as_millis()
        );
        cx.observe_global::<AppConfigStore>(move |cx| {
            apply_debug_mode(&mut stall_monitor, config::current(cx).debug_mode(), cx)
        })
        .detach();
        cx.set_global(super::rating_prompt::RatingPromptStore::load());
        match launch {
            GuiLaunch::ExternalTool(invocation) => {
                if let Err(error) = crate::external_tool::open_external_tool(invocation, cx) {
                    log::error!("failed to open external tool: {error}");
                    std::process::exit(1);
                }
                cx.activate(true);
            }
            GuiLaunch::Repository { path, .. } => {
                let cfg = config::current(cx);
                if cfg.onboarding.completed {
                    super::telemetry::maybe_ping(cfg.telemetry.enabled);
                }
                super::repositories::install(cx);
                super::menus::install(cx);
                cx.spawn(async move |cx| {
                    let path = cx
                        .background_spawn(async move {
                            resolve_repo_path(path, std::env::current_dir().ok(), &cfg.recent_repos)
                        })
                        .await;
                    cx.update(|cx| open_startup_window(path, cx));
                    log::info!(
                        "first window opened at {}ms",
                        launched_at.elapsed().as_millis()
                    );
                })
                .detach();
            }
        }
    });
}

fn apply_debug_mode(stall_monitor: &mut StallMonitor, enabled: bool, cx: &mut App) {
    jayjay_core::LogFile::set_debug(enabled);
    stall_monitor.set_enabled(enabled, cx);
}
