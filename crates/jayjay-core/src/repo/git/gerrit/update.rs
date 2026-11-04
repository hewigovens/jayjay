use std::sync::Arc;

use jj_lib::backend::CommitId;
use jj_lib::commit::Commit;
use jj_lib::merge::Merge;
use jj_lib::merged_tree::MergedTree;
use jj_lib::object_id::ObjectId as _;
use jj_lib::repo::{ReadonlyRepo, Repo as _};
use jj_lib::rewrite::merge_commit_trees;

use super::patch_set::PatchSet;
use super::uploads::{GerritUpload, GerritUploadStore};
use crate::repo::git::landed::{change_label, trailer_jj_change_ids, without_gerrit_trailer};
use crate::repo::support::block_on_result;
use crate::repo::{Repo, SyncToken};
use crate::types::*;

impl Repo {
    /// Refuses an upload that would replace a patch set made in Gerrit since JayJay's last upload.
    pub(crate) fn ensure_gerrit_not_ahead(&self, commit: &str) -> JayResult<()> {
        let repo = self.get_repo();
        for id in self.revset_commits(&repo, &format!("mutable() & ::{commit}"))? {
            let local = self.commit(&repo, &id)?;
            let Some(upload) = self.gerrit_upload(&local.change_id().hex())? else {
                continue;
            };
            if let Some(latest) = self.newer_patch_set(&upload)? {
                return Err(JayError::refused(
                    jayjay_i18n::repo::gerrit_newer_patch_set(
                        &change_label(&local),
                        &latest.number.to_string(),
                    ),
                ));
            }
        }
        Ok(())
    }

    fn newer_patch_set(&self, upload: &GerritUpload) -> JayResult<Option<PatchSet>> {
        Ok(self
            .latest_patch_set(upload.number)?
            .filter(|latest| latest.commit != upload.commit))
    }

    pub fn gerrit_update_title(&self) -> Option<String> {
        self.uploads_to_gerrit()
            .then(jayjay_i18n::repo::update_from_gerrit)
    }

    /// Takes Gerrit's newest patch set unless the change was edited locally since its upload.
    pub fn update_from_gerrit(&self, revision: &str, sync: &SyncToken) -> JayResult<String> {
        let _enter = sync.enter();
        let local_id = self.snapshot_and_follow_one(revision)?;
        let repo = self.get_repo();
        let local = self.commit(&repo, &commit_id(&local_id)?)?;
        let label = change_label(&local);
        let change = local.change_id().hex();
        let upload = self
            .gerrit_upload(&change)?
            .ok_or_else(|| JayError::refused(jayjay_i18n::repo::gerrit_not_uploaded(&label)))?;
        let Some(latest) = self.newer_patch_set(&upload)? else {
            return Ok(jayjay_i18n::repo::gerrit_up_to_date(&label));
        };
        let patch_set = latest.number.to_string();
        let fetched_id = self.fetch_patch_set(upload.number, &latest)?;
        sync.check()?;

        let _write = self.write_guard()?;
        self.refresh_working_copy()?;
        let repo = self.get_repo();
        let local = self.commit(&repo, self.follow_rewrites(&repo, local, revision)?.id())?;
        self.ensure_commit_mutable(&repo, &local, revision)?;
        let uploaded = self.commit(&repo, &commit_id(&upload.commit)?)?;
        // Checked after the snapshot so edits made during the fetch are kept.
        let message =
            |commit: &Commit| without_gerrit_trailer(commit.description(), local.change_id());
        let uploaded_tree = self.on_parents(&repo, &uploaded, local.parent_ids())?;
        if local.tree_ids() != uploaded_tree.tree_ids() || message(&local) != message(&uploaded) {
            return Err(JayError::refused(
                jayjay_i18n::repo::gerrit_changed_locally(&label, &patch_set),
            ));
        }
        let fetched = self.commit(&repo, &fetched_id)?;
        let parents = fetched
            .parent_ids()
            .iter()
            .map(|id| self.local_counterpart(id))
            .collect::<JayResult<Vec<_>>>()?;
        let tree = self.on_parents(&repo, &fetched, &parents)?;
        let mut tx = repo.start_transaction();
        block_on_result(
            "take Gerrit patch set",
            tx.repo_mut()
                .rewrite_commit(&local)
                .set_parents(parents)
                .set_tree(tree)
                .set_description(fetched.description())
                .write(),
        )?;
        self.commit_transaction_rebase(tx, "update change from Gerrit")?;
        GerritUploadStore::load().record(
            &self.gerrit_repository_key(),
            vec![(
                change,
                GerritUpload {
                    commit: latest.commit,
                    ..upload
                },
            )],
            &self.live_change_ids()?,
        );
        Ok(jayjay_i18n::repo::gerrit_updated(&label, &patch_set))
    }

    /// `commit`'s own diff on `parents`, so changes inherited from ancestors are neither local edits nor reverted.
    fn on_parents(
        &self,
        repo: &Arc<ReadonlyRepo>,
        commit: &Commit,
        parents: &[CommitId],
    ) -> JayResult<MergedTree> {
        if commit.parent_ids() == parents {
            return Ok(commit.tree());
        }
        let parents = parents
            .iter()
            .map(|id| self.commit(repo, id))
            .collect::<JayResult<Vec<_>>>()?;
        let target_base = block_on_result(
            "merge parent trees",
            merge_commit_trees(repo.as_ref(), &parents),
        )?;
        let base = self.load_parent_tree(repo, commit, "load patch set parent tree")?;
        block_on_result(
            "rebase patch set",
            MergedTree::merge(Merge::from_vec(vec![
                (target_base, "local parents".to_owned()),
                (base, "patch set parents".to_owned()),
                (commit.tree(), "patch set".to_owned()),
            ])),
        )
    }

    /// Gerrit parents are often copies of local changes; building on the local one keeps the stack from diverging.
    fn local_counterpart(&self, id: &CommitId) -> JayResult<CommitId> {
        let repo = self.get_repo();
        let parent = self.commit(&repo, id)?;
        let candidates = std::iter::once(parent.change_id().clone())
            .chain(trailer_jj_change_ids(parent.description()));
        for change_id in candidates {
            let targets =
                block_on_result("resolve Gerrit parent", repo.resolve_change_id(&change_id))?;
            let visible: Vec<CommitId> = targets
                .map(|targets| {
                    targets
                        .visible_with_offsets()
                        .map(|(_, id)| id.clone())
                        .collect()
                })
                .unwrap_or_default();
            if let [only] = visible.as_slice() {
                return Ok(only.clone());
            }
        }
        Ok(id.clone())
    }
}

fn commit_id(hex: &str) -> JayResult<CommitId> {
    CommitId::try_from_hex(hex)
        .ok_or_else(|| JayError::internal(format!("invalid commit id {hex}")))
}
