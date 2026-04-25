use std::collections::HashMap;

use jj_lib::config::StackedConfig;
use jj_lib::local_working_copy::LocalWorkingCopyFactory;
use jj_lib::settings::UserSettings;
use jj_lib::workspace::WorkingCopyFactories;

use super::environment;
use crate::types::*;

pub(crate) fn working_copy_factories() -> WorkingCopyFactories {
    let mut factories: WorkingCopyFactories = HashMap::new();
    factories.insert("local".to_string(), Box::new(LocalWorkingCopyFactory {}));
    factories
}

pub(crate) fn default_settings() -> Result<UserSettings, CoreError> {
    let mut config = StackedConfig::with_defaults();
    if let Some(home) = environment::home_dir() {
        let candidates = [
            home.join(".jjconfig.toml"),
            home.join(".config").join("jj").join("config.toml"),
        ];
        for path in candidates {
            if path.exists() {
                if let Ok(layer) = jj_lib::config::ConfigLayer::load_from_file(
                    jj_lib::config::ConfigSource::User,
                    path,
                ) {
                    config.add_layer(layer);
                }
                break;
            }
        }
    }
    UserSettings::from_config(config).map_err(|e| CoreError::Internal {
        message: format!("config error: {e}"),
    })
}
