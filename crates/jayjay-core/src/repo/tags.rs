use jj_lib::git::REMOTE_NAME_FOR_LOCAL_GIT_REPO;
use jj_lib::op_store::{RefTarget, RemoteRef};
use jj_lib::ref_name::{RefName, RemoteName};
use jj_lib::repo::Repo as _;
use jj_lib::view::View;

use super::{Repo, SyncToken, is_valid_bookmark_name};
use crate::types::*;

impl Repo {
    pub fn list_tags(&self) -> CoreResult<Vec<TagInfo>> {
        let repo = self.get_repo();
        Ok(repo
            .view()
            .tags()
            .filter(|(_, targets)| targets.local_target.is_present())
            .map(|(name, targets)| TagInfo {
                name: name.as_str().to_owned(),
                tracked_remotes: targets
                    .remote_refs
                    .iter()
                    .filter(|(remote, remote_ref)| holds_tag(remote, remote_ref))
                    .map(|(remote, _)| remote.as_str().to_owned())
                    .collect(),
            })
            .collect())
    }

    /// `jj tag set` without `--allow-move`: a tag names a release, so an existing tag is never silently moved.
    pub fn create_tag(&self, name: &str, rev: &str) -> CoreResult<()> {
        if !is_valid_bookmark_name(name) {
            return Err(CoreError::internal(format!("Invalid tag name: {name}")));
        }
        let _write = self.write_guard()?;
        let repo = self.get_repo();
        let commit = self.resolve_commit(&repo, rev)?;
        if self.follow_rewrites(&repo, commit.clone(), rev)?.id() != commit.id() {
            return Err(CoreError::internal(
                "Change was rewritten; reselect the tag target",
            ));
        }
        self.with_existing_commit_transaction(
            repo,
            commit,
            "create tag",
            false,
            |_, commit, repo_mut| {
                let name = RefName::new(name);
                if repo_mut.view().get_local_tag(name).is_present()
                    || remote_holds_tag(repo_mut.view(), name)
                {
                    return Err(CoreError::internal(format!(
                        "Tag already exists: {}",
                        name.as_str()
                    )));
                }
                repo_mut.set_local_tag_target(name, RefTarget::normal(commit.id().clone()));
                Ok(())
            },
        )
    }

    pub fn delete_tag(&self, name: &str) -> CoreResult<()> {
        let _write = self.write_guard()?;
        self.with_repo_transaction("delete tag", false, move |_, repo_mut| {
            let name = RefName::new(name);
            if repo_mut.view().get_local_tag(name).is_absent() {
                return Err(CoreError::internal(format!(
                    "tag '{}' not found",
                    name.as_str()
                )));
            }
            repo_mut.set_local_tag_target(name, RefTarget::absent());
            Ok(())
        })
    }

    pub fn delete_tag_and_push(&self, name: &str, sync: &SyncToken) -> CoreResult<String> {
        sync.check()?;
        let target = {
            let _write = self.write_guard()?;
            let target = self
                .get_repo()
                .view()
                .get_local_tag(RefName::new(name))
                .clone();
            self.delete_tag(name)?;
            target
        };
        let push_error = match self.git_push_tag(name, sync) {
            Ok(message) => return Ok(message),
            Err(error) => error,
        };
        // A cancel that lands after the push reloaded the view leaves nothing to restore.
        if !remote_holds_tag(self.get_repo().view(), RefName::new(name)) {
            return Err(push_error);
        }
        self.write_guard()
            .and_then(|_write| {
                self.with_repo_transaction("restore tag after failed push", false, |_, repo_mut| {
                    let name = RefName::new(name);
                    if repo_mut.view().get_local_tag(name).is_absent() {
                        repo_mut.set_local_tag_target(name, target);
                    }
                    Ok(())
                })
            })
            .map_err(|restore_error| {
                CoreError::internal(format!(
                    "{push_error}; could not restore tag '{name}' for retry: {restore_error}"
                ))
            })?;
        Err(push_error)
    }
}

/// jj's synthetic `git` remote mirrors the local refs; it is not a remote the tag can be pushed to or deleted from.
fn holds_tag(remote: &RemoteName, remote_ref: &RemoteRef) -> bool {
    remote.as_str() != REMOTE_NAME_FOR_LOCAL_GIT_REPO.as_str()
        && remote_ref.is_tracked()
        && remote_ref.target.is_present()
}

fn remote_holds_tag(view: &View, name: &RefName) -> bool {
    !tag_remotes(view, name).is_empty()
}

pub(super) fn tag_remotes(view: &View, name: &RefName) -> Vec<String> {
    view.all_remote_tags()
        .filter(|(symbol, remote_ref)| symbol.name == name && holds_tag(symbol.remote, remote_ref))
        .map(|(symbol, _)| symbol.remote.as_str().to_owned())
        .collect()
}
