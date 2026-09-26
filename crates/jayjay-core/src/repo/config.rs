mod env;
mod listing;

use std::collections::HashMap;

use jj_lib::local_working_copy::LocalWorkingCopyFactory;
use jj_lib::settings::UserSettings;
use jj_lib::workspace::WorkingCopyFactories;

pub(crate) use env::ConfigEnv;
pub use listing::{JjConfigEntry, JjConfigSection, JjUserConfig, jj_user_config};

use super::{JJ_CONFIG_USER_EMAIL, JJ_CONFIG_USER_NAME, Repo};

impl Repo {
    /// Warning message when `user.name`/`user.email` are missing from jj config, else `None`.
    pub fn check_user_config(&self) -> Option<String> {
        missing_user_config(self.get_repo().settings())
    }
}

fn missing_user_config(settings: &UserSettings) -> Option<String> {
    let missing: Vec<&str> = [
        (JJ_CONFIG_USER_NAME, settings.user_name()),
        (JJ_CONFIG_USER_EMAIL, settings.user_email()),
    ]
    .into_iter()
    .filter(|(_, value)| value.is_empty())
    .map(|(key, _)| key)
    .collect();
    if missing.is_empty() {
        return None;
    }
    Some(missing_user_config_message(&missing))
}

fn missing_user_config_message(missing: &[&str]) -> String {
    let commands = missing
        .iter()
        .map(|key| format!("`jj config set --user {key} <value>`"))
        .collect::<Vec<_>>()
        .join(" and ");
    format!(
        "jj is not fully configured — {} not set. Run {commands} or edit your config file.",
        missing.join(" and "),
    )
}

pub(crate) fn working_copy_factories() -> WorkingCopyFactories {
    let mut factories: WorkingCopyFactories = HashMap::new();
    factories.insert("local".to_string(), Box::new(LocalWorkingCopyFactory {}));
    factories
}

#[cfg(test)]
mod tests {
    use jj_lib::config::{ConfigLayer, ConfigSource, StackedConfig};
    use jj_lib::settings::UserSettings;

    use super::missing_user_config;

    fn settings(user_toml: &str) -> UserSettings {
        let mut config = StackedConfig::with_defaults();
        config.add_layer(ConfigLayer::parse(ConfigSource::User, user_toml).expect("parse config"));
        UserSettings::from_config(config).expect("settings")
    }

    #[test]
    fn missing_user_config_names_each_unset_key() {
        assert_eq!(
            missing_user_config(&settings("[user]\nname = 'Dev'\nemail = 'dev@example.com'")),
            None
        );
        assert_eq!(
            missing_user_config(&settings("[user]\nname = 'Dev'")).as_deref(),
            Some(
                "jj is not fully configured — user.email not set. \
                 Run `jj config set --user user.email <value>` or edit your config file."
            )
        );
        assert_eq!(
            missing_user_config(&settings("")).as_deref(),
            Some(
                "jj is not fully configured — user.name and user.email not set. \
                 Run `jj config set --user user.name <value>` and \
                 `jj config set --user user.email <value>` or edit your config file."
            )
        );
    }
}
