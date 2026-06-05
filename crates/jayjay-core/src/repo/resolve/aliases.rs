use std::collections::HashSet;
use std::sync::OnceLock;

use jj_lib::config::ConfigGetResultExt;
use jj_lib::fileset::FilesetAliasesMap;
use jj_lib::revset::RevsetAliasesMap;
use jj_lib::settings::UserSettings;

use super::super::Repo;
use crate::types::*;

const REVSET_ALIASES: &str = "revset-aliases";
const FILESET_ALIASES: &str = "fileset-aliases";
const DEFAULT_REVSET_ALIAS_RECORD_MARKER: &str = "--jayjay-default-revset-alias--";
const DEFAULT_REVSET_ALIAS_TEMPLATE: &str = r#"if(
  source == "default",
  name ++ "\t" ++ value ++ "\n--jayjay-default-revset-alias--\n",
  ""
)"#;

struct AliasConfig {
    definition: String,
    doc: Option<String>,
}

impl Repo {
    pub(crate) fn revset_aliases_map(
        &self,
        settings: &UserSettings,
    ) -> CoreResult<RevsetAliasesMap> {
        let mut aliases_map = RevsetAliasesMap::new();
        let mut loaded = HashSet::new();
        for name in settings.table_keys(REVSET_ALIASES) {
            let alias = Self::load_alias_config(settings, REVSET_ALIASES, name)?;
            Self::insert_revset_alias(&mut aliases_map, name, alias.definition, alias.doc)?;
            loaded.insert(name.to_owned());
        }

        for (name, definition) in self.cli_revset_aliases() {
            if loaded.contains(name.as_str()) {
                continue;
            }
            Self::insert_revset_alias(&mut aliases_map, name, definition.clone(), None)?;
            loaded.insert(name.to_owned());
        }
        Ok(aliases_map)
    }

    fn cli_revset_aliases(&self) -> &'static [(String, String)] {
        static DEFAULT_ALIASES: OnceLock<Vec<(String, String)>> = OnceLock::new();
        if let Some(aliases) = DEFAULT_ALIASES.get() {
            return aliases.as_slice();
        }

        let Some(aliases) = self.load_cli_revset_aliases() else {
            return &[];
        };
        if !aliases.is_empty() {
            let _ = DEFAULT_ALIASES.set(aliases);
        }
        DEFAULT_ALIASES.get().map(Vec::as_slice).unwrap_or(&[])
    }

    fn load_cli_revset_aliases(&self) -> Option<Vec<(String, String)>> {
        let output = self
            .run_jj(&[
                "--ignore-working-copy",
                "config",
                "list",
                "--include-defaults",
                "--include-overridden",
                REVSET_ALIASES,
                "-T",
                DEFAULT_REVSET_ALIAS_TEMPLATE,
            ])
            .ok()?;
        Some(Self::parse_cli_revset_aliases(&output))
    }

    pub(crate) fn fileset_aliases_map(
        &self,
        settings: &UserSettings,
    ) -> CoreResult<FilesetAliasesMap> {
        let mut aliases_map = FilesetAliasesMap::new();
        for name in settings.table_keys(FILESET_ALIASES) {
            let alias = Self::load_alias_config(settings, FILESET_ALIASES, name)?;
            Self::insert_fileset_alias(&mut aliases_map, name, alias.definition, alias.doc)?;
        }
        Ok(aliases_map)
    }

    fn load_alias_config(
        settings: &UserSettings,
        table: &str,
        name: &str,
    ) -> CoreResult<AliasConfig> {
        match settings.get_string([table, name]) {
            Ok(definition) => Ok(AliasConfig {
                definition,
                doc: None,
            }),
            Err(value_error) => {
                let definition = settings.get_string([table, name, "definition"]).map_err(|e| {
                    Error::internal(format_args!(
                        "load {table} {name}: {value_error}; load {table} {name}.definition: {e}"
                    ))
                })?;
                let doc = settings
                    .get_string([table, name, "doc"])
                    .optional()
                    .map_err(|e| Error::internal(format_args!("load {table} {name}.doc: {e}")))?;
                Ok(AliasConfig { definition, doc })
            }
        }
    }

    fn insert_revset_alias(
        aliases_map: &mut RevsetAliasesMap,
        name: &str,
        definition: String,
        doc: Option<String>,
    ) -> CoreResult<()> {
        aliases_map
            .insert(name, definition, doc)
            .map_err(|e| Error::internal(format_args!("parse revset alias {name}: {e}")))
    }

    fn insert_fileset_alias(
        aliases_map: &mut FilesetAliasesMap,
        name: &str,
        definition: String,
        doc: Option<String>,
    ) -> CoreResult<()> {
        aliases_map
            .insert(name, definition, doc)
            .map_err(|e| Error::internal(format_args!("parse fileset alias {name}: {e}")))
    }

    fn parse_cli_revset_aliases(output: &str) -> Vec<(String, String)> {
        output
            .split(DEFAULT_REVSET_ALIAS_RECORD_MARKER)
            .filter_map(|record| {
                let record = record.trim();
                if record.is_empty() {
                    return None;
                }
                let (config_name, value) = record.split_once('\t')?;
                let name = Self::parse_revset_alias_config_name(config_name.trim())?;
                let definition = Self::parse_toml_string(value.trim())?;
                Some((name, definition))
            })
            .collect()
    }

    fn parse_revset_alias_config_name(config_name: &str) -> Option<String> {
        let name = config_name
            .strip_prefix(REVSET_ALIASES)?
            .strip_prefix('.')?;
        if name.starts_with('"') || name.starts_with('\'') {
            Self::parse_toml_string(name)
        } else {
            Some(name.to_owned())
        }
    }

    fn parse_toml_string(raw: &str) -> Option<String> {
        toml::from_str(raw).ok()
    }
}

#[cfg(test)]
mod tests {
    use jj_lib::config::{ConfigLayer, ConfigSource, StackedConfig};

    use super::*;

    fn settings_with_config(text: &str) -> UserSettings {
        let mut config = StackedConfig::with_defaults();
        config.add_layer(ConfigLayer::parse(ConfigSource::User, text).expect("parse config"));
        UserSettings::from_config(config).expect("build user settings")
    }

    #[test]
    fn load_alias_config_reads_table_definition_and_doc() {
        let settings = settings_with_config(
            r#"
user.name = "Test User"
user.email = "test@example.com"

[revset-aliases."current_with_doc()"]
definition = "@"
doc = "Current working-copy change"
"#,
        );

        let alias =
            Repo::load_alias_config(&settings, REVSET_ALIASES, "current_with_doc()").unwrap();

        assert_eq!(alias.definition, "@");
        assert_eq!(alias.doc.as_deref(), Some("Current working-copy change"));
    }
}
