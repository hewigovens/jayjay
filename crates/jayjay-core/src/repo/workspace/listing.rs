use std::path::{Component, Path, PathBuf};

use futures::StreamExt as _;
use jj_lib::hex_util::encode_reverse_hex;
use jj_lib::matchers::EverythingMatcher;
use jj_lib::object_id::ObjectId;
use jj_lib::ref_name::WorkspaceName;
use jj_lib::repo::Repo as _;
use jj_lib::workspace_store::{SimpleWorkspaceStore, WorkspaceStore as _};
use pollster::FutureExt as _;

use super::super::Repo;
use super::super::support::{
    block_on, block_on_result, load_repo_at_head, load_workspace_internal,
};
use crate::types::*;

impl Repo {
    /// Reads the current operation so workspaces added elsewhere show up without a full refresh; never snapshots a working copy.
    pub fn workspace_list(&self) -> CoreResult<Vec<WorkspaceInfo>> {
        let repo = block_on_result("load workspaces", self.get_repo().loader().load_at_head())?;
        let mut workspaces = Vec::new();
        for (name, commit_id) in repo.view().wc_commit_ids() {
            let Ok(commit) = repo.store().get_commit(commit_id) else {
                continue;
            };
            let change_id = encode_reverse_hex(commit.change_id().as_bytes());
            let change_id_short_len =
                block_on(repo.shortest_unique_change_id_prefix_len(commit.change_id()))
                    .unwrap_or(change_id.len()) as u32;
            let parent_tree = self.load_parent_tree(&repo, &commit, "load parent tree")?;
            let files_changed = parent_tree
                .diff_stream(&commit.tree(), &EverythingMatcher)
                .count()
                .block_on() as u32;
            let is_current = name.as_str() == self.workspace_name.as_str();
            let (path, is_path_resolved) = if is_current {
                (self.path.clone(), true)
            } else {
                let recorded = self.recorded_workspace_root(name);
                match recorded.as_deref().and_then(existing_dir) {
                    Some(root) => (root, true),
                    None => (recorded.unwrap_or_default(), false),
                }
            };
            workspaces.push(WorkspaceInfo {
                name: name.as_str().to_owned(),
                path: path.to_string_lossy().into_owned(),
                is_path_resolved,
                is_current,
                change_id: ShortId::new(change_id, change_id_short_len),
                description: commit.description().lines().next().unwrap_or("").to_owned(),
                timestamp: commit.committer().timestamp.timestamp.0,
                has_conflict: commit.has_conflict(),
                files_changed,
            });
        }
        Ok(workspaces)
    }

    pub(super) fn recorded_workspace_root(&self, name: &WorkspaceName) -> Option<PathBuf> {
        let store = SimpleWorkspaceStore::load(&self.repo_path).ok()?;
        let relative = store.get_workspace_path(name).ok()??;
        Some(lexically_normalized(&self.repo_path.join(relative)))
    }

    /// Reads a fresh head; the in-memory view still shows a workspace another process forgot.
    pub fn workspace_presence(&self) -> WorkspacePresence {
        let context = "check workspace presence";
        let repo = load_workspace_internal(&self.path, context)
            .and_then(|workspace| load_repo_at_head(&workspace, context));
        let Ok(repo) = repo else {
            // Gone only once the checkout itself is gone.
            return if self.path.join(".jj").exists() {
                WorkspacePresence::Unknown
            } else {
                WorkspacePresence::Gone
            };
        };
        let name = self.workspace_name.as_ref();
        if repo.view().get_wc_commit_id(name).is_none() {
            return WorkspacePresence::Gone;
        }
        // The view still maps a re-added name; only the recorded root says this checkout owns it.
        let Some(recorded) = self.recorded_workspace_root(name) else {
            return WorkspacePresence::Exists;
        };
        match (existing_dir(&recorded), existing_dir(&self.path)) {
            (Some(recorded), Some(own)) if recorded == own => WorkspacePresence::Exists,
            (Some(_), Some(_)) => WorkspacePresence::Gone,
            _ => WorkspacePresence::Unknown,
        }
    }
}

fn lexically_normalized(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

/// dunce: no Windows verbatim prefix, so paths compare equal to what jj stores.
pub(super) fn existing_dir(path: &Path) -> Option<PathBuf> {
    dunce::canonicalize(path).ok().filter(|path| path.is_dir())
}
