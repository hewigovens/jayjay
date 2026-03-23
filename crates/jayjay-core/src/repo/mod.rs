mod annotate;
mod bookmarks;
mod config;
mod diff;
mod environment;
mod git;
mod log;
mod mutations;
mod resolve;
mod undo;
mod working_copy;

pub use environment::check_jj_environment;
pub use git::COMMIT_MESSAGE_PROMPT;
pub use git::detect_ai_provider;

pub const DEFAULT_REVSET: &str = "@ | ancestors(@, 20) | @-+";

use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use jj_lib::repo::ReadonlyRepo;
use jj_lib::repo_path::RepoPathUiConverter;
use jj_lib::transaction::Transaction;
use jj_lib::workspace::Workspace;
use pollster::FutureExt as _;

use config::{default_settings, working_copy_factories};

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

    pub(crate) fn run_jj(&self, args: &[&str]) -> CoreResult<String> {
        let output = std::process::Command::new(environment::jj_binary())
            .current_dir(&self.path)
            .args(args)
            .output()
            .map_err(|e| CoreError::Internal {
                message: format!("run jj {}: {e}", args.first().unwrap_or(&"")),
            })?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CoreError::Internal {
                message: stderr.trim().to_string(),
            });
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    pub(crate) fn run_jj_reload(&self, args: &[&str]) -> CoreResult<()> {
        self.run_jj(args)?;
        self.reload()
    }

    pub(crate) fn run_jj_quiet(&self, args: &[&str]) {
        let _ = std::process::Command::new(environment::jj_binary())
            .current_dir(&self.path)
            .args(args)
            .output();
    }

    pub(crate) fn get_repo(&self) -> Arc<ReadonlyRepo> {
        self.repo.read().unwrap().clone()
    }

    pub(crate) fn set_repo(&self, repo: Arc<ReadonlyRepo>) {
        *self.repo.write().unwrap() = repo;
    }

    pub(crate) fn reload(&self) -> CoreResult<()> {
        let settings = default_settings()?;
        let store_factories = jj_lib::repo::StoreFactories::default();
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

    pub(crate) fn commit_transaction(&self, tx: Transaction, description: &str) -> CoreResult<()> {
        let new_repo = tx
            .commit(description)
            .block_on()
            .map_err(|e| CoreError::Internal {
                message: format!("commit tx: {e}"),
            })?;
        self.set_repo(new_repo);
        Ok(())
    }

    pub(crate) fn commit_transaction_rebase(
        &self,
        mut tx: Transaction,
        description: &str,
    ) -> CoreResult<()> {
        tx.repo_mut()
            .rebase_descendants()
            .block_on()
            .map_err(|e| CoreError::Internal {
                message: format!("rebase descendants: {e}"),
            })?;
        self.commit_transaction(tx, description)
    }
}
