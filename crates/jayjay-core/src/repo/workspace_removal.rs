use std::path::{Component, Path, PathBuf};

use jj_lib::object_id::ObjectId;
use jj_lib::workspace::Workspace;
use jj_lib::workspace_store::{SimpleWorkspaceStore, WorkspaceStore as _};
use pollster::FutureExt as _;

use super::Repo;
use super::support::{load_repo_at_head, load_workspace_internal};
use crate::types::{CoreError, CoreResult};

impl Repo {
    /// Validate that `name` still identifies the checkout at `expected_root`, returning the exact operation the destructive flow must consume.
    pub fn workspace_removal_guard(
        &self,
        name: &str,
        expected_root: &str,
        expected_operation: &str,
    ) -> CoreResult<String> {
        if name == self.workspace_name.as_str() {
            return Err(CoreError::Internal {
                message: "cannot remove the current workspace".to_owned(),
            });
        }
        let context = "prepare workspace removal";
        let workspace = load_workspace_internal(&self.path, context)?;
        let repo = load_repo_at_head(&workspace, context)?;
        if repo.op_id().hex() != expected_operation {
            return Err(CoreError::Internal {
                message: format!(
                    "workspace {name} changed after it was listed; refresh and try again"
                ),
            });
        }
        let workspace_name = repo
            .view()
            .wc_commit_ids()
            .keys()
            .find(|workspace_name| workspace_name.as_str() == name)
            .ok_or_else(|| CoreError::Internal {
                message: format!("workspace {name} no longer exists"),
            })?;
        let expected_root =
            self.verified_workspace_root(&workspace, workspace_name, expected_root)?;
        let target = load_workspace_internal(&expected_root, context)?;
        let target_repo_path =
            std::fs::canonicalize(target.repo_path()).map_err(|error| CoreError::Internal {
                message: format!("canonicalize target workspace repository: {error}"),
            })?;
        let source_repo_path =
            std::fs::canonicalize(workspace.repo_path()).map_err(|error| CoreError::Internal {
                message: format!("canonicalize source workspace repository: {error}"),
            })?;
        if target.workspace_name() != workspace_name || target_repo_path != source_repo_path {
            return Err(CoreError::Internal {
                message: format!(
                    "workspace {name} at {} no longer belongs to this repository",
                    expected_root.display()
                ),
            });
        }
        Ok(repo.op_id().hex())
    }

    /// Remove exactly the workspace generation previously returned by `workspace_removal_guard`; operation-head locking makes the identity check and publication atomic with other jj commands.
    pub fn workspace_forget(
        &self,
        name: &str,
        expected_root: &str,
        expected_operation: &str,
    ) -> CoreResult<Option<String>> {
        self.workspace_forget_internal(name, Some(expected_root), expected_operation)
    }

    /// Forget a listed workspace whose checkout root could not be resolved. This recovery is name-only and never authorizes deleting files; the exact operation still locks the workspace generation being removed.
    pub fn workspace_forget_unresolved(
        &self,
        name: &str,
        expected_operation: &str,
    ) -> CoreResult<Option<String>> {
        if name == self.workspace_name.as_str() {
            return Err(CoreError::Internal {
                message: "cannot remove the current workspace".to_owned(),
            });
        }
        if self.workspace_path(name, false, None).is_ok() {
            return Err(CoreError::Internal {
                message: format!("workspace {name} root became available; refresh and try again"),
            });
        }
        self.workspace_forget_internal(name, None, expected_operation)
    }

    fn workspace_forget_internal(
        &self,
        name: &str,
        expected_root: Option<&str>,
        expected_operation: &str,
    ) -> CoreResult<Option<String>> {
        let context = "forget workspace";
        let workspace = load_workspace_internal(&self.path, context)?;
        let repo = load_repo_at_head(&workspace, context)?;
        let workspace_name = repo
            .view()
            .wc_commit_ids()
            .keys()
            .find(|workspace_name| workspace_name.as_str() == name)
            .map(ToOwned::to_owned)
            .ok_or_else(|| CoreError::Internal {
                message: format!("workspace {name} no longer exists"),
            })?;
        let op_heads_store = repo.loader().op_heads_store().clone();
        let op_heads_lock =
            op_heads_store
                .lock()
                .block_on()
                .map_err(|error| CoreError::Internal {
                    message: format!("lock operation heads before forgetting workspace: {error}"),
                })?;
        let head_ids =
            op_heads_store
                .get_op_heads()
                .block_on()
                .map_err(|error| CoreError::Internal {
                    message: format!("read operation heads before forgetting workspace: {error}"),
                })?;
        if repo.op_id().hex() != expected_operation
            || head_ids.as_slice() != std::slice::from_ref(repo.op_id())
        {
            return Err(CoreError::Internal {
                message: format!(
                    "workspace {name} changed after confirmation; refresh and try again"
                ),
            });
        }
        if let Some(expected_root) = expected_root
            && !self.recorded_workspace_root_matches(&workspace, &workspace_name, expected_root)?
        {
            return Err(CoreError::Internal {
                message: format!(
                    "workspace {name} moved after confirmation; refresh and try again"
                ),
            });
        }

        let old_working_copy_commit_id = self.current_wc_commit_id();
        let mut transaction = repo.start_transaction();
        transaction.set_workspace_name(self.workspace_name.as_ref());
        transaction
            .repo_mut()
            .remove_wc_commit(&workspace_name)
            .block_on()
            .map_err(|error| CoreError::Internal {
                message: format!("remove workspace {name}: {error}"),
            })?;
        transaction
            .repo_mut()
            .rebase_descendants()
            .block_on()
            .map_err(|error| CoreError::Internal {
                message: format!("rebase after forgetting workspace {name}: {error}"),
            })?;
        let unpublished = transaction
            .write(format!("forget workspace {name}"))
            .block_on()
            .map_err(|error| CoreError::Internal {
                message: format!("write forget workspace operation: {error}"),
            })?;
        let operation = unpublished.operation().clone();
        let updated_repo = unpublished.leave_unpublished();
        let workspace_store =
            SimpleWorkspaceStore::load(workspace.repo_path()).map_err(|error| {
                CoreError::Internal {
                    message: format!("load workspace store before forgetting: {error}"),
                }
            })?;
        op_heads_store
            .update_op_heads(operation.parent_ids(), operation.id())
            .block_on()
            .map_err(|error| CoreError::Internal {
                message: format!("publish forget workspace operation: {error}"),
            })?;
        let workspace_store_warning = workspace_store
            .forget(&[workspace_name.as_ref()])
            .err()
            .map(|error| {
                format!(
                    "Workspace {name} was forgotten, but its saved checkout path could not be removed: {error}"
                )
            });
        drop(op_heads_lock);
        self.set_repo(updated_repo);
        if self.current_wc_commit_id() != old_working_copy_commit_id
            && let Err(error) =
                self.check_out_current_working_copy("sync working copy after forgetting workspace")
        {
            let checkout_warning = format!(
                "Workspace {name} was forgotten, but the current working copy could not be synchronized: {error}"
            );
            return Ok(Some(match workspace_store_warning {
                Some(workspace_store_warning) => {
                    format!("{workspace_store_warning}\n{checkout_warning}")
                }
                None => checkout_warning,
            }));
        }
        Ok(workspace_store_warning)
    }

    fn verified_workspace_root(
        &self,
        workspace: &Workspace,
        workspace_name: &jj_lib::ref_name::WorkspaceName,
        expected_root: &str,
    ) -> CoreResult<PathBuf> {
        let expected_root =
            std::fs::canonicalize(expected_root).map_err(|error| CoreError::Internal {
                message: format!("canonicalize expected workspace root: {error}"),
            })?;
        let recorded_root = SimpleWorkspaceStore::load(workspace.repo_path())
            .and_then(|store| store.get_workspace_path(workspace_name))
            .map_err(|error| CoreError::Internal {
                message: format!("read workspace root: {error}"),
            })?;
        // Older repositories have no saved roots; the caller still proves ownership by loading expected_root and matching both its workspace name and repository below.
        let Some(recorded_root) = recorded_root else {
            return Ok(expected_root);
        };
        let recorded_root = std::fs::canonicalize(workspace.repo_path().join(recorded_root))
            .map_err(|error| CoreError::Internal {
                message: format!("canonicalize recorded workspace root: {error}"),
            })?;
        if recorded_root != expected_root {
            return Err(CoreError::Internal {
                message: format!(
                    "workspace {} moved from {} to {}",
                    workspace_name.as_str(),
                    expected_root.display(),
                    recorded_root.display()
                ),
            });
        }
        Ok(expected_root)
    }

    fn recorded_workspace_root_matches(
        &self,
        workspace: &Workspace,
        workspace_name: &jj_lib::ref_name::WorkspaceName,
        expected_root: &str,
    ) -> CoreResult<bool> {
        let Some(expected_root) = normalized_absolute_path(Path::new(expected_root)) else {
            return Ok(false);
        };
        let recorded_root = SimpleWorkspaceStore::load(workspace.repo_path())
            .and_then(|store| store.get_workspace_path(workspace_name))
            .map_err(|error| CoreError::Internal {
                message: format!("read workspace root before forgetting: {error}"),
            })?;
        // A missing legacy entry was already validated against the live checkout by workspace_removal_guard; the operation-head check above proves that validation is still current after quarantine.
        let Some(recorded_root) = recorded_root else {
            return Ok(true);
        };
        Ok(
            normalized_absolute_path(&workspace.repo_path().join(recorded_root))
                .is_some_and(|path| path == expected_root),
        )
    }
}

pub(super) fn normalized_absolute_path(path: &Path) -> Option<PathBuf> {
    // Windows canonicalization adds a verbatim prefix that JJ's workspace store does not preserve.
    let path = dunce::simplified(path);
    if !path.is_absolute() {
        return None;
    }
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return None;
                }
            }
            Component::Normal(part) => normalized.push(part),
        }
    }
    Some(normalized)
}

#[cfg(all(test, windows))]
mod tests {
    use super::normalized_absolute_path;
    use std::path::Path;

    #[test]
    fn windows_verbatim_and_standard_paths_have_the_same_identity() {
        assert_eq!(
            normalized_absolute_path(Path::new(r"\\?\C:\repo\feature")),
            normalized_absolute_path(Path::new(r"C:\repo\feature"))
        );
    }
}
