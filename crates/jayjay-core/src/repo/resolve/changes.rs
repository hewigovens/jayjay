use std::collections::HashSet;
use std::sync::Arc;

use jj_lib::commit::Commit as JjCommit;
use jj_lib::hex_util::encode_reverse_hex;
use jj_lib::object_id::ObjectId;
use jj_lib::repo::ReadonlyRepo;
use jj_lib::repo::Repo as JjRepo;

use super::super::Repo;
use super::super::log::ImmutableIds;
use super::super::support::block_on;
use crate::types::*;

impl Repo {
    pub(crate) fn commit_to_change_info(
        &self,
        repo: &Arc<ReadonlyRepo>,
        commit: &JjCommit,
        immutable_ids: Option<&ImmutableIds>,
        divergent_change_ids: Option<&HashSet<String>>,
    ) -> ChangeInfo {
        let change_id = encode_reverse_hex(commit.change_id().as_bytes());
        // Shortest prefix that still uniquely identifies this change. The index
        // is cached on the ReadonlyRepo, so per-commit calls stay cheap.
        let change_id_short_len = repo
            .shortest_unique_change_id_prefix_len(commit.change_id())
            .unwrap_or(change_id.len()) as u32;
        let commit_id = commit.id().hex();
        let commit_id_short_len = repo
            .index()
            .shortest_unique_commit_id_prefix_len(commit.id())
            .unwrap_or(commit_id.len()) as u32;
        let author = commit.author();
        let bookmarks: Vec<String> = repo
            .view()
            .local_bookmarks_for_commit(commit.id())
            .map(|(name, _)| name.as_str().to_owned())
            .collect();
        // jj_lib has no commit-scoped tag helper, so mirror local_bookmarks_for_commit.
        let tags: Vec<String> = repo
            .view()
            .local_tags()
            .filter(|(_, target)| target.added_ids().any(|id| id == commit.id()))
            .map(|(name, _)| name.as_str().to_owned())
            .collect();
        let working_copy_commit_id = repo.view().get_wc_commit_id(self.workspace_name.as_ref());
        let is_working_copy = working_copy_commit_id.is_some_and(|id| id == commit.id());
        let workspaces: Vec<String> = repo
            .view()
            .workspaces_for_wc_commit_id(commit.id())
            .into_iter()
            .filter(|name| **name != *self.workspace_name)
            .map(|name| name.as_str().to_owned())
            .collect();
        let has_conflict = commit.has_conflict();
        let is_empty = block_on(commit.is_empty(repo.as_ref())).unwrap_or(false);
        // Keep display loading resilient to an invalid immutable() revset; mutation paths still enforce immutability.
        let (is_immutable, has_immutable_child) = match immutable_ids {
            Some(ids) => (
                ids.commits.contains(&commit_id),
                ids.parents.contains(&commit_id),
            ),
            None => {
                let is_immutable = self.is_commit_immutable(repo, commit).unwrap_or(false);
                let has_immutable_child =
                    is_immutable && self.has_immutable_child(repo, commit).unwrap_or(false);
                (is_immutable, has_immutable_child)
            }
        };
        let has_children = !repo.view().heads().contains(commit.id());
        let discardable_working_copy = is_working_copy
            && is_empty
            && commit.description().is_empty()
            && bookmarks.is_empty()
            && tags.is_empty()
            && workspaces.is_empty()
            && !has_children
            && !repo
                .view()
                .all_remote_bookmarks()
                .any(|(_, remote_ref)| remote_ref.target.added_ids().any(|id| id == commit.id()));
        let new_change = NewChangeEligibility {
            on_top: !discardable_working_copy,
            before: !is_immutable,
            after: has_children && !has_immutable_child,
        };
        let is_divergent = divergent_change_ids
            .map(|ids| ids.contains(&change_id))
            .unwrap_or(false);

        ChangeInfo {
            change_id: ShortId::new(change_id, change_id_short_len),
            commit_id: ShortId::new(commit_id, commit_id_short_len),
            description: commit.description().to_owned(),
            author: CommitAuthor::new(
                author.name.clone(),
                author.email.clone(),
                author.timestamp.timestamp.0,
            ),
            parents: commit.parent_ids().iter().map(|id| id.hex()).collect(),
            bookmarks,
            tags,
            workspaces,
            is_working_copy,
            has_conflict,
            is_empty,
            is_immutable,
            is_divergent,
            new_change,
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
