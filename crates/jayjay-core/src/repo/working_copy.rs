use std::sync::Arc;
#[cfg(test)]
use std::sync::Mutex;

use jj_lib::commit::Commit;
use jj_lib::matchers::{EverythingMatcher, NothingMatcher};
use jj_lib::repo::{ReadonlyRepo, Repo as _};
use jj_lib::working_copy::SnapshotOptions;
use jj_lib::workspace::LockedWorkspace;

use super::Repo;
use super::support::{block_on_result, load_repo_at_head, load_workspace_internal};
use super::working_copy_ignore::{WorkingCopyIgnoreMatcher, base_git_ignores};
use crate::types::*;

const WORKING_COPY_REFRESH_STACK_SIZE: usize = 8 * 1024 * 1024;

#[cfg(test)]
type LockHook = Box<dyn Fn(&std::path::Path) + Send + Sync>;
#[cfg(test)]
static BEFORE_WORKING_COPY_LOCK: Mutex<Option<LockHook>> = Mutex::new(None);

impl Repo {
    pub(crate) fn working_copy_commit(&self, repo: &ReadonlyRepo) -> CoreResult<Commit> {
        let commit_id = repo
            .view()
            .get_wc_commit_id(self.workspace_name.as_ref())
            .ok_or_else(|| CoreError::internal("workspace has no working-copy commit"))?;
        repo.store()
            .get_commit(commit_id)
            .map_err(|error| CoreError::internal(format!("load working-copy commit: {error}")))
    }

    pub(crate) fn check_out_current_working_copy(&self, context: &str) -> CoreResult<()> {
        let mut workspace = load_workspace_internal(&self.path, context)?;
        let repo = load_repo_at_head(&workspace, context)?;
        let wc_commit_id = repo
            .view()
            .get_wc_commit_id(self.workspace_name.as_ref())
            .ok_or_else(|| CoreError::Internal {
                message: format!(
                    "workspace {} has no working-copy commit",
                    self.workspace_name.as_symbol()
                ),
            })?
            .clone();
        let wc_commit =
            repo.store()
                .get_commit(&wc_commit_id)
                .map_err(|e| CoreError::Internal {
                    message: format!("load working-copy commit: {e}"),
                })?;
        block_on_result(
            context,
            workspace.check_out(repo.op_id().clone(), None, &wc_commit),
        )?;
        self.set_repo(repo);
        Ok(())
    }

    pub fn refresh_working_copy(&self) -> CoreResult<()> {
        // Swift cooperative executor threads have small stacks; jj descendant rebases can need substantially more while polling tree merges.
        std::thread::scope(|scope| {
            let worker = std::thread::Builder::new()
                .name("jayjay-wc-refresh".to_owned())
                .stack_size(WORKING_COPY_REFRESH_STACK_SIZE)
                .spawn_scoped(scope, || self.refresh_working_copy_inner())
                .map_err(|error| CoreError::Internal {
                    message: format!("start working-copy refresh: {error}"),
                })?;
            match worker.join() {
                Ok(result) => result,
                Err(payload) => std::panic::resume_unwind(payload),
            }
        })
    }

    fn refresh_working_copy_inner(&self) -> CoreResult<()> {
        let mut workspace = load_workspace_internal(&self.path, "load workspace for snapshot")?;
        let repo_loader = workspace.repo_loader().clone();
        #[cfg(test)]
        if let Some(hook) = BEFORE_WORKING_COPY_LOCK.lock().unwrap().as_ref() {
            hook(&self.path);
        }
        // Lock before loading the head: a snapshot that lands while we wait would otherwise be rewritten from a stale head, forking `@`.
        let locked_ws =
            block_on_result("lock working copy", workspace.start_working_copy_mutation())?;
        let repo = block_on_result("load repo for snapshot", repo_loader.load_at_head())?;
        self.snapshot_locked_working_copy(locked_ws, repo)
    }

    fn snapshot_locked_working_copy(
        &self,
        mut locked_ws: LockedWorkspace<'_>,
        repo: Arc<ReadonlyRepo>,
    ) -> CoreResult<()> {
        let wc_commit = self.working_copy_commit(&repo)?;

        let snapshot_options = SnapshotOptions {
            base_ignores: base_git_ignores(&repo, &self.path)?,
            progress: None,
            start_tracking_matcher: &EverythingMatcher,
            force_tracking_matcher: &NothingMatcher,
            max_new_file_size: u64::MAX,
        };

        let snapshot = locked_ws.locked_wc().snapshot(&snapshot_options);
        let (new_tree, _) = block_on_result("snapshot working copy", snapshot)?;

        if new_tree.tree_ids_and_labels() != wc_commit.tree().tree_ids_and_labels() {
            let mut tx = repo.start_transaction();
            tx.set_is_snapshot(true);
            self.rewrite_commit_tree(
                tx.repo_mut(),
                &wc_commit,
                new_tree,
                "rewrite working-copy commit",
            )?;
            let rebase = tx.repo_mut().rebase_descendants();
            block_on_result("rebase descendants after snapshot", rebase)?;
            let commit = tx.commit("snapshot working copy");
            let new_repo = block_on_result("commit snapshot operation", commit)?;
            block_on_result(
                "finish working-copy snapshot",
                locked_ws.finish(new_repo.op_id().clone()),
            )?;
            self.set_repo(new_repo);
        } else {
            block_on_result(
                "finish clean working-copy snapshot",
                locked_ws.finish(repo.op_id().clone()),
            )?;
            self.set_repo(repo);
        }
        Ok(())
    }

    pub fn has_unignored_working_copy_paths(&self, paths: &[String]) -> CoreResult<bool> {
        let repo = self.get_repo();
        WorkingCopyIgnoreMatcher::new(&repo, self.workspace_name.as_ref(), &self.path)?
            .has_unignored_paths(paths)
    }

    /// Proxy for snapshot cost: `tree_state` size scales with file count without walking the tree.
    pub fn working_copy_is_large(&self) -> bool {
        const LARGE_TREE_STATE_BYTES: u64 = 8 * 1024 * 1024;
        self.path
            .join(".jj/working_copy/tree_state")
            .metadata()
            .map(|m| m.len() > LARGE_TREE_STATE_BYTES)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    use jj_test::init_jj_repo;

    use super::super::Repo;
    use super::super::support::{block_on_result, load_workspace_internal};
    use super::BEFORE_WORKING_COPY_LOCK;

    #[test]
    fn refresh_keeps_a_snapshot_that_lands_while_it_waits_for_the_lock() {
        let temp_dir = init_jj_repo();
        let repo_path = temp_dir.path().join("repo");
        let repo = Repo::open(&repo_path).expect("open repo");
        let mut workspace =
            load_workspace_internal(&repo_path, "load workspace").expect("load workspace");
        let repo_loader = workspace.repo_loader().clone();
        let locked_ws = block_on_result("lock", workspace.start_working_copy_mutation())
            .expect("hold the working-copy lock");
        fs::write(repo_path.join("hello.txt"), "edited under the lock\n").expect("edit file");
        let (reached_lock_tx, reached_lock_rx) = mpsc::channel();
        let refreshing_path = repo.path().to_owned();
        *BEFORE_WORKING_COPY_LOCK.lock().unwrap() = Some(Box::new(move |path| {
            if path == refreshing_path {
                let _ = reached_lock_tx.send(());
            }
        }));

        thread::scope(|scope| {
            let refresh = scope.spawn(|| repo.refresh_working_copy());
            reached_lock_rx
                .recv_timeout(Duration::from_secs(10))
                .expect("refresh reaches the working-copy lock");
            let head = block_on_result("load head", repo_loader.load_at_head()).expect("load head");
            repo.snapshot_locked_working_copy(locked_ws, head)
                .expect("snapshot under the held lock");
            refresh
                .join()
                .expect("refresh thread")
                .expect("refresh once the lock is released");
        });

        *BEFORE_WORKING_COPY_LOCK.lock().unwrap() = None;

        // A refresh that read the head before locking commits a second op head; loading then merges them into an op with two parents.
        repo.reload().expect("reload");
        assert_eq!(
            repo.get_repo().operation().parent_ids().len(),
            1,
            "refresh committed against a stale head"
        );
    }
}
