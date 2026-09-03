use std::collections::HashMap;
use std::path::PathBuf;

use etcetera::BaseStrategy as _;
use jj_lib::config::{ConfigLayer, ConfigResolutionContext, ConfigSource, StackedConfig};
use jj_lib::secure_config::SecureConfig;
use jj_lib::settings::UserSettings;
use jj_lib::workspace::WorkspaceLoader;

use super::super::support::canonicalize;
use crate::types::*;

const ENV_OVERRIDES: [(&str, &str); 6] = [
    ("JJ_USER", "user.name"),
    ("JJ_EMAIL", "user.email"),
    ("JJ_TIMESTAMP", "debug.commit-timestamp"),
    ("JJ_OP_TIMESTAMP", "debug.operation-timestamp"),
    ("JJ_OP_HOSTNAME", "operation.hostname"),
    ("JJ_OP_USERNAME", "operation.username"),
];

/// Config discovery mirroring the jj CLI: system, user, repo, and workspace layers, `JJ_*` overrides, and `[[--scope]]` tables resolved against the workspace being loaded.
pub(crate) struct ConfigEnv {
    home_dir: Option<PathBuf>,
    root_config_dir: Option<PathBuf>,
    config_paths: Vec<(ConfigSource, PathBuf)>,
    hostname: String,
    environment: HashMap<String, String>,
}

impl ConfigEnv {
    pub(crate) fn from_environment() -> Self {
        let home_dir = etcetera::home_dir().ok().map(|dir| canonicalize(&dir));
        let user_config_dir = etcetera::choose_base_strategy()
            .ok()
            .map(|strategy| strategy.config_dir());
        let system_config_dir = cfg!(unix).then(|| PathBuf::from("/etc"));
        let environment = std::env::vars_os()
            .filter_map(|(key, value)| Some((key.into_string().ok()?, value.into_string().ok()?)))
            .collect();
        Self::new(
            home_dir,
            user_config_dir,
            system_config_dir,
            gethostname::gethostname().to_string_lossy().into_owned(),
            environment,
        )
    }

    fn new(
        home_dir: Option<PathBuf>,
        user_config_dir: Option<PathBuf>,
        system_config_dir: Option<PathBuf>,
        hostname: String,
        environment: HashMap<String, String>,
    ) -> Self {
        let root_config_dir = user_config_dir.map(|dir| dir.join("jj"));
        let config_paths = match environment.get("JJ_CONFIG") {
            Some(paths) => std::env::split_paths(paths)
                .filter(|path| !path.as_os_str().is_empty())
                .map(|path| (ConfigSource::User, path))
                .collect(),
            None => system_config_dir
                .iter()
                .flat_map(|dir| [dir.join("jj/config.toml"), dir.join("jj/conf.d")])
                .map(|path| (ConfigSource::System, path))
                .chain(
                    home_dir
                        .iter()
                        .map(|home| home.join(".jjconfig.toml"))
                        .chain(
                            root_config_dir
                                .iter()
                                .flat_map(|root| [root.join("config.toml"), root.join("conf.d")]),
                        )
                        .map(|path| (ConfigSource::User, path)),
                )
                .collect(),
        };
        Self {
            home_dir,
            root_config_dir,
            config_paths,
            hostname,
            environment,
        }
    }

    /// Settings for `loader`'s workspace, resolved the way `jj` run inside it would resolve them.
    pub(crate) fn settings_for_workspace(
        &self,
        loader: &dyn WorkspaceLoader,
    ) -> CoreResult<UserSettings> {
        let repo_path = canonicalize(loader.repo_path());
        let workspace_path = canonicalize(loader.workspace_root());

        let mut config = StackedConfig::with_defaults();
        config.add_layer(self.env_base_layer());
        for (source, path) in &self.config_paths {
            if path.is_dir() {
                config.load_dir(*source, path).map_err(Error::internal)?;
            } else if path.is_file() {
                config.load_file(*source, path).map_err(Error::internal)?;
            }
        }
        if let Some(path) =
            self.secure_config_path(SecureConfig::new_repo(repo_path.clone()), "repos")?
        {
            config
                .load_file(ConfigSource::Repo, path)
                .map_err(Error::internal)?;
        }
        if let Some(path) = self.secure_config_path(
            SecureConfig::new_workspace(workspace_path.join(".jj")),
            "workspaces",
        )? {
            config
                .load_file(ConfigSource::Workspace, path)
                .map_err(Error::internal)?;
        }
        config.add_layer(self.env_overrides_layer());

        let context = ConfigResolutionContext {
            home_dir: self.home_dir.as_deref(),
            repo_path: Some(&repo_path),
            workspace_path: Some(&workspace_path),
            command: None,
            hostname: &self.hostname,
            environment: &self.environment,
        };
        let config = jj_lib::config::resolve(&config, &context).map_err(Error::internal)?;
        UserSettings::from_config(config).map_err(Error::internal)
    }

    /// Per-repo and per-workspace config live under the user config dir, keyed by the id file jj keeps next to the repo.
    fn secure_config_path(&self, config: SecureConfig, kind: &str) -> CoreResult<Option<PathBuf>> {
        let Some(root) = &self.root_config_dir else {
            return Ok(None);
        };
        let loaded = config
            .maybe_load_config(&mut rand::make_rng(), &root.join(kind))
            .map_err(Error::internal)?;
        Ok(loaded.config_file.filter(|path| path.is_file()))
    }

    fn env_base_layer(&self) -> ConfigLayer {
        let username = self
            .environment
            .get("USER")
            .or_else(|| self.environment.get("USERNAME"));
        env_layer(
            ConfigSource::EnvBase,
            [
                ("operation.hostname", Some(&self.hostname)),
                ("operation.username", username),
            ],
        )
    }

    fn env_overrides_layer(&self) -> ConfigLayer {
        env_layer(
            ConfigSource::EnvOverrides,
            ENV_OVERRIDES.map(|(variable, key)| (key, self.environment.get(variable))),
        )
    }
}

fn env_layer<'a>(
    source: ConfigSource,
    values: impl IntoIterator<Item = (&'static str, Option<&'a String>)>,
) -> ConfigLayer {
    let mut layer = ConfigLayer::empty(source);
    for (key, value) in values {
        if let Some(value) = value {
            layer
                .set_value(key, value.as_str())
                .expect("static config key");
        }
    }
    layer
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::PathBuf;

    use jj_lib::settings::UserSettings;
    use jj_lib::workspace::{DefaultWorkspaceLoaderFactory, WorkspaceLoaderFactory as _};
    use tempfile::TempDir;

    use super::ConfigEnv;

    struct Fixture {
        _dir: TempDir,
        home: PathBuf,
        config_dir: PathBuf,
        repo: PathBuf,
    }

    impl Fixture {
        /// `<repo>` in `user_config` expands to the fixture repo's canonical path.
        fn build(user_config: &str) -> Self {
            let dir = tempfile::tempdir().expect("tempdir");
            let root = dunce::canonicalize(dir.path()).expect("canonical tempdir");
            let home = root.join("home");
            let config_dir = root.join("config");
            let repo = root.join("repo");
            std::fs::create_dir_all(&home).expect("create home");
            std::fs::create_dir_all(config_dir.join("jj")).expect("create config dir");
            std::fs::write(
                config_dir.join("jj").join("config.toml"),
                user_config.replace("<repo>", &repo.to_string_lossy()),
            )
            .expect("write user config");
            jj_test::init_colocated(&repo);
            Self {
                _dir: dir,
                home,
                config_dir,
                repo,
            }
        }

        fn settings(&self) -> UserSettings {
            let env = ConfigEnv::new(
                Some(self.home.clone()),
                Some(self.config_dir.clone()),
                None,
                "test-host".to_owned(),
                HashMap::new(),
            );
            let loader = DefaultWorkspaceLoaderFactory
                .create(&self.repo)
                .expect("workspace loader");
            env.settings_for_workspace(loader.as_ref())
                .expect("resolve settings")
        }
    }

    #[test]
    fn scoped_user_config_resolves_against_the_loaded_repo() {
        let fixture = Fixture::build(include_str!("testdata/scoped_user_config.toml"));

        assert_eq!(fixture.settings().user_email(), "work@example.com");
    }

    #[cfg(unix)]
    #[test]
    fn repo_config_overrides_user_config() {
        let fixture = Fixture::build("user.name = \"User Config\"\n");
        let args = ["config", "set", "--repo", "user.name", "Repo Config"];
        let mut command = std::process::Command::new("jj");
        command
            .arg("-R")
            .arg(&fixture.repo)
            .args(args)
            .env("HOME", &fixture.home)
            .env("XDG_CONFIG_HOME", &fixture.config_dir)
            .env_remove("JJ_CONFIG");
        jj_test::run_command("jj", &args.map(String::from), &mut command);

        assert_eq!(fixture.settings().user_name(), "Repo Config");
    }
}
