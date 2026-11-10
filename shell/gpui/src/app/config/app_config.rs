use jayjay_core::shortcuts::{KeyPlatform, ShortcutOverrides};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use jayjay_core::flock::FlockIcon;
use jayjay_core::{AppDirs, UpdateChannel};
use serde::{Deserialize, Serialize};

use super::{
    AppearanceMode, DiffConfig, FeaturesConfig, LayoutConfig, OnboardingConfig, TelemetryConfig,
    ToolsConfig, WindowState,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct AppConfig {
    pub shortcuts: BTreeMap<String, Vec<String>>,
    pub appearance: AppearanceMode,
    pub flock_icon: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) language: Option<String>,
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
    pub tools: ToolsConfig,
    pub(crate) features: FeaturesConfig,
    pub onboarding: OnboardingConfig,
    pub telemetry: TelemetryConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) debug_mode: Option<bool>,
    pub update_channel: UpdateChannel,
    pub window: WindowState,
    pub recent_repos: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            shortcuts: BTreeMap::new(),
            appearance: AppearanceMode::System,
            flock_icon: FlockIcon::Classic.id().into(),
            language: None,
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
            debug_mode: None,
            update_channel: UpdateChannel::Stable,
            window: WindowState::default(),
            recent_repos: Vec::new(),
        }
    }
}

impl AppConfig {
    pub fn shortcut_overrides(&self) -> ShortcutOverrides {
        ShortcutOverrides::from_stored(self.shortcuts.clone(), KeyPlatform::current())
    }

    const MAX_RECENT_REPOS: usize = 12;

    /// Beta builds default to debug mode until the user picks a side.
    pub fn debug_mode(&self) -> bool {
        self.debug_mode
            .unwrap_or_else(jayjay_core::is_prerelease_build)
    }
    pub(crate) const MIN_FONT_SIZE: f32 = 9.;
    pub(crate) const MAX_FONT_SIZE: f32 = 24.;
    pub(crate) const DEFAULT_FONT_SIZE: f32 = 12.;
    pub(crate) const MIN_LINE_SPACING: f32 = 1.;
    pub(crate) const MAX_LINE_SPACING: f32 = 2.;

    pub(crate) fn preferred_languages(&self) -> Vec<String> {
        match &self.language {
            Some(tag) => vec![tag.clone()],
            None => sys_locale::get_locales().collect(),
        }
    }

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

    pub(crate) fn reset_fonts(&mut self) {
        let defaults = Self::default();
        self.interface_font_family = defaults.interface_font_family;
        self.interface_font_size = defaults.interface_font_size;
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

    /// Errs when the file cannot be read or backed up, so the caller can avoid saving over values it never saw.
    pub fn load() -> std::io::Result<Self> {
        let Some(path) = Self::config_path() else {
            return Ok(Self::default());
        };
        let with_path = |error: std::io::Error| {
            std::io::Error::new(error.kind(), format!("{}: {error}", path.display()))
        };
        let contents = match std::fs::read_to_string(&path) {
            Ok(contents) => contents,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Self::default());
            }
            Err(error) => return Err(with_path(error)),
        };
        let (config, complete) = Self::parse(&contents).unwrap_or_else(|error| {
            log::warn!("config.toml: {} is not valid TOML: {error}", path.display());
            (Self::default(), false)
        });
        if !complete {
            let backup = path.with_extension("toml.bak");
            std::fs::copy(&path, &backup).map_err(with_path)?;
            log::warn!(
                "config.toml: {} has values that could not be read; the original is kept at {}",
                path.display(),
                backup.display()
            );
        }
        Ok(config)
    }

    /// The flag is false when a value was dropped, so the caller knows a save would lose it.
    pub(crate) fn parse(contents: &str) -> Result<(Self, bool), toml::de::Error> {
        let table: toml::Table = toml::from_str(contents)?;
        let (mut config, complete) = match toml::Value::Table(table.clone()).try_into() {
            Ok(config) => (config, true),
            Err(_) => (Self::from_valid_entries(table), false),
        };
        config
            .interface_font_size
            .get_or_insert(config.code_font_size);
        Ok((config, complete))
    }

    /// Drops only the values that fail to deserialize, so one bad entry does not reset every other setting.
    fn from_valid_entries(table: toml::Table) -> Self {
        let mut kept = toml::Table::new();
        keep_valid_entries(&mut kept, &[], table);
        toml::Value::Table(kept).try_into().unwrap_or_default()
    }

    pub(super) fn save(&self) -> std::io::Result<()> {
        let Some(path) = Self::config_path() else {
            return Ok(());
        };
        let contents = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::other(format!("toml serialize: {e}")))?;
        jayjay_core::write_atomically(&path, contents.as_bytes())
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

fn keep_valid_entries(root: &mut toml::Table, path: &[String], entries: toml::Table) {
    let is_valid = |root: &toml::Table| toml::Value::Table(root.clone()).try_into::<AppConfig>();
    for (key, value) in entries {
        let nested = value.as_table().cloned();
        table_at(root, path).insert(key.clone(), value);
        let Err(error) = is_valid(root) else {
            continue;
        };
        let mut entry_path = path.to_vec();
        entry_path.push(key.clone());
        if let Some(entries) = nested {
            table_at(root, path).insert(key.clone(), toml::Table::new().into());
            if is_valid(root).is_ok() {
                keep_valid_entries(root, &entry_path, entries);
                continue;
            }
        }
        table_at(root, path).remove(&key);
        log::warn!("config.toml: ignoring `{}`: {error}", entry_path.join("."));
    }
}

fn table_at<'a>(root: &'a mut toml::Table, path: &[String]) -> &'a mut toml::Table {
    path.iter().fold(root, |table, key| {
        table
            .get_mut(key)
            .and_then(toml::Value::as_table_mut)
            .expect("parent table kept before its entries")
    })
}

fn normalize_repo_path(path: &Path) -> String {
    match jayjay_core::ssh::RemoteLocation::from_path(path) {
        Some(location) => location.encode(),
        None => dunce::canonicalize(path)
            .unwrap_or_else(|_| path.to_path_buf())
            .to_string_lossy()
            .into_owned(),
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
        assert_eq!(AppConfig::parse("").unwrap(), (AppConfig::default(), true));
    }

    #[test]
    fn a_file_that_is_not_toml_is_an_error() {
        assert!(AppConfig::parse("appearance = [").is_err());
    }

    #[test]
    fn one_invalid_value_keeps_the_other_settings() {
        let (cfg, complete) = AppConfig::parse(
            "appearance = \"Dark\"\nrecent_repos = [\"/r\"]\n[diff]\ntree_file_list = 3\nside_by_side = true\n[telemetry]\nenabled = false\n",
        )
        .unwrap();
        assert!(!complete);
        assert_eq!(cfg.appearance, AppearanceMode::System);
        assert_eq!(cfg.recent_repos, ["/r"]);
        assert!(cfg.diff.side_by_side);
        assert!(!cfg.diff.tree_file_list);
        assert!(!cfg.telemetry.enabled);
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
        let (config, _) = AppConfig::parse("font_family = \"menlo\"\nfont_size = 15.0\n").unwrap();
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
    fn zoom_moves_both_sizes_within_bounds_and_reset_fonts_restores_both() {
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
        config.interface_font_family = "Courier New".to_owned();
        config.reset_fonts();
        assert_eq!(
            (config.interface_font_size(), config.code_font_size()),
            (12., 12.)
        );
        assert_eq!(config.interface_font_family, "");
        assert_eq!(config.code_line_spacing, 1.);
        config.reset_zoom();
        assert!(config.is_default_zoom());
    }
}
