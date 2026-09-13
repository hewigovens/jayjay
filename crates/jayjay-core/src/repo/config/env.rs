use std::collections::HashMap;
use std::path::{Path, PathBuf};

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

    pub(crate) fn new(
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

        let mut config = self.stacked_base_config()?;
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
        self.finish_settings(config, &repo_path, &workspace_path, None)
    }

    pub(crate) fn user_config_snapshot(&self) -> super::JjUserConfigSnapshot {
        let files = match self.existing_user_config_files() {
            Ok(files) => files,
            Err((path, error)) => {
                return super::JjUserConfigSnapshot {
                    path: path.display().to_string(),
                    listing: String::new(),
                    error: Some(error),
                };
            }
        };
        let Some(path) = files.first() else {
            return super::JjUserConfigSnapshot::default();
        };
        let mut order = Vec::new();
        let mut values = HashMap::new();
        let mut next_scope = 0;
        for file in &files {
            let text = match std::fs::read_to_string(file) {
                Ok(text) => text,
                Err(error) => return user_config_error(path, file, error),
            };
            let table = match text.parse::<toml::Table>() {
                Ok(table) => table,
                Err(error) => return user_config_error(path, file, error),
            };
            let mut file_entries = Vec::new();
            flatten_toml_value("", &toml::Value::Table(table), &mut file_entries);
            for (key, value) in rematch_scope_keys(file_entries, &mut next_scope) {
                insert_flattened_key(&mut order, &mut values, key, value);
            }
        }
        super::JjUserConfigSnapshot {
            path: path.display().to_string(),
            listing: order
                .into_iter()
                .filter_map(|key| values.remove(&key).map(|value| format!("{key} = {value}")))
                .collect::<Vec<_>>()
                .join("\n"),
            error: None,
        }
    }

    fn existing_user_config_files(&self) -> Result<Vec<PathBuf>, (PathBuf, String)> {
        let mut files = Vec::new();
        for (source, path) in &self.config_paths {
            if *source != ConfigSource::User {
                continue;
            }
            if path.is_file() {
                files.push(path.clone());
            } else if path.is_dir() {
                let entries = std::fs::read_dir(path)
                    .map_err(|error| (path.clone(), format!("{}: {error}", path.display())))?;
                let mut dir_files = Vec::new();
                for entry in entries {
                    let entry = entry
                        .map_err(|error| (path.clone(), format!("{}: {error}", path.display())))?;
                    let file = entry.path();
                    if file.is_file() && file.extension().is_some_and(|ext| ext == "toml") {
                        dir_files.push(file);
                    }
                }
                dir_files.sort();
                files.extend(dir_files);
            }
        }
        Ok(files)
    }

    /// Settings for a destination that does not have a jj workspace yet (`jj git init`).
    pub(crate) fn settings_for_new_workspace(
        &self,
        workspace_root: &Path,
    ) -> CoreResult<UserSettings> {
        let workspace_path = canonicalize(workspace_root);
        let repo_path = workspace_path.join(".jj").join("repo");
        self.finish_settings(
            self.stacked_base_config()?,
            &repo_path,
            &workspace_path,
            Some("git init"),
        )
    }

    fn stacked_base_config(&self) -> CoreResult<StackedConfig> {
        let mut config = StackedConfig::with_defaults();
        config.add_layer(self.env_base_layer());
        for (source, path) in &self.config_paths {
            if path.is_dir() {
                config.load_dir(*source, path).map_err(Error::internal)?;
            } else if path.is_file() {
                config.load_file(*source, path).map_err(Error::internal)?;
            }
        }
        Ok(config)
    }

    fn finish_settings(
        &self,
        mut config: StackedConfig,
        repo_path: &Path,
        workspace_path: &Path,
        command: Option<&str>,
    ) -> CoreResult<UserSettings> {
        config.add_layer(self.env_overrides_layer());
        let context = ConfigResolutionContext {
            home_dir: self.home_dir.as_deref(),
            repo_path: Some(repo_path),
            workspace_path: Some(workspace_path),
            command,
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

fn user_config_error(
    path: &Path,
    file: &Path,
    error: impl std::fmt::Display,
) -> super::JjUserConfigSnapshot {
    super::JjUserConfigSnapshot {
        path: path.display().to_string(),
        listing: String::new(),
        error: Some(format!("{}: {error}", file.display())),
    }
}

fn flatten_toml_value(prefix: &str, value: &toml::Value, out: &mut Vec<(String, String)>) {
    match value {
        toml::Value::Table(table) => {
            for (key, child) in table {
                flatten_toml_value(&flatten_toml_key(prefix, key), child, out);
            }
        }
        toml::Value::Array(items)
            if !items.is_empty() && items.iter().all(toml::Value::is_table) =>
        {
            for (index, child) in items.iter().enumerate() {
                flatten_toml_value(&format!("{prefix}[{index}]"), child, out);
            }
        }
        other => out.push((prefix.to_owned(), other.to_string())),
    }
}

fn flatten_toml_key(prefix: &str, key: &str) -> String {
    let segment = if key.contains('.') {
        format!("\"{}\"", key.replace('"', "\\\""))
    } else {
        key.to_owned()
    };
    if prefix.is_empty() {
        segment
    } else {
        format!("{prefix}.{segment}")
    }
}

fn insert_flattened_key(
    order: &mut Vec<String>,
    values: &mut HashMap<String, String>,
    key: String,
    value: String,
) {
    if values.contains_key(&key) {
        values.insert(key, value);
        return;
    }
    values.retain(|existing, _| {
        !is_config_path_prefix(existing, &key) && !is_config_path_prefix(&key, existing)
    });
    order.retain(|existing| values.contains_key(existing));
    values.insert(key.clone(), value);
    order.push(key);
}

fn is_config_path_prefix(prefix: &str, key: &str) -> bool {
    key.starts_with(prefix) && key.len() > prefix.len() && key.as_bytes()[prefix.len()] == b'.'
}

/// Scopes from separate user files are additive; reindex `--scope[N]` so a later file cannot replace an earlier file's tables.
fn rematch_scope_keys(
    entries: Vec<(String, String)>,
    next_scope: &mut usize,
) -> Vec<(String, String)> {
    let mut local_to_global = HashMap::new();
    entries
        .into_iter()
        .map(|(key, value)| match split_scope_index(&key) {
            Some((local, rest)) => {
                let global = *local_to_global.entry(local).or_insert_with(|| {
                    let id = *next_scope;
                    *next_scope += 1;
                    id
                });
                (format!("--scope[{global}]{rest}"), value)
            }
            None => (key, value),
        })
        .collect()
}

fn split_scope_index(key: &str) -> Option<(usize, &str)> {
    let rest = key.strip_prefix("--scope[")?;
    let (index, rest) = rest.split_once(']')?;
    Some((index.parse().ok()?, rest))
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

    #[test]
    fn user_config_snapshot_lists_existing_file() {
        let fixture = Fixture::build("user.name = \"Alice\"\nuser.email = \"a@example.com\"\n");
        let env = ConfigEnv::new(
            Some(fixture.home.clone()),
            Some(fixture.config_dir.clone()),
            None,
            "test-host".to_owned(),
            HashMap::new(),
        );

        let snapshot = env.user_config_snapshot();

        assert!(
            snapshot.path.ends_with("config.toml"),
            "path: {}",
            snapshot.path
        );
        assert!(snapshot.listing.contains("user.name = \"Alice\""));
        assert!(snapshot.listing.contains("user.email = \"a@example.com\""));
    }

    #[test]
    fn user_config_snapshot_expands_scoped_tables() {
        let fixture = Fixture::build(include_str!("testdata/scoped_user_config.toml"));
        let env = ConfigEnv::new(
            Some(fixture.home.clone()),
            Some(fixture.config_dir.clone()),
            None,
            "test-host".to_owned(),
            HashMap::new(),
        );

        let snapshot = env.user_config_snapshot();

        assert!(
            snapshot
                .listing
                .contains("user.email = \"personal@example.com\""),
            "listing: {}",
            snapshot.listing
        );
        assert!(
            snapshot
                .listing
                .contains("--scope[0].user.email = \"work@example.com\""),
            "listing: {}",
            snapshot.listing
        );
        assert!(
            snapshot
                .listing
                .contains("--scope[0].--when.repositories = ["),
            "listing: {}",
            snapshot.listing
        );
        assert!(
            !snapshot.listing.contains("--scope ="),
            "listing: {}",
            snapshot.listing
        );
    }

    #[test]
    fn user_config_snapshot_scopes_from_later_files_are_additive() {
        let fixture = Fixture::build(
            "[[--scope]]\n--when.commands = [\"log\"]\n[--scope.user]\nname = \"Logger\"\n",
        );
        let conf_d = fixture.config_dir.join("jj").join("conf.d");
        std::fs::create_dir_all(&conf_d).expect("conf.d");
        std::fs::write(
            conf_d.join("extra.toml"),
            "[[--scope]]\n--when.commands = [\"diff\"]\n[--scope.user]\nname = \"Differ\"\n",
        )
        .expect("extra scope");
        let env = ConfigEnv::new(
            Some(fixture.home.clone()),
            Some(fixture.config_dir.clone()),
            None,
            "test-host".to_owned(),
            HashMap::new(),
        );

        let snapshot = env.user_config_snapshot();

        assert!(
            snapshot
                .listing
                .contains("--scope[0].user.name = \"Logger\""),
            "listing: {}",
            snapshot.listing
        );
        assert!(
            snapshot
                .listing
                .contains("--scope[1].user.name = \"Differ\""),
            "listing: {}",
            snapshot.listing
        );
        assert!(
            snapshot
                .listing
                .contains("--scope[0].--when.commands = [\"log\"]"),
            "listing: {}",
            snapshot.listing
        );
        assert!(
            snapshot
                .listing
                .contains("--scope[1].--when.commands = [\"diff\"]"),
            "listing: {}",
            snapshot.listing
        );
    }

    #[test]
    fn new_workspace_settings_resolve_future_repo_path_scopes() {
        let fixture = Fixture::build(
            "git.object-hash = \"sha1\"\n\n[[--scope]]\n--when.repositories = [\"<repo>/.jj/repo\"]\n[--scope.git]\nobject-hash = \"sha256\"\n",
        );
        let env = ConfigEnv::new(
            Some(fixture.home.clone()),
            Some(fixture.config_dir.clone()),
            None,
            "test-host".to_owned(),
            HashMap::new(),
        );

        let init_settings = env
            .settings_for_new_workspace(&fixture.repo)
            .expect("resolve init settings");
        assert_eq!(
            init_settings
                .get_string("git.object-hash")
                .expect("init object-hash"),
            "sha256"
        );
        assert_eq!(
            fixture
                .settings()
                .get_string("git.object-hash")
                .expect("workspace object-hash"),
            "sha256"
        );
    }

    #[test]
    fn new_workspace_settings_resolve_git_init_command_scopes() {
        let fixture = Fixture::build(
            "git.object-hash = \"sha1\"\n\n[[--scope]]\n--when.commands = [\"git init\"]\n[--scope.git]\nobject-hash = \"sha256\"\n",
        );
        let env = ConfigEnv::new(
            Some(fixture.home.clone()),
            Some(fixture.config_dir.clone()),
            None,
            "test-host".to_owned(),
            HashMap::new(),
        );

        let init_settings = env
            .settings_for_new_workspace(&fixture.repo)
            .expect("resolve init settings");
        assert_eq!(
            init_settings
                .get_string("git.object-hash")
                .expect("init object-hash"),
            "sha256"
        );
        assert_eq!(
            fixture
                .settings()
                .get_string("git.object-hash")
                .expect("workspace object-hash"),
            "sha1"
        );
    }

    #[test]
    fn user_config_snapshot_drops_shadowed_keys() {
        let fixture = Fixture::build(
            "revset-aliases.\"mine()\" = \"all()\"\n[revset-aliases.\"ours()\"]\ndefinition = \"trunk()\"\n",
        );
        let conf_d = fixture.config_dir.join("jj").join("conf.d");
        std::fs::create_dir_all(&conf_d).expect("conf.d");
        std::fs::write(
            conf_d.join("override.toml"),
            "revset-aliases.\"ours()\" = \"none()\"\n[revset-aliases.\"mine()\"]\ndefinition = \"immutable_heads()\"\n",
        )
        .expect("override");
        let env = ConfigEnv::new(
            Some(fixture.home.clone()),
            Some(fixture.config_dir.clone()),
            None,
            "test-host".to_owned(),
            HashMap::new(),
        );

        let snapshot = env.user_config_snapshot();

        assert!(
            snapshot
                .listing
                .contains("revset-aliases.mine().definition = \"immutable_heads()\""),
            "listing: {}",
            snapshot.listing
        );
        assert!(
            snapshot
                .listing
                .contains("revset-aliases.ours() = \"none()\""),
            "listing: {}",
            snapshot.listing
        );
        assert!(
            !snapshot
                .listing
                .contains("revset-aliases.mine() = \"all()\""),
            "listing: {}",
            snapshot.listing
        );
        assert!(
            !snapshot
                .listing
                .contains("revset-aliases.ours().definition"),
            "listing: {}",
            snapshot.listing
        );
    }

    #[test]
    fn user_config_snapshot_last_user_layer_wins() {
        let fixture = Fixture::build("user.name = \"Alice\"\nuser.email = \"a@example.com\"\n");
        let conf_d = fixture.config_dir.join("jj").join("conf.d");
        std::fs::create_dir_all(&conf_d).expect("conf.d");
        std::fs::write(conf_d.join("override.toml"), "user.name = \"Bob\"\n").expect("override");
        let env = ConfigEnv::new(
            Some(fixture.home.clone()),
            Some(fixture.config_dir.clone()),
            None,
            "test-host".to_owned(),
            HashMap::new(),
        );

        let snapshot = env.user_config_snapshot();

        assert!(snapshot.listing.contains("user.name = \"Bob\""));
        assert!(!snapshot.listing.contains("Alice"));
        assert_eq!(
            snapshot.listing.matches("user.name").count(),
            1,
            "listing: {}",
            snapshot.listing
        );
    }

    #[test]
    fn user_config_snapshot_preserves_dotted_quoted_keys() {
        let fixture = Fixture::build("\"foo.bar\" = 1\n[foo]\nbar = 2\n");
        let env = ConfigEnv::new(
            Some(fixture.home.clone()),
            Some(fixture.config_dir.clone()),
            None,
            "test-host".to_owned(),
            HashMap::new(),
        );

        let snapshot = env.user_config_snapshot();

        assert!(
            snapshot.listing.contains("\"foo.bar\" = 1"),
            "listing: {}",
            snapshot.listing
        );
        assert!(
            snapshot.listing.contains("foo.bar = 2"),
            "listing: {}",
            snapshot.listing
        );
    }

    #[test]
    fn user_config_snapshot_preserves_equals_in_quoted_keys() {
        let fixture = Fixture::build("[remotes.\"foo=bar\"]\nauto-track-bookmarks = \"glob:*\"\n");
        let env = ConfigEnv::new(
            Some(fixture.home.clone()),
            Some(fixture.config_dir.clone()),
            None,
            "test-host".to_owned(),
            HashMap::new(),
        );

        let snapshot = env.user_config_snapshot();

        assert!(
            snapshot
                .listing
                .contains("remotes.foo=bar.auto-track-bookmarks = \"glob:*\""),
            "listing: {}",
            snapshot.listing
        );
    }

    #[cfg(unix)]
    #[test]
    fn user_config_snapshot_surfaces_unreadable_config_dir() {
        use std::os::unix::fs::PermissionsExt;

        let fixture = Fixture::build("user.name = \"Alice\"\n");
        let conf_d = fixture.config_dir.join("jj").join("conf.d");
        std::fs::create_dir_all(&conf_d).expect("conf.d");
        let perms = std::fs::Permissions::from_mode(0o000);
        std::fs::set_permissions(&conf_d, perms).expect("chmod conf.d");
        let env = ConfigEnv::new(
            Some(fixture.home.clone()),
            Some(fixture.config_dir.clone()),
            None,
            "test-host".to_owned(),
            HashMap::new(),
        );

        let snapshot = env.user_config_snapshot();
        let _ = std::fs::set_permissions(&conf_d, std::fs::Permissions::from_mode(0o755));

        assert!(
            snapshot
                .error
                .as_deref()
                .is_some_and(|error| error.contains("conf.d")),
            "error: {:?}",
            snapshot.error
        );
        assert!(snapshot.listing.is_empty(), "listing: {}", snapshot.listing);
    }

    #[test]
    fn user_config_snapshot_surfaces_invalid_toml() {
        let fixture = Fixture::build("user.name = \"Alice\"\n");
        let conf_d = fixture.config_dir.join("jj").join("conf.d");
        std::fs::create_dir_all(&conf_d).expect("conf.d");
        std::fs::write(conf_d.join("bad.toml"), "not = toml [\n").expect("invalid toml");
        let env = ConfigEnv::new(
            Some(fixture.home.clone()),
            Some(fixture.config_dir.clone()),
            None,
            "test-host".to_owned(),
            HashMap::new(),
        );

        let snapshot = env.user_config_snapshot();

        assert!(
            snapshot
                .error
                .as_deref()
                .is_some_and(|error| error.contains("bad.toml")),
            "error: {:?}",
            snapshot.error
        );
        assert!(snapshot.listing.is_empty(), "listing: {}", snapshot.listing);
    }

    #[test]
    fn user_config_snapshot_is_empty_when_file_missing() {
        let dir = tempfile::tempdir().expect("tempdir");
        let home = dir.path().join("home");
        let config_dir = dir.path().join("config");
        std::fs::create_dir_all(&home).expect("home");
        std::fs::create_dir_all(&config_dir).expect("config");
        let env = ConfigEnv::new(
            Some(home),
            Some(config_dir),
            None,
            "test-host".to_owned(),
            HashMap::new(),
        );

        assert_eq!(
            env.user_config_snapshot(),
            super::super::JjUserConfigSnapshot::default()
        );
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
