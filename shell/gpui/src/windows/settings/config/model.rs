use jayjay_core::{JjConfigSection, JjUserConfig, jj_user_config};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct JjConfigSnapshot {
    pub(super) path: String,
    /// Whether `path` is a file the user can open; a missing config keeps its prospective path but offers no Open action.
    pub(super) exists: bool,
    pub(super) sections: Vec<JjConfigSection>,
    pub(super) error: Option<String>,
}

/// Re-reads the config files on every call — the caller (`ensure_jj_config_loaded`) already caches the result per `SettingsView` instance, so this must stay fresh rather than memoizing for the process lifetime.
pub(super) fn load_jj_config_snapshot() -> JjConfigSnapshot {
    match jj_user_config() {
        Ok(config) => snapshot_from(config),
        Err(error) => JjConfigSnapshot {
            path: String::new(),
            exists: false,
            sections: Vec::new(),
            error: Some(error.to_string()),
        },
    }
}

/// Core reports where the user config would live even before it exists, and the listing then holds only environment-derived values, so an absent config is an empty state rather than a listing.
fn snapshot_from(config: JjUserConfig) -> JjConfigSnapshot {
    if let Some(error) = config.error {
        return JjConfigSnapshot {
            path: config.path,
            exists: config.exists,
            sections: Vec::new(),
            error: Some(error),
        };
    }
    if !config.exists {
        return JjConfigSnapshot {
            path: config.path,
            exists: false,
            sections: Vec::new(),
            error: Some("Config not found".to_owned()),
        };
    }
    JjConfigSnapshot {
        path: config.path,
        exists: true,
        sections: config.sections,
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use jayjay_core::{JjConfigEntry, JjConfigSection, JjUserConfig};

    use super::snapshot_from;

    #[test]
    fn a_load_failure_keeps_the_path_for_repair() {
        let failed = snapshot_from(JjUserConfig {
            path: "/home/dev/.config/jj/config.toml".to_owned(),
            exists: true,
            sections: Vec::new(),
            error: Some("expected `]`".to_owned()),
        });

        assert_eq!(failed.path, "/home/dev/.config/jj/config.toml");
        assert!(failed.exists, "a broken file can still be opened");
        assert_eq!(failed.error.as_deref(), Some("expected `]`"));
    }

    #[test]
    fn missing_user_config_file_is_an_empty_state_not_a_listing() {
        let config = |exists| JjUserConfig {
            path: "/nowhere/config.toml".to_owned(),
            exists,
            sections: vec![JjConfigSection {
                name: "operation".to_owned(),
                entries: vec![JjConfigEntry {
                    key: "hostname".to_owned(),
                    value: "host".to_owned(),
                }],
            }],
            error: None,
        };

        let missing = snapshot_from(config(false));
        assert_eq!(missing.error.as_deref(), Some("Config not found"));
        assert!(!missing.exists, "nothing to open for a missing config");
        assert!(missing.sections.is_empty());

        let found = snapshot_from(config(true));
        assert_eq!(found.error, None);
        assert_eq!(found.sections.len(), 1);
    }
}
