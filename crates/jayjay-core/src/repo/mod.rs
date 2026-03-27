mod annotate;
mod bookmarks;
mod command;
mod config;
mod conflicts;
mod diff;
mod diffedit;
mod environment;
mod git;
mod git_ai;
mod log;
mod mutations;
mod mutations_files;
mod resolve;
mod support;
mod transaction;
mod undo;
mod working_copy;
mod workspace;

pub use environment::check_jj_environment;
pub use git::COMMIT_MESSAGE_PROMPT;
pub use git::detect_ai_provider;

pub const DEFAULT_REVSET_DEPTH: u32 = 20;
pub const DEFAULT_REVSET: &str = "present(@) | ancestors(trunk().., 20) | trunk()";

pub fn build_default_revset(depth: u32) -> String {
    format!("present(@) | ancestors(trunk().., {depth}) | trunk()")
}

use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use jj_lib::repo::ReadonlyRepo;
use jj_lib::repo_path::RepoPathBuf;
use jj_lib::repo_path::RepoPathUiConverter;
use jj_lib::transaction::Transaction;
use jj_lib::workspace::Workspace;

use config::{default_settings, working_copy_factories};
use support::{block_on_result, load_repo_at_head, load_workspace_internal};

use crate::types::*;

pub struct Repo {
    pub(crate) path: PathBuf,
    pub(crate) workspace_name: jj_lib::ref_name::WorkspaceNameBuf,
    pub(crate) repo: RwLock<Arc<ReadonlyRepo>>,
}

impl Repo {
    pub fn open(path: &Path) -> CoreResult<Self> {
        let settings = default_settings()?;
        let store_factories = jj_lib::repo::StoreFactories::default();
        let wc_factories = working_copy_factories();

        let workspace =
            Workspace::load(&settings, path, &store_factories, &wc_factories).map_err(|e| {
                CoreError::RepoNotFound {
                    path: format!("{}: {e}", path.display()),
                }
            })?;

        let repo = load_repo_at_head(&workspace, "failed to load repo")?;

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

    pub(crate) fn parse_repo_path(&self, path: &str) -> CoreResult<RepoPathBuf> {
        RepoPathBuf::parse_fs_path(&self.path, &self.path, path).map_err(|e| CoreError::Internal {
            message: format!("invalid path {path}: {e}"),
        })
    }

    pub(crate) fn parse_repo_paths(&self, paths: &[String]) -> CoreResult<Vec<RepoPathBuf>> {
        paths
            .iter()
            .map(|path| self.parse_repo_path(path))
            .collect()
    }

    pub(crate) fn reload(&self) -> CoreResult<()> {
        let workspace = load_workspace_internal(&self.path, "reload workspace")?;
        let repo = load_repo_at_head(&workspace, "reload repo")?;
        self.set_repo(repo);
        Ok(())
    }

    pub(crate) fn commit_transaction(&self, tx: Transaction, description: &str) -> CoreResult<()> {
        let new_repo = self.commit_transaction_to_repo(tx, description)?;
        self.set_repo(new_repo);
        Ok(())
    }

    pub(crate) fn commit_transaction_rebase(
        &self,
        mut tx: Transaction,
        description: &str,
    ) -> CoreResult<()> {
        block_on_result("rebase descendants", tx.repo_mut().rebase_descendants())?;
        self.commit_transaction(tx, description)
    }

    pub(crate) fn commit_transaction_to_repo(
        &self,
        tx: Transaction,
        description: &str,
    ) -> CoreResult<Arc<ReadonlyRepo>> {
        block_on_result("commit tx", tx.commit(description))
    }

    pub(crate) fn commit_transaction_rebase_to_repo(
        &self,
        mut tx: Transaction,
        description: &str,
    ) -> CoreResult<Arc<ReadonlyRepo>> {
        block_on_result("rebase descendants", tx.repo_mut().rebase_descendants())?;
        self.commit_transaction_to_repo(tx, description)
    }
}
