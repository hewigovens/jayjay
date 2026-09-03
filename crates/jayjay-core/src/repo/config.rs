mod env;

use std::collections::HashMap;

use jj_lib::local_working_copy::LocalWorkingCopyFactory;
use jj_lib::workspace::WorkingCopyFactories;

pub(crate) use env::ConfigEnv;

use super::{JJ_CONFIG_USER_EMAIL, JJ_CONFIG_USER_NAME, Repo};

impl Repo {
    /// Warning message when `user.name`/`user.email` are missing from jj config, else `None`.
    pub fn check_user_config(&self) -> Option<String> {
        let has_name = self.run_jj(&["config", "get", JJ_CONFIG_USER_NAME]).is_ok();
        let has_email = self
            .run_jj(&["config", "get", JJ_CONFIG_USER_EMAIL])
            .is_ok();
        if has_name && has_email {
            return None;
        }
        let mut missing = Vec::new();
        if !has_name {
            missing.push(JJ_CONFIG_USER_NAME);
        }
        if !has_email {
            missing.push(JJ_CONFIG_USER_EMAIL);
        }
        Some(missing_user_config_message(&missing))
    }
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
    use super::{JJ_CONFIG_USER_EMAIL, JJ_CONFIG_USER_NAME, missing_user_config_message};

    #[test]
    fn missing_user_config_message_formats_each_command() {
        let message = missing_user_config_message(&[JJ_CONFIG_USER_NAME, JJ_CONFIG_USER_EMAIL]);

        assert_eq!(
            message,
            "jj is not fully configured — user.name and user.email not set. \
             Run `jj config set --user user.name <value>` and \
             `jj config set --user user.email <value>` or edit your config file."
        );
    }
}
