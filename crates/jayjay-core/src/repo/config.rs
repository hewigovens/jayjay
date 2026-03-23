use std::collections::HashMap;

use jj_lib::config::StackedConfig;
use jj_lib::local_working_copy::LocalWorkingCopyFactory;
use jj_lib::settings::UserSettings;
use jj_lib::workspace::WorkingCopyFactories;

use crate::types::*;

pub(crate) fn working_copy_factories() -> WorkingCopyFactories {
    let mut factories: WorkingCopyFactories = HashMap::new();
    factories.insert("local".to_string(), Box::new(LocalWorkingCopyFactory {}));
    factories
}

pub(crate) fn default_settings() -> Result<UserSettings, CoreError> {
    let mut config = StackedConfig::with_defaults();
    if let Ok(home) = std::env::var("HOME") {
        let candidates = [
            format!("{home}/.jjconfig.toml"),
            format!("{home}/.config/jj/config.toml"),
        ];
        for path in candidates {
            let p = std::path::PathBuf::from(&path);
            if p.exists() {
                if let Ok(layer) = jj_lib::config::ConfigLayer::load_from_file(
                    jj_lib::config::ConfigSource::User,
                    p,
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
