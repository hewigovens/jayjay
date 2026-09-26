use jj_lib::config::{ConfigItem, ConfigNamePathBuf, ConfigSource, StackedConfig};

use super::ConfigEnv;
use crate::types::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JjConfigEntry {
    /// The name inside its section, such as `email` for `user.email`.
    pub key: String,
    /// The value as TOML text, so strings keep their quotes as `jj config list` prints them.
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JjConfigSection {
    /// The first segment of each entry's name; top-level values fall under `general`.
    pub name: String,
    pub entries: Vec<JjConfigEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JjUserConfig {
    /// The first user config file or directory that exists, else where jj would create one.
    pub path: String,
    pub exists: bool,
    pub sections: Vec<JjConfigSection>,
    /// Why the files could not be loaded, with `path` still set so the tab can offer to open the file that needs fixing.
    pub error: Option<String>,
}

/// What `jj config list` reports outside a repository, from the same layers core loads, with jj's own defaults left out.
pub fn jj_user_config() -> CoreResult<JjUserConfig> {
    user_config_from(&ConfigEnv::from_environment())
}

fn user_config_from(env: &ConfigEnv) -> CoreResult<JjUserConfig> {
    let mut path = env
        .user_config_path()
        .ok_or_else(|| CoreError::internal("no home directory for the jj user config"))?;
    let (sections, error) = match env.user_level_config() {
        Ok(config) => (effective_sections(config), None),
        Err(failure) => {
            if let Some(failing) = failure.path {
                path = failing;
            }
            (Vec::new(), Some(failure.message))
        }
    };
    Ok(JjUserConfig {
        exists: path.exists(),
        path: path.display().to_string(),
        sections,
        error,
    })
}

/// The values jj would use, grouped by section in name order; merging the layers first drops a value that a higher layer's table or scalar shadows.
fn effective_sections(mut config: StackedConfig) -> Vec<JjConfigSection> {
    config.remove_layers(ConfigSource::Default);
    let Ok(merged) = config.get_table(ConfigNamePathBuf::root()) else {
        return Vec::new();
    };
    let mut values = Vec::new();
    let mut pending: Vec<(ConfigNamePathBuf, &ConfigItem)> = merged
        .iter()
        .map(|(key, item)| (ConfigNamePathBuf::from_iter([key]), item))
        .collect();
    while let Some((name, item)) = pending.pop() {
        if let Some(table) = item.as_table_like() {
            for (key, child) in table.iter() {
                let mut child_name = name.clone();
                child_name.push(key);
                pending.push((child_name, child));
            }
        } else if let Ok(value) = item.clone().into_value() {
            // Arrays of tables are neither table-like nor plain values; they list as inline tables, as `jj config list` prints them.
            values.push((name, value.to_string().trim().to_owned()));
        }
    }
    values.sort();

    let mut sections: Vec<JjConfigSection> = Vec::new();
    for (name, value) in values {
        let head: ConfigNamePathBuf = name.components().take(1).cloned().collect();
        let rest: ConfigNamePathBuf = name.components().skip(1).cloned().collect();
        let (section, key) = if rest.is_root() {
            ("general".to_owned(), head.to_string())
        } else {
            (head.to_string(), rest.to_string())
        };
        let entry = JjConfigEntry { key, value };
        match sections
            .iter_mut()
            .find(|existing| existing.name == section)
        {
            Some(existing) => existing.entries.push(entry),
            None => sections.push(JjConfigSection {
                name: section,
                entries: vec![entry],
            }),
        }
    }
    sections
}

#[cfg(test)]
mod tests {
    use jj_lib::config::{ConfigLayer, ConfigSource, StackedConfig};

    use std::collections::HashMap;

    use super::{ConfigEnv, JjConfigEntry, JjConfigSection, effective_sections, user_config_from};

    #[test]
    fn a_malformed_user_config_keeps_its_path_and_reports_the_error() {
        let dir = tempfile::tempdir().expect("tempdir");
        let home = dir.path().join("home");
        let config_dir = dir.path().join("config");
        std::fs::create_dir_all(config_dir.join("jj")).expect("create config dir");
        std::fs::create_dir_all(&home).expect("create home");
        let file = config_dir.join("jj").join("config.toml");
        std::fs::write(&file, "[user\nname = 'broken'\n").expect("write malformed config");
        let env = ConfigEnv::new(
            Some(home),
            Some(config_dir),
            None,
            "test-host".to_owned(),
            HashMap::new(),
        );

        let config = user_config_from(&env).expect("path resolves");

        assert_eq!(config.path, file.display().to_string());
        assert!(config.exists);
        assert!(config.sections.is_empty());
        assert!(config.error.is_some(), "the parse failure is reported");

        // A later JJ_CONFIG entry that fails is the file to open, not the first one.
        let good = dir.path().join("good.toml");
        std::fs::write(&good, "[user]\nname = 'fine'\n").expect("write good config");
        let env = ConfigEnv::new(
            Some(dir.path().join("home")),
            Some(dir.path().join("config")),
            None,
            "test-host".to_owned(),
            HashMap::from([(
                "JJ_CONFIG".to_owned(),
                std::env::join_paths([&good, &file])
                    .expect("join paths")
                    .to_string_lossy()
                    .into_owned(),
            )]),
        );
        let config = user_config_from(&env).expect("path resolves");
        assert_eq!(config.path, file.display().to_string());
        assert!(config.error.is_some());
    }

    fn sections(layers: &[(ConfigSource, &str)]) -> Vec<JjConfigSection> {
        let mut config = StackedConfig::with_defaults();
        for (source, text) in layers {
            config.add_layer(ConfigLayer::parse(*source, text).expect("parse layer"));
        }
        effective_sections(config)
    }

    fn section(name: &str, entries: &[(&str, &str)]) -> JjConfigSection {
        JjConfigSection {
            name: name.to_owned(),
            entries: entries
                .iter()
                .map(|(key, value)| JjConfigEntry {
                    key: (*key).to_owned(),
                    value: (*value).to_owned(),
                })
                .collect(),
        }
    }

    #[test]
    fn sections_keep_arrays_of_tables_as_inline_tables() {
        let listing = sections(&[(ConfigSource::User, "[[hooks]]\ncmd = 'lint'\n")]);

        assert_eq!(
            listing,
            [section("general", &[("hooks", "[{ cmd = 'lint' }]")])]
        );
    }

    #[test]
    fn sections_hold_the_winning_values_in_name_order_without_defaults() {
        let listing = sections(&[
            (
                ConfigSource::System,
                "top = 1\n[user]\nname = 'System'\n[ui]\neditor = 'vi'\n",
            ),
            (
                ConfigSource::User,
                "zzz = 2\n[user]\nname = 'Alice'\nemail = 'a@example.com'\n",
            ),
        ]);

        assert_eq!(
            listing,
            [
                section("general", &[("top", "1"), ("zzz", "2")]),
                section("ui", &[("editor", "'vi'")]),
                section("user", &[("email", "'a@example.com'"), ("name", "'Alice'")]),
            ]
        );
    }

    #[test]
    fn a_higher_layer_value_or_table_shadows_the_lower_one() {
        let scalar_over_table = sections(&[
            (ConfigSource::System, "[foo]\nbar = 2\n"),
            (ConfigSource::User, "foo = 1\n"),
        ]);
        assert_eq!(scalar_over_table, [section("general", &[("foo", "1")])]);

        let table_over_scalar = sections(&[
            (ConfigSource::System, "foo = 1\n"),
            (ConfigSource::User, "[foo]\nbar = 2\n"),
        ]);
        assert_eq!(table_over_scalar, [section("foo", &[("bar", "2")])]);
    }
}
