pub mod actions;
pub mod cli_install;
pub mod config;
pub mod feedback;
pub mod fonts;
pub mod fs_watcher;
pub mod links;
pub mod menus;
pub mod repositories;
pub mod telemetry;
pub mod theme;
pub mod tools;

#[cfg(target_os = "linux")]
pub(crate) const APP_ID: &str = "dev.hewig.JayJay";
