mod bookmarks;
mod diff;
mod git;
mod log;
mod mutations;
mod undo;
mod working_copy;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use jj_lib::commit::Commit as JjCommit;
use jj_lib::config::StackedConfig;
use jj_lib::fileset::FilesetAliasesMap;
use jj_lib::git::REMOTE_NAME_FOR_LOCAL_GIT_REPO;
use jj_lib::hex_util::encode_reverse_hex;
use jj_lib::local_working_copy::LocalWorkingCopyFactory;
use jj_lib::object_id::ObjectId;
use jj_lib::repo::{ReadonlyRepo, Repo as _, StoreFactories};
use jj_lib::repo_path::RepoPathUiConverter;
use jj_lib::revset::{
    self, RevsetAliasesMap, RevsetDiagnostics, RevsetExtensions, RevsetParseContext,
    RevsetWorkspaceContext, SymbolResolver,
};
use jj_lib::settings::UserSettings;
use jj_lib::time_util::DatePatternContext;
use jj_lib::workspace::{Workspace, WorkingCopyFactories};
use pollster::FutureExt as _;

use crate::types::*;

pub struct Repo {
    pub(crate) path: PathBuf,
    pub(crate) workspace_name: jj_lib::ref_name::WorkspaceNameBuf,
    pub(crate) repo: RwLock<Arc<ReadonlyRepo>>,
}

/// Find the jj binary. macOS app bundles don't inherit shell PATH.
pub(crate) fn jj_binary() -> String {
    let candidates = [
        "/opt/homebrew/bin/jj",
        "/usr/local/bin/jj",
        "/usr/bin/jj",
    ];
    // Check home cargo bin
    if let Ok(home) = std::env::var("HOME") {
        let cargo_jj = format!("{home}/.cargo/bin/jj");
        if std::path::Path::new(&cargo_jj).exists() {
            return cargo_jj;
        }
    }
    for path in candidates {
        if std::path::Path::new(path).exists() {
            return path.to_string();
        }
    }
    "jj".to_string() // fallback to PATH
}

pub(crate) fn working_copy_factories() -> WorkingCopyFactories {
    let mut factories: WorkingCopyFactories = HashMap::new();
    factories.insert("local".to_string(), Box::new(LocalWorkingCopyFactory {}));
    factories
}

pub(crate) fn default_settings() -> Result<UserSettings, CoreError> {
    let config = StackedConfig::with_defaults();
    UserSettings::from_config(config).map_err(|e| CoreError::Internal {
        message: format!("config error: {e}"),
    })
}

impl Repo {
    pub fn open(path: &Path) -> CoreResult<Self> {
        let settings = default_settings()?;
        let store_factories = StoreFactories::default();
        let wc_factories = working_copy_factories();

        let workspace = Workspace::load(&settings, path, &store_factories, &wc_factories)
            .map_err(|e| CoreError::RepoNotFound {
                path: format!("{}: {e}", path.display()),
            })?;

        let repo = workspace
            .repo_loader()
            .load_at_head()
            .block_on()
            .map_err(|e| CoreError::Internal {
                message: format!("failed to load repo: {e}"),
            })?;

        Ok(Self {
            path: workspace.workspace_root().to_owned(),
            workspace_name: workspace.workspace_name().to_owned(),
            repo: RwLock::new(repo),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn path_converter(&self) -> RepoPathUiConverter {
        RepoPathUiConverter::Fs {
            cwd: self.path.clone(),
            base: self.path.clone(),
        }
    }

    pub(crate) fn get_repo(&self) -> Arc<ReadonlyRepo> {
        self.repo.read().unwrap().clone()
    }

    pub(crate) fn set_repo(&self, repo: Arc<ReadonlyRepo>) {
        *self.repo.write().unwrap() = repo;
    }

    pub(crate) fn reload(&self) -> CoreResult<()> {
        let settings = default_settings()?;
        let store_factories = StoreFactories::default();
        let wc_factories = working_copy_factories();
        let workspace = Workspace::load(&settings, &self.path, &store_factories, &wc_factories)
            .map_err(|e| CoreError::Internal {
                message: format!("reload workspace: {e}"),
            })?;
        let repo = workspace
            .repo_loader()
            .load_at_head()
            .block_on()
            .map_err(|e| CoreError::Internal {
                message: format!("reload repo: {e}"),
            })?;
        self.set_repo(repo);
        Ok(())
    }

    fn revset_workspace_context<'a>(
        &'a self,
        path_converter: &'a RepoPathUiConverter,
    ) -> RevsetWorkspaceContext<'a> {
        RevsetWorkspaceContext {
            path_converter,
            workspace_name: self.workspace_name.as_ref(),
        }
    }

    pub(crate) fn resolve_commit(
        &self,
        repo: &Arc<ReadonlyRepo>,
        rev: &str,
    ) -> CoreResult<JjCommit> {
        let settings = repo.settings();
        let aliases_map = RevsetAliasesMap::default();
        let fileset_aliases_map = FilesetAliasesMap::default();
        let extensions = RevsetExtensions::default();
        let path_converter = self.path_converter();

        let context = RevsetParseContext {
            aliases_map: &aliases_map,
            local_variables: HashMap::new(),
            user_email: settings.user_email(),
            date_pattern_context: DatePatternContext::from(chrono::Local::now()),
            default_ignored_remote: Some(REMOTE_NAME_FOR_LOCAL_GIT_REPO),
            fileset_aliases_map: &fileset_aliases_map,
            use_glob_by_default: true,
            extensions: &extensions,
            workspace: Some(self.revset_workspace_context(&path_converter)),
        };

        let mut diagnostics = RevsetDiagnostics::new();
        let expression = revset::parse(&mut diagnostics, rev, &context).map_err(|e| {
            CoreError::RevNotFound {
                rev: format!("{rev}: {e}"),
            }
        })?;

        let empty_extensions: &[&Box<dyn revset::SymbolResolverExtension>] = &[];
        let symbol_resolver = SymbolResolver::new(repo.as_ref(), empty_extensions);
        let resolved = expression
            .resolve_user_expression(repo.as_ref(), &symbol_resolver)
            .map_err(|e| CoreError::RevNotFound {
                rev: format!("{rev}: {e}"),
            })?;

        let revset = resolved
            .evaluate(repo.as_ref())
            .map_err(|e| CoreError::Internal {
                message: format!("revset eval: {e}"),
            })?;

        let commit_id = revset
            .iter()
            .next()
            .ok_or_else(|| CoreError::RevNotFound {
                rev: rev.to_owned(),
            })?
            .map_err(|e| CoreError::Internal {
                message: format!("revset iter: {e}"),
            })?;

        repo.store()
            .get_commit(&commit_id)
            .map_err(|e| CoreError::Internal {
                message: format!("get commit: {e}"),
            })
    }

    pub(crate) fn commit_to_change_info(
        &self,
        repo: &Arc<ReadonlyRepo>,
        commit: &JjCommit,
    ) -> ChangeInfo {
        let change_id = encode_reverse_hex(commit.change_id().as_bytes());
        let commit_id = commit.id().hex();
        let author = commit.author();
        let bookmarks: Vec<String> = repo
            .view()
            .local_bookmarks_for_commit(commit.id())
            .map(|(name, _)| name.as_str().to_owned())
            .collect();
        let wc_id = repo
            .view()
            .get_wc_commit_id(self.workspace_name.as_ref());
        let is_working_copy = wc_id.is_some_and(|id| id == commit.id());
        let has_conflict = commit.has_conflict();
        let is_empty = commit.is_empty(repo.as_ref()).unwrap_or(false);

        ChangeInfo {
            change_id,
            commit_id,
            description: commit.description().to_owned(),
            author: author.name.clone(),
            email: author.email.clone(),
            timestamp_millis: author.timestamp.timestamp.0,
            parents: commit.parent_ids().iter().map(|id| id.hex()).collect(),
            bookmarks,
            is_working_copy,
            has_conflict,
            is_empty,
        }
    }

    pub(crate) fn should_include_in_log(
        &self,
        repo: &Arc<ReadonlyRepo>,
        commit: &JjCommit,
    ) -> bool {
        let author = commit.author();
        let description = commit.description().trim();
        let is_empty_root = commit.parent_ids().is_empty()
            && description.is_empty()
            && author.name.is_empty()
            && author.email.is_empty();
        if is_empty_root {
            return false;
        }
        let wc_id = repo
            .view()
            .get_wc_commit_id(self.workspace_name.as_ref());
        !wc_id.is_some_and(|id| id == commit.id() && commit.parent_ids().is_empty())
    }
}
