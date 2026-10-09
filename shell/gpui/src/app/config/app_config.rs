use std::path::{Path, PathBuf};

use jayjay_core::{AppDirs, UpdateChannel};
use serde::{Deserialize, Serialize};

use super::{
    AppearanceMode, DiffConfig, FeaturesConfig, LayoutConfig, OnboardingConfig, TelemetryConfig,
    ToolsConfig, WindowState,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct AppConfig {
    pub appearance: AppearanceMode,
    pub interface_font_family: String,
    /// Missing only in configs saved before the split, where one size drove interface text too; `parse` seeds it from the code size.
    #[serde(default)]
    pub(crate) interface_font_size: Option<f32>,
    #[serde(alias = "font_family")]
    pub code_font_family: String,
    #[serde(alias = "font_size")]
    pub(crate) code_font_size: f32,
    pub code_line_spacing: f32,
    pub code_ligatures: bool,
    pub diff: DiffConfig,
    pub layout: LayoutConfig,
    pub(crate) tools: ToolsConfig,
    pub(crate) features: FeaturesConfig,
    pub onboarding: OnboardingConfig,
    pub telemetry: TelemetryConfig,
    pub update_channel: UpdateChannel,
    pub window: WindowState,
    pub recent_repos: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            appearance: AppearanceMode::System,
            interface_font_family: String::new(),
            interface_font_size: Some(AppConfig::DEFAULT_FONT_SIZE),
            code_font_family: String::new(),
            code_font_size: AppConfig::DEFAULT_FONT_SIZE,
            code_line_spacing: AppConfig::MIN_LINE_SPACING,
            code_ligatures: true,
            diff: DiffConfig::default(),
            layout: LayoutConfig::default(),
            tools: ToolsConfig::default(),
            features: FeaturesConfig::default(),
            onboarding: OnboardingConfig::default(),
            telemetry: TelemetryConfig::default(),
            update_channel: UpdateChannel::Stable,
            window: WindowState::default(),
            recent_repos: Vec::new(),
        }
    }
}

impl AppConfig {
    const MAX_RECENT_REPOS: usize = 12;
    pub(crate) const MIN_FONT_SIZE: f32 = 9.;
    pub(crate) const MAX_FONT_SIZE: f32 = 24.;
    pub(crate) const DEFAULT_FONT_SIZE: f32 = 12.;
    pub(crate) const MIN_LINE_SPACING: f32 = 1.;
    pub(crate) const MAX_LINE_SPACING: f32 = 2.;

    pub fn interface_font_size(&self) -> f32 {
        self.interface_font_size.unwrap_or(self.code_font_size)
    }

    pub fn code_font_size(&self) -> f32 {
        self.code_font_size
    }

    pub(crate) fn adjust_interface_font_size(&mut self, delta: f32) {
        self.interface_font_size =
            Some(Self::clamped_font_size(self.interface_font_size() + delta));
    }

    pub(crate) fn adjust_code_font_size(&mut self, delta: f32) {
        self.code_font_size = Self::clamped_font_size(self.code_font_size + delta);
    }

    pub(crate) fn adjust_line_spacing(&mut self, delta: f32) {
        let spacing = ((self.code_line_spacing + delta) * 10.).round() / 10.;
        self.code_line_spacing = spacing.clamp(Self::MIN_LINE_SPACING, Self::MAX_LINE_SPACING);
    }

    pub(crate) fn zoom(&mut self, delta: f32) {
        self.adjust_interface_font_size(delta);
        self.adjust_code_font_size(delta);
    }

    pub(crate) fn reset_zoom(&mut self) {
        self.interface_font_size = Some(Self::DEFAULT_FONT_SIZE);
        self.code_font_size = Self::DEFAULT_FONT_SIZE;
    }

    pub(crate) fn is_default_zoom(&self) -> bool {
        self.interface_font_size() == Self::DEFAULT_FONT_SIZE
            && self.code_font_size == Self::DEFAULT_FONT_SIZE
    }

    pub(crate) fn reset_code_font(&mut self) {
        let defaults = Self::default();
        self.code_font_family = defaults.code_font_family;
        self.code_font_size = defaults.code_font_size;
        self.code_line_spacing = defaults.code_line_spacing;
        self.code_ligatures = defaults.code_ligatures;
    }

    fn clamped_font_size(size: f32) -> f32 {
        size.clamp(Self::MIN_FONT_SIZE, Self::MAX_FONT_SIZE)
    }

    fn config_path() -> Option<PathBuf> {
        AppDirs::new().map(|dirs| dirs.config.join("config.toml"))
    }

    pub fn load() -> Self {
        let Some(path) = Self::config_path() else {
            return Self::default();
        };
        match std::fs::read_to_string(&path) {
            Ok(contents) => Self::parse(&contents),
            Err(_) => Self::default(),
        }
    }

    pub(crate) fn parse(contents: &str) -> Self {
        let mut config: Self = toml::from_str(contents).unwrap_or_default();
        config
            .interface_font_size
            .get_or_insert(config.code_font_size);
        config
    }

    pub(super) fn save(&self) -> std::io::Result<()> {
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

    pub fn record_opened_repo(&mut self, path: &Path) {
        let normalized = normalize_repo_path(path);
        self.recent_repos.retain(|entry| entry != &normalized);
        self.recent_repos.insert(0, normalized);
        self.recent_repos.truncate(Self::MAX_RECENT_REPOS);
    }

    /// Lists the path again without claiming recency, since the first entry doubles as the startup repository.
    pub fn restore_recent_repo(&mut self, path: &Path) {
        let normalized = normalize_repo_path(path);
        if self.recent_repos.contains(&normalized) {
            return;
        }
        self.recent_repos.truncate(Self::MAX_RECENT_REPOS - 1);
        self.recent_repos.push(normalized);
    }

    pub(crate) fn clear_recent_repos(&mut self) {
        self.recent_repos.clear();
    }

    pub(crate) fn remove_recent_repo(&mut self, path: &str) {
        self.recent_repos.retain(|entry| entry != path);
    }
}

fn normalize_repo_path(path: &Path) -> String {
    path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .to_string()
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
        assert_eq!(AppConfig::parse(""), AppConfig::default());
    }

    #[test]
    fn explicit_telemetry_opt_out_is_preserved() {
        let cfg: AppConfig = toml::from_str("[telemetry]\nenabled = false\n").unwrap();
        assert!(!cfg.telemetry.enabled);
    }

    #[test]
    fn legacy_file_column_width_seeds_secondary_pane_width() {
        let cfg: AppConfig = toml::from_str("[layout]\nfile_column_width = 300.0\n").unwrap();
        assert_eq!(cfg.layout.secondary_pane_width, 300.);
    }

    #[test]
    fn unknown_keys_are_ignored() {
        let s = "appearance = \"dark\"\nunknown_root_key = 42\n";
        let cfg: AppConfig = toml::from_str(s).unwrap();
        assert_eq!(cfg.appearance, AppearanceMode::Dark);
    }

    #[test]
    fn restoring_a_recent_repo_keeps_the_startup_entry_first() {
        let mut cfg = AppConfig::default();
        for ix in 0..AppConfig::MAX_RECENT_REPOS {
            cfg.record_opened_repo(Path::new(&format!("/tmp/repo-{ix}")));
        }
        cfg.restore_recent_repo(Path::new("/tmp/repo-3"));
        assert_eq!(
            cfg.recent_repos.iter().position(|p| p == "/tmp/repo-3"),
            Some(8)
        );

        cfg.restore_recent_repo(Path::new("/tmp/unpinned"));
        assert_eq!(
            cfg.recent_repos.first().map(String::as_str),
            Some("/tmp/repo-11")
        );
        assert_eq!(
            cfg.recent_repos.last().map(String::as_str),
            Some("/tmp/unpinned")
        );
        assert_eq!(cfg.recent_repos.len(), AppConfig::MAX_RECENT_REPOS);
    }

    #[test]
    fn recent_repos_are_deduped_and_capped() {
        let mut cfg = AppConfig::default();
        for ix in 0..14 {
            cfg.record_opened_repo(Path::new(&format!("/tmp/repo-{ix}")));
        }
        cfg.record_opened_repo(Path::new("/tmp/repo-4"));

        assert_eq!(
            cfg.recent_repos.first().map(String::as_str),
            Some("/tmp/repo-4")
        );
        assert_eq!(cfg.recent_repos.len(), AppConfig::MAX_RECENT_REPOS);
        assert_eq!(
            cfg.recent_repos
                .iter()
                .filter(|path| path.as_str() == "/tmp/repo-4")
                .count(),
            1
        );
    }

    #[test]
    fn pre_split_config_keeps_the_code_font_and_seeds_the_interface_size() {
        let config = AppConfig::parse("font_family = \"menlo\"\nfont_size = 15.0\n");
        assert_eq!(config.code_font_family, "menlo");
        assert_eq!(config.code_font_size(), 15.);
        assert_eq!(config.interface_font_family, "");
        assert_eq!(config.interface_font_size(), 15.);
        assert!(
            toml::to_string(&config)
                .unwrap()
                .contains("code_font_family = \"menlo\"")
        );
    }

    #[test]
    fn zoom_moves_both_sizes_within_bounds_and_reset_code_font_keeps_the_interface() {
        let mut config = AppConfig {
            interface_font_size: Some(AppConfig::MAX_FONT_SIZE - 1.),
            ..Default::default()
        };
        config.zoom(1.);
        config.zoom(1.);
        assert_eq!(
            (config.interface_font_size(), config.code_font_size()),
            (AppConfig::MAX_FONT_SIZE, 14.)
        );
        config.adjust_line_spacing(0.1);
        config.reset_code_font();
        assert_eq!(
            (config.interface_font_size(), config.code_font_size()),
            (AppConfig::MAX_FONT_SIZE, 12.)
        );
        assert_eq!(config.code_line_spacing, 1.);
        config.reset_zoom();
        assert!(config.is_default_zoom());
    }
}
