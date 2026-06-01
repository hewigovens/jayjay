//! TOML config at `$HOME/.config/jayjay/config.toml`. Owned by the GPUI shell;
//! the SwiftUI shell uses its own UserDefaults-backed `AppSettings` and does
//! not consume this schema.

pub mod appearance;
pub mod diff;
pub mod features;
pub mod layout;
pub mod store;
pub mod tools;
pub mod window;

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub use appearance::AppearanceMode;
pub use diff::DiffConfig;
pub use features::FeaturesConfig;
pub use layout::LayoutConfig;
pub use store::{AppConfigStore, current, update};
pub use tools::ToolsConfig;
pub use window::WindowState;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct AppConfig {
    pub appearance: AppearanceMode,
    pub font_family: String,
    pub font_size: f32,
    pub diff: DiffConfig,
    pub layout: LayoutConfig,
    pub tools: ToolsConfig,
    pub features: FeaturesConfig,
    pub window: WindowState,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            appearance: AppearanceMode::System,
            font_family: String::new(),
            font_size: 12.0,
            diff: DiffConfig::default(),
            layout: LayoutConfig::default(),
            tools: ToolsConfig::default(),
            features: FeaturesConfig::default(),
            window: WindowState::default(),
        }
    }
}

impl AppConfig {
    /// Resolve the config file path via `ProjectDirs` so each platform gets
    /// its native location:
    /// - macOS:   `~/Library/Application Support/dev.hewig.jayjay/config.toml`
    /// - Linux:   `~/.config/jayjay/config.toml`
    /// - Windows: `%APPDATA%\hewig\jayjay\config\config.toml`
    pub fn config_path() -> Option<PathBuf> {
        directories::ProjectDirs::from("dev", "hewig", "jayjay")
            .map(|d| d.config_dir().join("config.toml"))
    }

    /// Read from disk; falls back to defaults on missing/malformed files.
    pub fn load() -> Self {
        let Some(path) = Self::config_path() else {
            return Self::default();
        };
        match std::fs::read_to_string(&path) {
            Ok(contents) => toml::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Write to disk, creating parent directories as needed.
    pub fn save(&self) -> std::io::Result<()> {
        let Some(path) = Self::config_path() else {
            return Ok(());
        };
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let contents = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::other(format!("toml serialize: {e}")))?;
        std::fs::write(path, contents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_defaults() {
        let cfg = AppConfig::default();
        let s = toml::to_string_pretty(&cfg).unwrap();
        let back: AppConfig = toml::from_str(&s).unwrap();
        assert_eq!(cfg, back);
    }

    #[test]
    fn empty_config_file_uses_defaults() {
        let cfg: AppConfig = toml::from_str("").unwrap();
        assert_eq!(cfg, AppConfig::default());
    }

    #[test]
    fn unknown_keys_are_ignored() {
        let s = "appearance = \"dark\"\nunknown_root_key = 42\n";
        let cfg: AppConfig = toml::from_str(s).unwrap();
        assert_eq!(cfg.appearance, AppearanceMode::Dark);
    }
}
