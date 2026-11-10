use std::cell::LazyCell;
use std::sync::Arc;

use jj_lib::backend::CommitId;
use jj_lib::git::REMOTE_NAME_FOR_LOCAL_GIT_REPO;
use jj_lib::object_id::ObjectId;
use jj_lib::op_store::{RefTarget, RemoteRef};
use jj_lib::ref_name::{RefName, RemoteName, RemoteNameBuf};
use jj_lib::repo::{MutableRepo, ReadonlyRepo, Repo as _};

use super::Repo;
use super::ref_name::ensure_valid_ref_name;
use super::support::{block_on_result, short_change_id};
use crate::trunk;
use crate::types::*;

impl Repo {
    pub fn trunk_name(&self) -> String {
        let repo = self.get_repo();
        let bookmarks: Vec<String> = self
            .revset_commit_ids(&repo, "trunk()")
            .iter()
            .filter_map(CommitId::try_from_hex)
            .flat_map(|id| self.commit_bookmarks(&repo).names(&id))
            .collect();
        trunk::display_name(&bookmarks).to_owned()
    }

    #[cfg_attr(feature = "hotpath", hotpath::measure(impl_type = "Repo"))]
    pub fn list_bookmarks(&self) -> JayResult<Vec<BookmarkInfo>> {
        let repo = self.get_repo();
        let mut bookmarks = Vec::new();
        let mut orphans = Vec::new();
        let default_remote = LazyCell::new(|| self.default_remote());
        for (name, refs) in repo.view().bookmarks() {
            let local_target = refs.local_target;
            // jj's synthetic `git` remote mirrors the local git refs; it is not a remote the bookmark can track or sync with.
            let remote_refs: Vec<_> = refs
                .remote_refs
                .into_iter()
                .filter(|(remote, _)| *remote != REMOTE_NAME_FOR_LOCAL_GIT_REPO)
                .collect();
            if local_target.is_absent() {
                orphans.extend(orphan_bookmark(&repo, name, &remote_refs, &default_remote));
                continue;
            }
            let tracked_remotes: Vec<String> = remote_refs
                .iter()
                .filter(|(_, remote_ref)| remote_ref.is_tracked())
                .map(|(remote, _)| remote.as_str().to_owned())
                .collect();
            let available_remotes: Vec<String> = remote_refs
                .iter()
                .map(|(remote, _)| remote.as_str().to_owned())
                .collect();

            let local_target_commit = local_target.as_normal();
            let remote_targets: Vec<RemoteBookmarkTarget> = remote_refs
                .iter()
                .filter(|(_, remote_ref)| remote_ref.is_tracked())
                .map(|(remote, remote_ref)| {
                    let (change_id, description) = summary_at_target(&repo, &remote_ref.target);
                    let (status, ahead, behind) =
                        self.remote_sync_status(&repo, local_target_commit, &remote_ref.target);
                    RemoteBookmarkTarget {
                        remote: remote.as_str().to_owned(),
                        change_id: change_id.id,
                        description,
                        status,
                        ahead,
                        behind,
                    }
                })
                .collect();

            let (change_id, description) = summary_at_target(&repo, local_target);
            bookmarks.push(BookmarkInfo {
                name: name.as_str().to_owned(),
                change_id,
                description,
                is_tracking_remote: !tracked_remotes.is_empty(),
                is_deleted: false,
                is_conflicted: !local_target.is_resolved(),
                tracked_remotes,
                available_remotes,
                has_local_target: true,
                remote_targets,
            });
        }
        bookmarks.extend(orphans);
        Ok(bookmarks)
    }

    pub(super) fn remote_bookmark_change_id(&self, name: &str, remote: &str) -> Option<String> {
        let repo = self.get_repo();
        repo.view()
            .all_remote_bookmarks()
            .find(|(symbol, remote_ref)| {
                symbol.name.as_str() == name
                    && symbol.remote.as_str() == remote
                    && !remote_ref.target.is_absent()
            })
            .map(|(_, remote_ref)| summary_at_target(&repo, &remote_ref.target).0.id)
    }

    pub fn create_bookmark(&self, name: &str, rev: &str) -> JayResult<()> {
        ensure_valid_ref_name("bookmark", name)?;
        let _write = self.write_guard()?;
        self.with_resolved_commit_transaction(
            rev,
            "create bookmark",
            false,
            |_, commit, repo_mut| {
                self.set_bookmark_target(
                    repo_mut,
                    name,
                    RefTarget::resolved(Some(commit.id().clone())),
                );
                Ok(())
            },
        )
    }

    pub fn move_bookmark(&self, name: &str, to_rev: &str) -> JayResult<()> {
        let _write = self.write_guard()?;
        self.with_resolved_commit_transaction(
            to_rev,
            "move bookmark",
            false,
            |_, commit, repo_mut| {
                self.set_bookmark_target(
                    repo_mut,
                    name,
                    RefTarget::resolved(Some(commit.id().clone())),
                );
                Ok(())
            },
        )
    }

    pub fn delete_bookmark(&self, name: &str) -> JayResult<()> {
        let _write = self.write_guard()?;
        self.with_repo_transaction("delete bookmark", false, |_, repo_mut| {
            self.set_bookmark_target(repo_mut, name, RefTarget::absent());
            Ok(())
        })
    }

    /// Drop this bookmark from the commit `rev` resolves to. Pass the DAG chip's commit id so a working-copy snapshot cannot miss the target. Remaining targets stay; one remaining target resolves the conflict; none deletes the bookmark.
    pub fn remove_bookmark_from_rev(&self, name: &str, rev: &str) -> JayResult<()> {
        let _write = self.write_guard()?;
        let repo = self.get_repo();
        let commit = self.resolve_commit(&repo, rev)?;
        let current = repo.view().get_local_bookmark(RefName::new(name)).clone();
        let Some(new_target) = bookmark_target_without_commit(&current, commit.id()) else {
            return Err(JayError::Internal {
                message: format!("bookmark '{name}' does not point at this change"),
            });
        };
        let description = if new_target.is_absent() {
            "delete bookmark"
        } else if !new_target.is_resolved() {
            "remove bookmark from revision"
        } else {
            "resolve bookmark"
        };
        self.with_existing_commit_transaction(
            repo,
            commit,
            description,
            false,
            move |_, _, repo_mut| {
                self.set_bookmark_target(repo_mut, name, new_target);
                Ok(())
            },
        )
    }

    /// `jj bookmark forget`: drop the local bookmark and stop tracking its remote counterparts, so nothing is staged for a push. The `git` remote is left to the export, which drops a colocated branch along with its bookmark, as the CLI does.
    pub fn forget_bookmark(&self, name: &str) -> JayResult<()> {
        let _write = self.write_guard()?;
        self.with_repo_transaction("forget bookmark", false, |_, repo_mut| {
            self.forget_bookmark_in(repo_mut, name)
        })
    }

    fn forget_bookmark_in(&self, repo_mut: &mut MutableRepo, name: &str) -> JayResult<()> {
        let bookmark = RefName::new(name);
        let remotes: Vec<RemoteNameBuf> = repo_mut
            .view()
            .all_remote_bookmarks()
            .filter(|(symbol, _)| {
                symbol.name == bookmark && symbol.remote != REMOTE_NAME_FOR_LOCAL_GIT_REPO
            })
            .map(|(symbol, _)| symbol.remote.to_owned())
            .collect();
        if remotes.is_empty() && repo_mut.view().get_local_bookmark(bookmark).is_absent() {
            return Err(JayError::Internal {
                message: format!("bookmark '{name}' not found"),
            });
        }
        self.set_bookmark_target(repo_mut, name, RefTarget::absent());
        for remote in &remotes {
            repo_mut.untrack_remote_bookmark(bookmark.to_remote_symbol(remote));
        }
        Ok(())
    }

    pub fn rename_bookmark(&self, old_name: &str, new_name: &str) -> JayResult<()> {
        let _write = self.write_guard()?;
        if old_name == new_name {
            return Ok(());
        }
        ensure_valid_ref_name("bookmark", new_name)?;
        // Destination uniqueness is checked inside the transaction so a concurrent create cannot be overwritten.
        self.with_repo_transaction("rename bookmark", false, move |_, repo_mut| {
            let target = repo_mut
                .view()
                .get_local_bookmark(RefName::new(old_name))
                .clone();
            if target.is_absent() {
                return Err(JayError::Internal {
                    message: format!("bookmark '{old_name}' not found"),
                });
            }
            if !repo_mut
                .view()
                .get_local_bookmark(RefName::new(new_name))
                .is_absent()
            {
                return Err(JayError::Internal {
                    message: format!("Bookmark already exists: {new_name}"),
                });
            }
            self.set_bookmark_target(repo_mut, new_name, target);
            self.set_bookmark_target(repo_mut, old_name, RefTarget::absent());
            Ok(())
        })
    }

    /// `jj bookmark track name@remote`: merge the remote bookmark into the local one and follow it from now on. Already-tracked is not an error.
    pub fn track_bookmark(&self, name: &str, remote: &str) -> JayResult<()> {
        let _write = self.write_guard()?;
        let symbol = RefName::new(name).to_remote_symbol(RemoteName::new(remote));
        let remote_ref = self.get_repo().view().get_remote_bookmark(symbol).clone();
        if remote_ref.is_absent() {
            return Err(JayError::Internal {
                message: format!("no such remote bookmark: {name}@{remote}"),
            });
        }
        if remote_ref.is_tracked() {
            return Ok(());
        }
        self.with_repo_transaction("track remote bookmark", false, move |_, repo_mut| {
            block_on_result(
                "track remote bookmark",
                repo_mut.track_remote_bookmark(symbol),
            )
        })
    }

    /// Fetch the configured remotes, then forget bookmarks that are deleted locally but still tracked on a remote.
    /// Forgetting never propagates deletions to remotes. Returns the number of bookmarks forgotten.
    pub fn forget_stale_bookmarks(&self) -> JayResult<u32> {
        let _ = self.run_jj(&["git", "fetch"]);
        let _write = self.write_guard()?;
        self.reload()?;

        let bookmarks = self.list_bookmarks()?;
        let stale: Vec<&str> = bookmarks
            .iter()
            .filter(|b| b.is_deleted)
            .map(|b| b.name.as_str())
            .collect();
        let count = stale.len() as u32;
        if count > 0 {
            // One operation, so a single undo restores every bookmark.
            self.with_repo_transaction("forget stale bookmarks", false, |_, repo_mut| {
                stale
                    .iter()
                    .try_for_each(|name| self.forget_bookmark_in(repo_mut, name))
            })?;
            self.reload()?;
        }
        Ok(count)
    }
}

impl Repo {
    /// Classify a tracked remote ref against its local bookmark, with ahead/behind counts.
    fn remote_sync_status(
        &self,
        repo: &Arc<ReadonlyRepo>,
        local: Option<&CommitId>,
        remote: &RefTarget,
    ) -> (RemoteSyncStatus, u32, u32) {
        if remote.is_absent() {
            return (RemoteSyncStatus::Deleted, 0, 0);
        }
        match (local, remote.as_normal()) {
            (Some(l), Some(r)) if l == r => (RemoteSyncStatus::Synced, 0, 0),
            (Some(l), Some(r)) => {
                let ahead = self.count_revset(repo, &format!("{}..{}", r.hex(), l.hex()));
                let behind = self.count_revset(repo, &format!("{}..{}", l.hex(), r.hex()));
                let status = match (ahead, behind) {
                    (0, 0) => RemoteSyncStatus::Synced,
                    (_, 0) => RemoteSyncStatus::Ahead,
                    (0, _) => RemoteSyncStatus::Behind,
                    _ => RemoteSyncStatus::Diverged,
                };
                (status, ahead, behind)
            }
            _ => (RemoteSyncStatus::Diverged, 0, 0),
        }
    }
}

/// An entry for a name with no local target. A tracked ref is a bookmark deleted locally but still on the remote (jj's "(deleted)"); an untracked one is remote-only.
fn orphan_bookmark(
    repo: &ReadonlyRepo,
    name: &RefName,
    remote_refs: &[(&RemoteName, &RemoteRef)],
    default_remote: &str,
) -> Option<BookmarkInfo> {
    let refs: Vec<_> = remote_refs
        .iter()
        .filter(|(_, remote_ref)| !remote_ref.target.is_absent())
        .collect();
    let first_target = &refs
        .iter()
        .find(|(remote, _)| remote.as_str() == default_remote)
        .or(refs.first())?
        .1
        .target;
    let tracked_remotes: Vec<String> = refs
        .iter()
        .filter(|(_, remote_ref)| remote_ref.is_tracked())
        .map(|(remote, _)| remote.as_str().to_owned())
        .collect();
    let (change_id, description) = summary_at_target(repo, first_target);
    Some(BookmarkInfo {
        name: name.as_str().to_owned(),
        change_id,
        description,
        is_tracking_remote: false,
        is_deleted: !tracked_remotes.is_empty(),
        is_conflicted: !first_target.is_resolved(),
        tracked_remotes,
        available_remotes: refs
            .iter()
            .map(|(remote, _)| remote.as_str().to_owned())
            .collect(),
        has_local_target: false,
        remote_targets: Vec::new(),
    })
}

/// The change id, with the repo-wide unique prefix length the DAG highlights, and the first description line of a ref target.
fn summary_at_target(repo: &ReadonlyRepo, target: &RefTarget) -> (ShortId, String) {
    let Some(commit) = target
        .as_normal()
        .and_then(|commit_id| repo.store().get_commit(commit_id).ok())
    else {
        return (ShortId::new(String::new(), 0), String::new());
    };
    let description = commit.description().lines().next().unwrap_or("").to_owned();
    (short_change_id(repo, &commit), description)
}

/// Drop `commit_id` from a bookmark target. `None` if that commit is not one of the added ids.
pub(super) fn bookmark_target_without_commit(
    target: &RefTarget,
    commit_id: &CommitId,
) -> Option<RefTarget> {
    let mut remaining: Vec<CommitId> = target.present_adds().cloned().collect();
    let before = remaining.len();
    remaining.retain(|id| id != commit_id);
    if remaining.len() == before {
        return None;
    }
    Some(match remaining.as_slice() {
        [] => RefTarget::absent(),
        [only] => RefTarget::normal(only.clone()),
        _ => RefTarget::from_legacy_form(target.present_removes().cloned(), remaining),
    })
}
