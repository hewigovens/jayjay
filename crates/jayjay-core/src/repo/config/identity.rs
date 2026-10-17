use std::path::PathBuf;

use jj_lib::config::{ConfigFile, ConfigSource, StackedConfig};

use super::{ConfigEnv, JJ_CONFIG_USER_EMAIL, JJ_CONFIG_USER_NAME};
use crate::repo::subprocess_command;
use crate::types::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserIdentity {
    pub name: String,
    pub email: String,
}

impl UserIdentity {
    /// jj's own user-level identity, when it sets both values.
    pub fn configured() -> JayResult<Option<Self>> {
        let identity = Self::read(&ConfigEnv::from_environment())?;
        Ok((!identity.name.is_empty() && !identity.email.is_empty()).then_some(identity))
    }

    /// jj's user-level values, with unset ones taken from git's global identity, which jj never reads.
    pub fn suggested() -> JayResult<Self> {
        let Self { name, email } = Self::read(&ConfigEnv::from_environment())?;
        let or_git = |value: String, key| {
            if value.is_empty() {
                git_global_config(key)
            } else {
                value
            }
        };
        Ok(Self {
            name: or_git(name, JJ_CONFIG_USER_NAME),
            email: or_git(email, JJ_CONFIG_USER_EMAIL),
        })
    }

    /// The file `save` writes.
    pub fn config_path() -> JayResult<String> {
        Ok(config_file(&ConfigEnv::from_environment())?
            .display()
            .to_string())
    }

    pub fn save(&self) -> JayResult<()> {
        self.save_in(&ConfigEnv::from_environment())
    }

    fn read(env: &ConfigEnv) -> JayResult<Self> {
        Ok(Self::from_config(&user_level_config(env)?))
    }

    fn from_config(config: &StackedConfig) -> Self {
        let value = |key| config.get::<String>(key).unwrap_or_default();
        Self {
            name: value(JJ_CONFIG_USER_NAME),
            email: value(JJ_CONFIG_USER_EMAIL),
        }
    }

    fn save_in(&self, env: &ConfigEnv) -> JayResult<()> {
        let saved = Self {
            name: self.name.trim().to_owned(),
            email: self.email.trim().to_owned(),
        };
        let path = config_file(env)?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(JayError::internal)?;
        }
        let mut file =
            ConfigFile::load_or_empty(ConfigSource::User, path).map_err(JayError::internal)?;
        file.set_value(JJ_CONFIG_USER_NAME, saved.name.as_str())
            .and_then(|_| file.set_value(JJ_CONFIG_USER_EMAIL, saved.email.as_str()))
            .map_err(JayError::internal)?;
        file.save().map_err(JayError::internal)?;

        // A later config file or `JJ_USER`/`JJ_EMAIL` can still win over the file just written.
        let config = user_level_config(env)?;
        let effective = Self::from_config(&config);
        let key = if effective.name != saved.name {
            JJ_CONFIG_USER_NAME
        } else if effective.email != saved.email {
            JJ_CONFIG_USER_EMAIL
        } else {
            return Ok(());
        };
        let layer = config
            .layers()
            .iter()
            .rev()
            .find(|layer| !matches!(layer.look_up_item(key), Ok(None)));
        let source = match layer {
            Some(layer) if layer.source == ConfigSource::EnvOverrides => {
                ConfigEnv::override_variable(key).map(str::to_owned)
            }
            Some(layer) => layer.path.as_ref().map(|path| path.display().to_string()),
            None => None,
        }
        .unwrap_or_else(|| "another jj config file".to_owned());
        Err(JayError::internal(format!(
            "Saved, but {source} overrides {key}"
        )))
    }
}

impl std::fmt::Display for UserIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} <{}>", self.name, self.email)
    }
}

fn user_level_config(env: &ConfigEnv) -> JayResult<StackedConfig> {
    env.user_level_config()
        .map_err(|failure| JayError::internal(failure.message))
}

/// A user config directory (conf.d or a `JJ_CONFIG` directory) gets its own `config.toml`.
fn config_file(env: &ConfigEnv) -> JayResult<PathBuf> {
    let path = env
        .user_config_path()
        .ok_or_else(|| JayError::internal("no home directory for the jj user config"))?;
    Ok(if path.is_dir() {
        path.join("config.toml")
    } else {
        path
    })
}

fn git_global_config(key: &str) -> String {
    subprocess_command("git")
        .args(["config", "--global", "--includes", "--get", key])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::Path;

    use super::{ConfigEnv, UserIdentity};

    fn env(root: &Path, environment: &[(&str, &str)]) -> ConfigEnv {
        ConfigEnv::new(
            Some(root.join("home")),
            Some(root.join("config")),
            None,
            "test-host".to_owned(),
            environment
                .iter()
                .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
                .collect::<HashMap<_, _>>(),
        )
    }

    fn identity(name: &str, email: &str) -> UserIdentity {
        UserIdentity {
            name: name.to_owned(),
            email: email.to_owned(),
        }
    }

    #[test]
    fn saving_creates_the_file_or_keeps_the_rest_of_it() {
        let created = tempfile::tempdir().expect("tempdir");
        let env_created = env(created.path(), &[]);
        identity(" Dev ", "dev@example.com")
            .save_in(&env_created)
            .expect("save into a missing file");
        assert_eq!(
            UserIdentity::read(&env_created).expect("read"),
            identity("Dev", "dev@example.com")
        );

        let kept = tempfile::tempdir().expect("tempdir");
        let file = kept.path().join("config").join("jj").join("config.toml");
        std::fs::create_dir_all(file.parent().expect("config dir")).expect("create config dir");
        std::fs::write(
            &file,
            "# mine\n[ui]\neditor = \"vim\"\n\n[user]\nname = \"Old\"\n",
        )
        .expect("write config");
        let env_kept = env(kept.path(), &[]);
        assert_eq!(
            UserIdentity::read(&env_kept).expect("read"),
            identity("Old", "")
        );

        identity("New", "new@example.com")
            .save_in(&env_kept)
            .expect("save over a partial identity");

        let text = std::fs::read_to_string(&file).expect("read config");
        assert!(
            text.starts_with("# mine\n[ui]\neditor = \"vim\"\n"),
            "{text}"
        );
        assert_eq!(
            UserIdentity::read(&env_kept).expect("read"),
            identity("New", "new@example.com")
        );
    }

    #[test]
    fn saving_reports_what_still_overrides_the_saved_identity() {
        let dir = tempfile::tempdir().expect("tempdir");
        let save = |env: &ConfigEnv| {
            identity("Dev", "dev@example.com")
                .save_in(env)
                .expect_err("overridden")
                .to_string()
        };
        let error = save(&env(dir.path(), &[("JJ_EMAIL", "env@example.com")]));
        assert!(error.contains("JJ_EMAIL overrides user.email"), "{error}");

        let first = dir.path().join("first.toml");
        let later = dir.path().join("later.toml");
        std::fs::write(&first, "").expect("write first config");
        let jj_config = std::env::join_paths([&first, &later]).expect("join paths");
        let env = env(dir.path(), &[("JJ_CONFIG", &jj_config.to_string_lossy())]);
        let expected = format!("{} overrides user.name", later.display());
        for later_config in ["[user]\nname = \"Later\"\n", "user = \"broken\"\n"] {
            std::fs::write(&later, later_config).expect("write later config");
            let error = save(&env);
            assert!(error.contains(&expected), "{error}");
        }
    }
}
