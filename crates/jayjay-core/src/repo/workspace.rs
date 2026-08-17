use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use futures::StreamExt as _;
use jj_lib::hex_util::encode_reverse_hex;
use jj_lib::matchers::EverythingMatcher;
use jj_lib::object_id::ObjectId;
use jj_lib::op_store::OperationId;
use jj_lib::repo::{ReadonlyRepo, Repo as _};
use jj_lib::workspace::Workspace;
use jj_lib::workspace_store::{SimpleWorkspaceStore, WorkspaceStore as _};
use pollster::FutureExt as _;

use super::Repo;
use super::support::{load_repo_at_head, load_workspace_internal, op_is_ancestor_of};
use super::workspace_path::is_valid_workspace_name;
use super::workspace_removal::normalized_absolute_path;
use crate::types::*;

const IGNORE_WORKING_COPY_ARG: &str = "--ignore-working-copy";
const WORKSPACE_COMMAND: &str = "workspace";
const WORKSPACE_LIST_MAX_ATTEMPTS: usize = 3;

/// Workspace roots cost a jj subprocess each, so they are cached, but only for the operation they were resolved at: every add and forget, including one made by another process, lands as a new operation, so a name recreated at a different root cannot outlive its cached path.
#[derive(Default)]
pub(super) struct WorkspacePathCache {
    op_id: Option<OperationId>,
    paths: HashMap<String, String>,
}

impl WorkspacePathCache {
    /// Advances the cache only when `repo` is at least as new as its current generation. An older overlapping list call must not roll a newer generation backwards.
    fn sync_to(&mut self, repo: &Arc<ReadonlyRepo>) -> CoreResult<bool> {
        if self.op_id.as_ref() == Some(repo.op_id()) {
            return Ok(true);
        }
        if let Some(op_id) = self.op_id.as_ref()
            && !op_is_ancestor_of(repo, op_id)?
        {
            return Ok(false);
        }
        self.op_id = Some(repo.op_id().clone());
        self.paths.clear();
        Ok(true)
    }

    fn path(&self, op_id: &OperationId, name: &str) -> Option<&String> {
        (self.op_id.as_ref() == Some(op_id))
            .then(|| self.paths.get(name))
            .flatten()
    }

    /// A resolver can finish after another call advances the cache, so insertion is conditional on the operation captured before resolution began.
    fn insert_if_current(&mut self, op_id: &OperationId, name: String, path: String) -> bool {
        if self.op_id.as_ref() != Some(op_id) {
            return false;
        }
        self.paths.insert(name, path);
        true
    }
}

impl Repo {
    /// List all workspaces with the status of each one's committed `@`, retrying from a newer operation when roots and the repository head change concurrently; loading repository views never snapshots another working copy.
    pub fn workspace_list(&self) -> CoreResult<Vec<WorkspaceInfo>> {
        let mut repo = self.get_repo();
        for _ in 0..WORKSPACE_LIST_MAX_ATTEMPTS {
            let (workspaces, newer_repo) = self.workspace_list_at(repo)?;
            let Some(newer_repo) = newer_repo else {
                return Ok(workspaces);
            };
            repo = newer_repo;
        }
        Err(CoreError::Internal {
            message:
                "workspace list kept changing while roots were resolving; refresh and try again"
                    .to_owned(),
        })
    }

    fn workspace_list_at(
        &self,
        repo: Arc<ReadonlyRepo>,
    ) -> CoreResult<(Vec<WorkspaceInfo>, Option<Arc<ReadonlyRepo>>)> {
        let operation_id = repo.op_id().hex();
        let cache_op_id = self
            .workspace_paths
            .write()
            .unwrap()
            .sync_to(&repo)?
            .then(|| repo.op_id().clone());
        let mut workspaces = Vec::new();

        for (name, commit_id) in repo.view().wc_commit_ids() {
            let name = name.as_str().to_owned();
            let Ok(commit) = repo.store().get_commit(commit_id) else {
                continue;
            };
            let change_id = encode_reverse_hex(commit.change_id().as_bytes());
            let change_id_short_len = repo
                .shortest_unique_change_id_prefix_len(commit.change_id())
                .unwrap_or(change_id.len()) as u32;
            let parent_tree = self.load_parent_tree(&repo, &commit, "load parent tree")?;
            let files_changed = parent_tree
                .diff_stream(&commit.tree(), &EverythingMatcher)
                .count()
                .block_on() as u32;
            let is_current = name == self.workspace_name.as_str();
            let (path, is_path_resolved) =
                match self.workspace_path(&name, is_current, cache_op_id.as_ref()) {
                    Ok(path) => (path, true),
                    Err(_) => (
                        self.recorded_workspace_path(&name).unwrap_or_default(),
                        false,
                    ),
                };
            workspaces.push(WorkspaceInfo {
                path,
                is_path_resolved,
                is_current,
                operation_id: operation_id.clone(),
                change_id: ShortId::new(change_id, change_id_short_len),
                description: commit.description().lines().next().unwrap_or("").to_owned(),
                timestamp: commit.committer().timestamp.timestamp.0,
                has_conflict: commit.has_conflict(),
                files_changed,
                name,
            });
        }

        let context = "validate workspace list";
        let workspace = load_workspace_internal(&self.path, context)?;
        let current_repo = load_repo_at_head(&workspace, context)?;
        if current_repo.op_id() != repo.op_id() {
            return Ok((workspaces, Some(current_repo)));
        }
        Ok((workspaces, None))
    }

    /// Resolve a workspace root via the CLI, reusing the root already resolved for the current operation.
    pub(super) fn workspace_path(
        &self,
        name: &str,
        is_current: bool,
        cache_op_id: Option<&OperationId>,
    ) -> CoreResult<String> {
        if is_current {
            return Ok(self.path.to_string_lossy().into_owned());
        }
        if let Some(op_id) = cache_op_id
            && let Some(path) = self.workspace_paths.read().unwrap().path(op_id, name)
            && Path::new(path).is_dir()
        {
            return Ok(path.clone());
        }
        let name_arg = format!("--name={name}");
        let path = self.run_jj(&[
            IGNORE_WORKING_COPY_ARG,
            WORKSPACE_COMMAND,
            "root",
            &name_arg,
        ])?;
        if path.is_empty() || !Path::new(&path).is_absolute() {
            return Err(CoreError::Internal {
                message: format!("workspace root for {name} did not resolve to an absolute path"),
            });
        }
        if let Some(op_id) = cache_op_id {
            self.workspace_paths.write().unwrap().insert_if_current(
                op_id,
                name.to_owned(),
                path.clone(),
            );
        }
        Ok(path)
    }

    /// Preserve a stable identity for an unresolved row when jj's root command cannot reach the checkout; shells may use it only to quiesce a matching window before name-based recovery.
    fn recorded_workspace_path(&self, name: &str) -> Option<String> {
        let workspace = load_workspace_internal(&self.path, "read recorded workspace path").ok()?;
        let workspace_name = jj_lib::ref_name::WorkspaceName::new(name);
        let path = SimpleWorkspaceStore::load(workspace.repo_path())
            .and_then(|store| store.get_workspace_path(workspace_name))
            .ok()??;
        normalized_absolute_path(&workspace.repo_path().join(path))?
            .into_os_string()
            .into_string()
            .ok()
    }

    /// Whether this checkout still owns its workspace name. Read from a freshly loaded head, never the cached repo, because the case worth detecting is another process forgetting this workspace, which the in-memory view still shows.
    pub fn workspace_presence(&self) -> WorkspacePresence {
        let context = "check workspace presence";
        let loaded = load_workspace_internal(&self.path, context).and_then(|workspace| {
            let repo = load_repo_at_head(&workspace, context)?;
            Ok((workspace, repo))
        });
        let Ok((workspace, repo)) = loaded else {
            // A failed load only proves removal once the checkout itself is gone; otherwise it can be an op head mid-write or transient IO.
            return if self.path.join(".jj").exists() {
                WorkspacePresence::Unknown
            } else {
                WorkspacePresence::Gone
            };
        };
        if repo
            .view()
            .get_wc_commit_id(self.workspace_name.as_ref())
            .is_none()
        {
            return WorkspacePresence::Gone;
        }
        self.recorded_root_presence(&workspace)
    }

    /// The view maps a name to a commit, not to a checkout, so a name another process forgot and re-added elsewhere still resolves here; only the recorded root says whether this checkout still owns the name.
    fn recorded_root_presence(&self, workspace: &Workspace) -> WorkspacePresence {
        let recorded = SimpleWorkspaceStore::load(workspace.repo_path())
            .and_then(|store| store.get_workspace_path(self.workspace_name.as_ref()));
        let recorded = match recorded {
            // Repos predating recorded roots have no entry, which is no evidence of a move.
            Ok(None) => return WorkspacePresence::Exists,
            Ok(Some(path)) => workspace.repo_path().join(path),
            Err(_) => return WorkspacePresence::Unknown,
        };
        let (Ok(recorded), Ok(own)) = (
            std::fs::canonicalize(recorded),
            std::fs::canonicalize(&self.path),
        ) else {
            return WorkspacePresence::Unknown;
        };
        if recorded == own {
            WorkspacePresence::Exists
        } else {
            WorkspacePresence::Gone
        }
    }

    /// Create a new workspace at the given path, optionally on a specific revision.
    pub fn workspace_add(&self, dest: &str, name: &str, rev: &str) -> CoreResult<String> {
        if !name.is_empty() && !is_valid_workspace_name(name) {
            return Err(CoreError::Internal {
                message: format!("invalid workspace name: {name}"),
            });
        }
        if rev.starts_with('-') {
            return Err(CoreError::Internal {
                message: format!("invalid revision: {rev}"),
            });
        }
        let mut args = vec![WORKSPACE_COMMAND, "add"];
        if !name.is_empty() {
            args.extend(["--name", name]);
        }
        if !rev.is_empty() {
            args.extend(["-r", rev]);
        }
        // `--` so an option-shaped destination is read as a literal path, never as a jj flag.
        args.extend(["--", dest]);
        let output = self.run_jj(&args)?;
        self.reload()?;
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use jj_test::init_jj_repo;

    use super::WorkspacePathCache;
    use crate::Repo;

    #[test]
    fn workspace_path_cache_rejects_stale_generation_writes_and_rollbacks() {
        let temp_dir = init_jj_repo();
        let repo_path = temp_dir.path().join("repo");
        let repo = Repo::open(&repo_path).expect("open repo");
        let old_repo = repo.get_repo();

        let dest = temp_dir.path().join("feature-ws");
        repo.workspace_add(dest.to_str().expect("utf8 dest"), "feature", "")
            .expect("add workspace");
        let new_repo = repo.get_repo();

        let mut cache = WorkspacePathCache::default();
        assert!(cache.sync_to(&old_repo).expect("set old generation"));
        assert!(cache.insert_if_current(old_repo.op_id(), "feature".to_owned(), "old".to_owned()));
        assert!(cache.sync_to(&new_repo).expect("advance generation"));

        assert!(!cache.insert_if_current(
            old_repo.op_id(),
            "feature".to_owned(),
            "stale".to_owned()
        ));
        assert!(
            !cache.sync_to(&old_repo).expect("reject old generation"),
            "an older overlapping list call must not roll the cache back"
        );
        assert_eq!(cache.op_id.as_ref(), Some(new_repo.op_id()));
        assert!(cache.paths.is_empty());
    }
}
