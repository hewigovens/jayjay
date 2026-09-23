pub mod actions;
pub mod cli_install;
pub mod config;
mod error_text;
pub mod feedback;
pub mod fonts;
pub mod fs_watcher;
mod gpui_assets;
pub mod links;
pub mod menus;
pub mod motion;
pub mod repositories;
pub(crate) mod runtime;
mod startup_window;
pub mod telemetry;
pub mod theme;
pub mod tools;

pub(crate) use error_text::error_text;

pub(crate) const APP_ID: &str = "dev.hewig.JayJay";

pub fn window_options() -> gpui::WindowOptions {
    gpui::WindowOptions {
        app_id: Some(APP_ID.to_owned()),
        ..Default::default()
    }
}
