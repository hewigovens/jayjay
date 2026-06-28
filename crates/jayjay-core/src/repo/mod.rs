mod annotate;
mod bookmarks;
mod command;
mod commit_ai;
mod config;
mod conflicts;
mod diff;
mod diffedit;
mod environment;
mod evolog;
mod git;
mod hosted_repo;
mod init;
mod log;
mod mutations;
mod mutations_files;
mod path_operands;
mod platform;
mod pull_requests;
mod resolve;
mod stacked_pr;
mod support;
mod transaction;
mod undo;
mod working_copy;
mod working_copy_ignore;
mod workspace;

pub use commit_ai::COMMIT_MESSAGE_PROMPT;
pub use commit_ai::detect_ai_provider;
pub use environment::check_gh_environment;
pub use environment::check_glab_environment;
pub use environment::check_jj_environment;
pub(crate) use environment::command as subprocess_command;
pub use environment::find_existing_binary;
pub use environment::jj_binary;
pub use environment::login_shell;
pub use environment::login_shell_path;
pub use init::init_jj_git_repo;
pub use stacked_pr::is_valid_bookmark_name;

pub const DEFAULT_REVSET_DEPTH: u32 = 20;
pub const DEFAULT_REVSET: &str = "present(@) | ancestors(immutable_heads().., 20) | trunk()";

pub const JJ_CONFIG_USER_NAME: &str = "user.name";
pub const JJ_CONFIG_USER_EMAIL: &str = "user.email";

pub fn build_default_revset(depth: u32) -> String {
    format!("present(@) | ancestors(immutable_heads().., {depth}) | trunk()")
}

use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use jj_lib::repo::ReadonlyRepo;
use jj_lib::repo_path::RepoPathBuf;
use jj_lib::repo_path::RepoPathUiConverter;
use jj_lib::transaction::Transaction;
use jj_lib::workspace::Workspace;

use config::{default_settings, working_copy_factories};
use support::{block_on_result, load_repo_at_head, load_workspace_internal, op_is_ancestor_of};

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

    fn replace_repo(&self, repo: Arc<ReadonlyRepo>) {
        *self.repo.write().unwrap() = repo;
    }

    /// Installs `repo` unless it would roll the in-memory view backwards.
    /// Concurrent mutations/refreshes each `load -> work -> set_repo`, so a slow
    /// loser can arrive with a stale or divergent op; keep the newer state and
    /// reconcile from disk (which merges concurrent op heads) instead of clobbering it.
    pub(crate) fn set_repo(&self, repo: Arc<ReadonlyRepo>) {
        let mut current = self.repo.write().unwrap();
        let candidate_is_current_or_newer =
            op_is_ancestor_of(&repo, current.op_id()).unwrap_or(true);
        if candidate_is_current_or_newer {
            *current = repo;
            return;
        }
        drop(current);
        // Candidate is stale or divergent: reload the merged on-disk head.
        if let Err(error) = self.replace_with_loaded_head() {
            // Reload failed; fall back to the candidate rather than block writes.
            self.replace_repo(repo);
            debug_assert!(false, "set_repo reconcile failed: {error}");
        }
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
        self.replace_with_loaded_head()
    }

    fn replace_with_loaded_head(&self) -> CoreResult<()> {
        let workspace = load_workspace_internal(&self.path, "reload workspace")?;
        let repo = load_repo_at_head(&workspace, "reload repo")?;
        self.replace_repo(repo);
        Ok(())
    }

    pub(crate) fn commit_transaction(&self, tx: Transaction, description: &str) -> CoreResult<()> {
        let old_working_copy_commit_id = self.current_wc_commit_id();
        let new_repo = self.commit_transaction_to_repo(tx, description)?;
        self.set_repo(new_repo);
        // If @ changed, sync the working directory on disk
        if self.current_wc_commit_id() != old_working_copy_commit_id {
            self.check_out_current_working_copy("sync working copy after transaction")?;
        }
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

    fn current_wc_commit_id(&self) -> Option<String> {
        use jj_lib::object_id::ObjectId;
        let repo = self.get_repo();
        repo.view()
            .get_wc_commit_id(self.workspace_name.as_ref())
            .map(|id| id.hex())
    }

    pub(crate) fn commit_transaction_to_repo(
        &self,
        tx: Transaction,
        description: &str,
    ) -> CoreResult<Arc<ReadonlyRepo>> {
        block_on_result("commit tx", tx.commit(description))
    }
}

#[cfg(test)]
mod tests {
    use jj_lib::object_id::ObjectId as _;
    use jj_test::init_jj_repo;

    use super::Repo;

    fn current_op(repo: &Repo) -> String {
        repo.get_repo().op_id().hex()
    }

    fn description_of_at(repo: &Repo) -> String {
        repo.log("@")
            .expect("log @")
            .into_iter()
            .next()
            .expect("at least one change")
            .description
    }

    #[test]
    fn set_repo_rejects_stale_operation() {
        let temp_dir = init_jj_repo();
        let repo_path = temp_dir.path().join("repo");
        let repo = Repo::open(&repo_path).expect("open repo");

        // Stand-in for a repo a concurrent refresh loaded before the mutation.
        let stale = repo.get_repo();
        let stale_op = current_op(&repo);

        repo.describe("@", "newer in-memory state")
            .expect("describe advances state");
        let newer_op = current_op(&repo);
        assert_ne!(stale_op, newer_op, "describe should advance the operation");

        // The losing refresh installs its pre-mutation repo last.
        repo.set_repo(stale);

        assert_ne!(
            current_op(&repo),
            stale_op,
            "stale operation must not overwrite newer in-memory state"
        );
        assert_eq!(
            description_of_at(&repo),
            "newer in-memory state",
            "newer description must survive a stale set_repo"
        );
    }

    #[test]
    fn reload_replaces_divergent_in_memory_repo() {
        let temp_dir = init_jj_repo();
        let repo_path = temp_dir.path().join("repo");
        let repo = Repo::open(&repo_path).expect("open repo");
        let expected_op = current_op(&repo);

        let other_temp_dir = init_jj_repo();
        let other_path = other_temp_dir.path().join("repo");
        let other_repo = Repo::open(&other_path).expect("open other repo");
        other_repo
            .describe("@", "unrelated operation")
            .expect("advance other repo");

        *repo.repo.write().unwrap() = other_repo.get_repo();

        repo.reload().expect("reload replaces divergent repo");

        assert_eq!(current_op(&repo), expected_op);
    }

    #[test]
    fn op_is_ancestor_of_orders_operations() {
        use super::support::op_is_ancestor_of;

        let temp_dir = init_jj_repo();
        let repo_path = temp_dir.path().join("repo");
        let repo = Repo::open(&repo_path).expect("open repo");

        let base = repo.get_repo();
        repo.describe("@", "forward progress")
            .expect("describe advances state");
        let forward = repo.get_repo();

        assert!(
            op_is_ancestor_of(&forward, base.op_id()).expect("walk ancestors"),
            "the new head must be recognized as a descendant of the base op"
        );
        assert!(
            !op_is_ancestor_of(&base, forward.op_id()).expect("walk ancestors"),
            "the base op must not be recognized as a descendant of the new head"
        );
        assert!(
            op_is_ancestor_of(&forward, forward.op_id()).expect("walk ancestors"),
            "an op is its own ancestor for install purposes"
        );
    }
}
