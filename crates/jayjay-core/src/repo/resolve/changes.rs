use std::collections::HashSet;
use std::sync::Arc;

use jj_lib::commit::Commit as JjCommit;
use jj_lib::hex_util::encode_reverse_hex;
use jj_lib::object_id::ObjectId;
use jj_lib::repo::ReadonlyRepo;

use super::super::Repo;
use crate::types::*;

impl Repo {
    pub(crate) fn commit_to_change_info(
        &self,
        repo: &Arc<ReadonlyRepo>,
        commit: &JjCommit,
        immutable_ids: Option<&HashSet<String>>,
        divergent_change_ids: Option<&HashSet<String>>,
    ) -> ChangeInfo {
        let change_id = encode_reverse_hex(commit.change_id().as_bytes());
        let commit_id = commit.id().hex();
        let author = commit.author();
        let bookmarks: Vec<String> = repo
            .view()
            .local_bookmarks_for_commit(commit.id())
            .map(|(name, _)| name.as_str().to_owned())
            .collect();
        let working_copy_commit_id = repo.view().get_wc_commit_id(self.workspace_name.as_ref());
        let is_working_copy = working_copy_commit_id.is_some_and(|id| id == commit.id());
        let has_conflict = commit.has_conflict();
        let is_empty = pollster::block_on(commit.is_empty(repo.as_ref())).unwrap_or(false);
        let is_immutable = immutable_ids
            .map(|ids| ids.contains(&commit_id))
            .unwrap_or(false);
        let is_divergent = divergent_change_ids
            .map(|ids| ids.contains(&change_id))
            .unwrap_or(false);

        ChangeInfo {
            change_id,
            commit_id,
            description: commit.description().to_owned(),
            author: CommitAuthor::new(
                author.name.clone(),
                author.email.clone(),
                author.timestamp.timestamp.0,
            ),
            parents: commit.parent_ids().iter().map(|id| id.hex()).collect(),
            bookmarks,
            is_working_copy,
            has_conflict,
            is_empty,
            is_immutable,
            is_divergent,
        }
    }

    pub(crate) fn should_include_in_log(
        &self,
        repo: &Arc<ReadonlyRepo>,
        commit: &JjCommit,
    ) -> bool {
        let change_id = encode_reverse_hex(commit.change_id().as_bytes());
        let commit_id = commit.id().hex();
        let description = commit.description().trim();
        let bookmarks: Vec<_> = repo
            .view()
            .local_bookmarks_for_commit(commit.id())
            .collect();
        let working_copy_commit_id = repo.view().get_wc_commit_id(self.workspace_name.as_ref());
        let is_working_copy = working_copy_commit_id.is_some_and(|id| id == commit.id());

        if !is_working_copy && description.is_empty() && bookmarks.is_empty() {
            let all_zero_commit = commit_id.chars().all(|c| c == '0');
            let all_z_change = change_id.chars().all(|c| c == 'z');
            let no_parents = commit.parent_ids().is_empty();
            if all_zero_commit || all_z_change || no_parents {
                return false;
            }
        }

        true
    }
}
