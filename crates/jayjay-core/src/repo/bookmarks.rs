use jj_lib::hex_util::encode_reverse_hex;
use jj_lib::object_id::ObjectId;
use jj_lib::op_store::RefTarget;
use jj_lib::ref_name::RefName;
use jj_lib::repo::Repo as _;
use pollster::FutureExt as _;

use super::Repo;
use crate::types::*;

impl Repo {
    pub fn list_bookmarks(&self) -> CoreResult<Vec<BookmarkInfo>> {
        let repo = self.get_repo();
        let mut bookmarks = Vec::new();
        for (name, target) in repo.view().local_bookmarks() {
            if let Some(commit_id) = target.as_normal() {
                let change_id = match repo.store().get_commit(commit_id) {
                    Ok(commit) => encode_reverse_hex(commit.change_id().as_bytes()),
                    Err(_) => String::new(),
                };
                let is_tracking = repo
                    .view()
                    .all_remote_bookmarks()
                    .any(|(sym, _)| sym.name == name);
                bookmarks.push(BookmarkInfo {
                    name: name.as_str().to_owned(),
                    change_id,
                    is_tracking_remote: is_tracking,
                });
            }
        }
        Ok(bookmarks)
    }

    pub fn create_bookmark(&self, name: &str, rev: &str) -> CoreResult<()> {
        let repo = self.get_repo();
        let commit = self.resolve_commit(&repo, rev)?;
        let mut tx = repo.start_transaction();
        tx.repo_mut().set_local_bookmark_target(
            RefName::new(name),
            RefTarget::resolved(Some(commit.id().clone())),
        );
        let new_repo =
            tx.commit("create bookmark")
                .block_on()
                .map_err(|e| CoreError::Internal {
                    message: format!("commit tx: {e}"),
                })?;
        self.set_repo(new_repo);
        Ok(())
    }

    pub fn move_bookmark(&self, name: &str, to_rev: &str) -> CoreResult<()> {
        let repo = self.get_repo();
        let commit = self.resolve_commit(&repo, to_rev)?;
        let mut tx = repo.start_transaction();
        tx.repo_mut().set_local_bookmark_target(
            RefName::new(name),
            RefTarget::resolved(Some(commit.id().clone())),
        );
        let new_repo = tx
            .commit("move bookmark")
            .block_on()
            .map_err(|e| CoreError::Internal {
                message: format!("commit tx: {e}"),
            })?;
        self.set_repo(new_repo);
        Ok(())
    }

    pub fn delete_bookmark(&self, name: &str) -> CoreResult<()> {
        let repo = self.get_repo();
        let mut tx = repo.start_transaction();
        tx.repo_mut()
            .set_local_bookmark_target(RefName::new(name), RefTarget::absent());
        let new_repo =
            tx.commit("delete bookmark")
                .block_on()
                .map_err(|e| CoreError::Internal {
                    message: format!("commit tx: {e}"),
                })?;
        self.set_repo(new_repo);
        Ok(())
    }

    pub fn rename_bookmark(&self, old_name: &str, new_name: &str) -> CoreResult<()> {
        let repo = self.get_repo();
        let target = repo
            .view()
            .get_local_bookmark(RefName::new(old_name))
            .clone();
        if target.is_absent() {
            return Err(CoreError::Internal {
                message: format!("bookmark '{old_name}' not found"),
            });
        }
        let mut tx = repo.start_transaction();
        tx.repo_mut()
            .set_local_bookmark_target(RefName::new(new_name), target);
        tx.repo_mut()
            .set_local_bookmark_target(RefName::new(old_name), RefTarget::absent());
        let new_repo =
            tx.commit("rename bookmark")
                .block_on()
                .map_err(|e| CoreError::Internal {
                    message: format!("commit tx: {e}"),
                })?;
        self.set_repo(new_repo);
        Ok(())
    }

    pub fn track_bookmark(&self, name: &str, remote: &str) -> CoreResult<()> {
        let mut cmd = std::process::Command::new(super::jj_binary());
        cmd.current_dir(&self.path);
        cmd.args(["bookmark", "track", &format!("{name}@{remote}")]);
        let output = cmd.output().map_err(|e| CoreError::Internal {
            message: format!("run jj bookmark track: {e}"),
        })?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CoreError::Internal {
                message: format!("bookmark track failed: {stderr}"),
            });
        }
        self.reload()
    }
}
