use jj_lib::backend::CommitId;
use jj_lib::commit::Commit;
use jj_lib::repo::ReadonlyRepo;

use super::result::change_view;
use super::session::RestoreTarget;
use crate::Repo;
use crate::repo::support::block_on_result;
use crate::types::*;
use jj_lib::repo::Repo as _;

impl Repo {
    pub(super) fn restore_target(&self, target: &RestoreTarget) -> JayResult<Option<String>> {
        self.refresh_working_copy()?;
        match target {
            RestoreTarget::Change { change_id, parents } => {
                match self.resolve_commit(&self.get_repo(), change_id) {
                    Ok(commit) => {
                        self.edit_existing(commit, "bisect: finish")?;
                        Ok(None)
                    }
                    Err(_) => {
                        self.new_on_parents(parents, "bisect: finish")?;
                        Ok(Some(
                        "The original change was gone, so this restored a new change on its parents.".to_owned(),
                    ))
                    }
                }
            }
            RestoreTarget::Parents { parents } => {
                self.new_on_parents(parents, "bisect: finish")?;
                Ok(None)
            }
        }
    }

    fn edit_existing(&self, commit: Commit, description: &str) -> JayResult<()> {
        let repo = self.get_repo();
        if self.working_copy_commit(&repo)?.id() == commit.id() {
            return Ok(());
        }
        self.with_repo_transaction(description, true, |_, repo_mut| {
            self.edit_working_copy_commit(repo_mut, &commit, description)
        })
    }

    fn new_on_parents(&self, parents: &[CommitId], description: &str) -> JayResult<()> {
        if parents.is_empty() {
            return Err(JayError::internal("bisect has no parents to restore onto"));
        }
        let repo = self.get_repo();
        let commits = parents
            .iter()
            .map(|id| {
                let commit = repo
                    .store()
                    .get_commit(id)
                    .map_err(|error| JayError::internal(format!("load restore parent: {error}")))?;
                self.follow_rewrites(&repo, commit, "bisect restore parent")
            })
            .collect::<JayResult<Vec<_>>>()?;
        let tree = if commits.len() == 1 {
            commits[0].tree()
        } else {
            block_on_result(
                "merge restore parents",
                jj_lib::rewrite::merge_commit_trees(repo.as_ref(), &commits),
            )?
        };
        let parent_ids: Vec<CommitId> = commits.iter().map(|commit| commit.id().clone()).collect();
        self.with_repo_transaction(description, true, move |_, repo_mut| {
            let new_commit = repo_mut
                .new_commit(parent_ids, tree)
                .set_description("")
                .write();
            let new_commit = block_on_result("bisect restore", new_commit)?;
            self.edit_working_copy_commit(repo_mut, &new_commit, description)
        })
    }

    /// `MutableRepo::edit` abandons `@` only when it is an unreferenced discardable head. Anything else has to be restored by change id, or a bookmarked empty change would be left behind.
    pub(super) fn capture_restore(&self, range: &[CommitId]) -> JayResult<RestoreTarget> {
        let repo = self.get_repo();
        let working_copy = self.working_copy_commit(&repo)?;
        let parents = working_copy.parent_ids().to_vec();
        if !range.contains(working_copy.id()) && self.edit_would_abandon(&repo, &working_copy)? {
            Ok(RestoreTarget::Parents { parents })
        } else {
            Ok(RestoreTarget::Change {
                change_id: super::result::change_id_of(&working_copy),
                parents,
            })
        }
    }

    fn edit_would_abandon(&self, repo: &ReadonlyRepo, commit: &Commit) -> JayResult<bool> {
        let discardable = block_on_result("bisect restore", commit.is_discardable(repo))?;
        if !discardable || !repo.view().heads().contains(commit.id()) {
            return Ok(false);
        }
        Ok(!commit_is_referenced(
            repo,
            self.workspace_name.as_ref(),
            commit.id(),
        ))
    }

    pub(super) fn kept_changes(&self, temps: &[String]) -> JayResult<Vec<BisectChange>> {
        let repo = self.get_repo();
        let mut kept = Vec::new();
        for change_id in temps {
            let Ok(commit) = self.resolve_commit(&repo, change_id) else {
                continue;
            };
            kept.push(change_view(&commit));
        }
        Ok(kept)
    }
}

fn commit_is_referenced(
    repo: &ReadonlyRepo,
    workspace: &jj_lib::ref_name::WorkspaceName,
    id: &CommitId,
) -> bool {
    let view = repo.view();
    if view
        .wc_commit_ids()
        .iter()
        .any(|(name, wc_id)| name.as_str() != workspace.as_str() && wc_id == id)
    {
        return true;
    }
    if view
        .local_bookmarks()
        .any(|(_, target)| target.present_adds().any(|add| add == id))
    {
        return true;
    }
    view.local_tags()
        .any(|(_, target)| target.present_adds().any(|add| add == id))
}
