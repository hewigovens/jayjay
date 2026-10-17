mod env;
mod identity;
mod listing;

use std::collections::HashMap;

use jj_lib::local_working_copy::LocalWorkingCopyFactory;
use jj_lib::workspace::WorkingCopyFactories;

pub(crate) use env::ConfigEnv;
pub use identity::UserIdentity;
pub use listing::{JjConfigEntry, JjConfigSection, JjUserConfig, jj_user_config};

pub const JJ_CONFIG_USER_NAME: &str = "user.name";
pub const JJ_CONFIG_USER_EMAIL: &str = "user.email";

pub(crate) fn working_copy_factories() -> WorkingCopyFactories {
    let mut factories: WorkingCopyFactories = HashMap::new();
    factories.insert("local".to_string(), Box::new(LocalWorkingCopyFactory {}));
    factories
}
