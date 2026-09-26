use std::path::{Path, PathBuf};

use jj_lib::local_working_copy::LocalWorkingCopyFactory;
use jj_lib::ref_name::{WorkspaceName, WorkspaceNameBuf};
use jj_lib::rewrite::merge_commit_trees;
use jj_lib::workspace::Workspace;
use jj_lib::workspace_store::{SimpleWorkspaceStore, WorkspaceStore as _};

use super::super::Repo;
use super::super::support::{block_on_result, load_workspace_internal};
use super::super::workspace_path::is_valid_workspace_name;
use super::listing::existing_dir;
use crate::types::*;

impl Repo {
    /// `jj workspace add`: a new checkout of this repo at `dest` whose working copy starts on `rev`, or on the current working copy's parents when `rev` is empty.
    pub fn workspace_add(&self, dest: &str, name: &str, rev: &str) -> CoreResult<String> {
        let _write = self.write_guard()?;
        if !name.is_empty() && !is_valid_workspace_name(name) {
            return Err(CoreError::Internal {
                message: format!("invalid workspace name: {name}"),
            });
        }
        let dest = self.path.join(dest);
        let name = if name.is_empty() {
            dest.file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| CoreError::internal("workspace destination has no name"))?
        } else {
            name
        };
        let workspace_name = WorkspaceNameBuf::from(name);

        self.refresh_working_copy()?;
        let repo = self.get_repo();
        if repo.view().get_wc_commit_id(&workspace_name).is_some() {
            return Err(CoreError::internal(format!(
                "workspace named '{name}' already exists"
            )));
        }
        let parents = if rev.is_empty() {
            block_on_result("load parents", self.working_copy_commit(&repo)?.parents())?
        } else {
            vec![self.resolve_commit(&repo, rev)?]
        };
        if !dest.exists() {
            std::fs::create_dir(&dest).map_err(|error| {
                CoreError::internal(format!("create {}: {error}", dest.display()))
            })?;
        } else if dest
            .read_dir()
            .map_or(true, |mut entries| entries.next().is_some())
        {
            return Err(CoreError::internal(
                "destination path exists and is not an empty directory",
            ));
        }

        let (mut workspace, repo) = block_on_result(
            "create workspace",
            Workspace::init_workspace_with_existing_repo(
                &dest,
                &self.repo_path,
                &repo,
                &LocalWorkingCopyFactory {},
                workspace_name.clone(),
            ),
        )?;
        let sparse_patterns = load_workspace_internal(&self.path, "read sparse patterns")?
            .working_copy()
            .sparse_patterns()
            .map_err(CoreError::internal)?
            .to_vec();
        let mut tx = repo.start_transaction();
        let tree = block_on_result(
            "merge parent trees",
            merge_commit_trees(tx.repo(), &parents),
        )?;
        let parent_ids = parents.iter().map(|parent| parent.id().clone()).collect();
        let wc_commit = block_on_result(
            "create working-copy commit",
            tx.repo_mut().new_commit(parent_ids, tree).write(),
        )?;
        block_on_result(
            "edit working copy",
            tx.repo_mut().edit(workspace_name.clone(), &wc_commit),
        )?;
        block_on_result("rebase descendants", tx.repo_mut().rebase_descendants())?;
        let new_repo = block_on_result(
            "commit workspace",
            tx.commit(format!(
                "create initial working-copy commit in workspace {}",
                workspace_name.as_symbol()
            )),
        )?;
        let context = "check out new workspace";
        let mut locked_ws = block_on_result(context, workspace.start_working_copy_mutation())?;
        block_on_result(
            context,
            locked_ws.locked_wc().set_sparse_patterns(sparse_patterns),
        )?;
        block_on_result(context, locked_ws.locked_wc().check_out(&wc_commit))?;
        block_on_result(context, locked_ws.finish(new_repo.op_id().clone()))?;
        self.set_repo(new_repo);
        Ok(format!("Created workspace in \"{}\"", dest.display()))
    }

    /// `expected_root` prevents a stale workspace row from forgetting a replacement with the same name.
    pub fn workspace_forget(&self, name: &str, expected_root: Option<&str>) -> CoreResult<()> {
        let _write = self.write_guard()?;
        self.ensure_workspace_is_not_current(name)?;
        if let Some(expected_root) = expected_root {
            self.verify_workspace_root(name, expected_root)?;
        }
        self.forget_workspace_name(name)
    }

    pub(super) fn ensure_workspace_is_not_current(&self, name: &str) -> CoreResult<()> {
        if name == self.workspace_name.as_str() {
            return Err(CoreError::Internal {
                message: "cannot forget the current workspace".to_owned(),
            });
        }
        Ok(())
    }

    /// `jj workspace forget`: drop the workspace's working-copy commit from the view and its saved root, leaving its files alone.
    pub(super) fn forget_workspace_name(&self, name: &str) -> CoreResult<()> {
        let workspace_name = WorkspaceName::new(name);
        self.refresh_working_copy()?;
        if self
            .get_repo()
            .view()
            .get_wc_commit_id(workspace_name)
            .is_none()
        {
            return Err(CoreError::internal(format!("no such workspace: {name}")));
        }
        // The saved root goes first, as in the CLI: a workspace left in the view without a root is a state the app already handles, the reverse is not.
        SimpleWorkspaceStore::load(&self.repo_path)
            .and_then(|store| store.forget(&[workspace_name]))
            .map_err(|error| CoreError::internal(format!("forget workspace root: {error}")))?;
        self.with_repo_transaction(
            &format!("forget workspace {}", workspace_name.as_symbol()),
            true,
            |_, repo_mut| {
                block_on_result(
                    "forget workspace",
                    repo_mut.remove_workspace(workspace_name),
                )
            },
        )
    }

    pub(super) fn verify_workspace_root(
        &self,
        name: &str,
        expected_root: &str,
    ) -> CoreResult<PathBuf> {
        let mismatch = |why: &str| CoreError::Internal {
            message: format!("workspace {name} at {expected_root} {why}; refresh and try again"),
        };
        let expected =
            existing_dir(Path::new(expected_root)).ok_or_else(|| mismatch("is not a directory"))?;
        if let Some(recorded) = self.recorded_workspace_root(WorkspaceName::new(name))
            && existing_dir(&recorded).as_ref() != Some(&expected)
        {
            return Err(mismatch("moved"));
        }
        self.verify_workspace_checkout(name, &expected)
            .map_err(|error| mismatch(&format!("is not a jj workspace: {error}")))?;
        Ok(expected)
    }

    pub(super) fn verify_workspace_checkout(&self, name: &str, root: &Path) -> CoreResult<()> {
        let target = load_workspace_internal(root, "verify workspace root")?;
        let same_repo =
            dunce::canonicalize(target.repo_path()).ok().as_ref() == Some(&self.repo_path);
        if target.workspace_name().as_str() != name || !same_repo {
            return Err(CoreError::internal(format!(
                "workspace {name} at {} no longer belongs to this repository",
                root.display()
            )));
        }
        Ok(())
    }
}
