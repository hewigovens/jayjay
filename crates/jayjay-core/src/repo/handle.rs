use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};

use jj_lib::backend::CommitId;
use jj_lib::repo::ReadonlyRepo;
use jj_lib::repo_path::RepoPathBuf;
use jj_lib::transaction::Transaction;
use jj_lib::ui_path::RepoPathUiConverter;

use super::cache::RepoCache;
use super::command_process::RunningJjProcesses;
use super::git::lfs::LfsCache;
use super::log;
use super::resolve::{CommitEmptiness, CommitRefNames};
use super::support::{
    block_on_result, canonicalize, load_repo_at_head, load_workspace, load_workspace_internal,
    op_is_ancestor_of,
};
use super::write_lock;

use crate::types::*;

pub struct Repo {
    pub(super) path: PathBuf,
    pub(super) repo_path: PathBuf,
    pub(super) workspace_name: jj_lib::ref_name::WorkspaceNameBuf,
    repo: RwLock<Arc<ReadonlyRepo>>,
    pub(super) running_jj_processes: RunningJjProcesses,
    pub(super) immutable_ids_cache: RepoCache<log::ImmutableIds>,
    pub(super) commit_tags_cache: RepoCache<CommitRefNames>,
    pub(super) commit_bookmarks_cache: RepoCache<CommitRefNames>,
    pub(super) commit_emptiness: CommitEmptiness,
    /// A changed-file count costs a full parent-tree diff; re-diff only workspaces whose working-copy commit moved.
    pub(super) workspace_files_changed_cache: RwLock<HashMap<String, (CommitId, u32)>>,
    pub(super) lfs_cache: Mutex<LfsCache>,
    pub(super) write_lock: Arc<parking_lot::ReentrantMutex<()>>,
}

impl Repo {
    pub fn open(path: &Path) -> CoreResult<Self> {
        let workspace = load_workspace(path).map_err(|error| {
            if path.join(".jj").is_dir() {
                CoreError::Internal {
                    message: format!("failed to load repo: {error}"),
                }
            } else {
                CoreError::RepoNotFound {
                    path: path.display().to_string(),
                }
            }
        })?;

        let repo = load_repo_at_head(&workspace, "failed to load repo")?;

        let repo_path = canonicalize(workspace.repo_path());
        Ok(Self {
            path: workspace.workspace_root().to_owned(),
            write_lock: write_lock::for_store(&repo_path),
            repo_path,
            workspace_name: workspace.workspace_name().to_owned(),
            repo: RwLock::new(repo),
            running_jj_processes: RunningJjProcesses::default(),
            immutable_ids_cache: RepoCache::default(),
            commit_tags_cache: RepoCache::default(),
            commit_bookmarks_cache: RepoCache::default(),
            commit_emptiness: CommitEmptiness::default(),
            workspace_files_changed_cache: RwLock::new(HashMap::new()),
            lfs_cache: Mutex::new(LfsCache::default()),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn repository_store_path(&self) -> &Path {
        &self.repo_path
    }

    pub fn workspace_name(&self) -> &str {
        self.workspace_name.as_str()
    }

    pub(super) fn path_converter(&self) -> RepoPathUiConverter {
        RepoPathUiConverter::Fs {
            cwd: self.path.clone(),
            base: self.path.clone(),
        }
    }

    pub(super) fn get_repo(&self) -> Arc<ReadonlyRepo> {
        self.repo.read().unwrap().clone()
    }

    fn replace_repo(&self, repo: Arc<ReadonlyRepo>) {
        *self.repo.write().unwrap() = repo;
    }

    /// A slow concurrent writer can arrive with a stale or divergent op; keep the newer state and reconcile from disk.
    pub(super) fn set_repo(&self, repo: Arc<ReadonlyRepo>) {
        let mut current = self.repo.write().unwrap();
        let candidate_is_current_or_newer =
            op_is_ancestor_of(&repo, current.op_id()).unwrap_or(true);
        if candidate_is_current_or_newer {
            *current = repo;
            return;
        }
        drop(current);
        if let Err(error) = self.replace_with_loaded_head() {
            // Reload failed; fall back to the candidate rather than block writes.
            self.replace_repo(repo);
            debug_assert!(false, "set_repo reconcile failed: {error}");
        }
    }

    pub(super) fn parse_repo_path(&self, path: &str) -> CoreResult<RepoPathBuf> {
        jj_lib::ui_path::parse_fs_path(&self.path, &self.path, path).map_err(|e| {
            CoreError::Internal {
                message: format!("invalid path {path}: {e}"),
            }
        })
    }

    pub(super) fn parse_repo_paths(&self, paths: &[String]) -> CoreResult<Vec<RepoPathBuf>> {
        paths
            .iter()
            .map(|path| self.parse_repo_path(path))
            .collect()
    }

    pub(super) fn reload(&self) -> CoreResult<()> {
        self.replace_with_loaded_head()
    }

    fn replace_with_loaded_head(&self) -> CoreResult<()> {
        let workspace = load_workspace_internal(&self.path, "reload workspace")?;
        let repo = load_repo_at_head(&workspace, "reload repo")?;
        self.replace_repo(repo);
        Ok(())
    }

    pub(super) fn commit_transaction_rebase(
        &self,
        mut tx: Transaction,
        description: &str,
    ) -> CoreResult<()> {
        block_on_result("rebase descendants", tx.repo_mut().rebase_descendants())?;
        self.commit_transaction(tx, description)
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

        let stale = repo.get_repo();
        let stale_op = current_op(&repo);

        repo.describe("@", "newer in-memory state")
            .expect("describe advances state");
        let newer_op = current_op(&repo);
        assert_ne!(stale_op, newer_op, "describe should advance the operation");

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
        use super::super::support::op_is_ancestor_of;

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
