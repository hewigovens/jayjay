use std::collections::HashSet;
use std::sync::OnceLock;

use jj_lib::config::{ConfigGetResultExt, ConfigValue};
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

    /// jj's own default aliases, so user aliases can build on names like `builtin_immutable_heads()`; loaded once per process.
    fn cli_revset_aliases(&self) -> &'static [(String, String)] {
        static DEFAULT_ALIASES: OnceLock<Vec<(String, String)>> = OnceLock::new();
        DEFAULT_ALIASES.get_or_init(|| {
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
                .unwrap_or_default();
            self.supported_aliases(Self::parse_cli_revset_aliases(&output))
        })
    }

    /// A CLI newer than the embedded jj-lib can define aliases this build cannot parse; drop those and whatever depends on them so the functions in `expressions.rs` stand in.
    fn supported_aliases(&self, mut aliases: Vec<(String, String)>) -> Vec<(String, String)> {
        let fileset_aliases_map = FilesetAliasesMap::new();
        loop {
            let mut aliases_map = RevsetAliasesMap::new();
            for (name, definition) in &aliases {
                let _ = aliases_map.insert(name, definition.clone(), None);
            }
            let before = aliases.len();
            aliases.retain(|(name, _)| {
                self.parse_revset(&aliases_map, &fileset_aliases_map, "", name)
                    .is_ok()
            });
            if aliases.len() == before {
                return aliases;
            }
        }
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
        raw.parse::<ConfigValue>().ok()?.as_str().map(str::to_owned)
    }
}

#[cfg(test)]
mod tests {
    use jj_lib::config::{ConfigLayer, ConfigSource, StackedConfig};
    use jj_test::init_jj_repo;

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

    #[test]
    fn parse_cli_revset_aliases_reads_quoted_names_and_multiline_values() {
        let aliases =
            Repo::parse_cli_revset_aliases(include_str!("testdata/cli_revset_aliases.txt"));

        let trunk = r#"latest(
  remote_bookmarks(exact:"main", exact:"origin") | root()
)
"#;
        assert_eq!(
            aliases,
            [
                ("trunk()".to_owned(), trunk.to_owned()),
                (
                    "immutable()".to_owned(),
                    "::(immutable_heads() | root())".to_owned(),
                ),
            ]
        );
    }

    #[test]
    fn supported_aliases_drop_what_the_embedded_jj_lib_cannot_parse() {
        let temp_dir = init_jj_repo();
        let repo = Repo::open(&temp_dir.path().join("repo")).expect("open repo");

        let kept = repo.supported_aliases(vec![
            ("broken()".to_owned(), "no_such_function()".to_owned()),
            ("uses_broken()".to_owned(), "broken() | root()".to_owned()),
            ("ok()".to_owned(), "root()".to_owned()),
        ]);

        assert_eq!(kept, vec![("ok()".to_owned(), "root()".to_owned())]);
    }
}
