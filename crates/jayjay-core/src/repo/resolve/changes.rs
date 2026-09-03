use std::collections::{HashMap, HashSet};
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

#[derive(Default)]
pub(crate) struct CommitRefIndex {
    bookmarks: HashMap<String, Vec<String>>,
    tags: HashMap<String, Vec<String>>,
    workspaces: HashMap<String, Vec<String>>,
    working_copy_commit_id: Option<String>,
    remote_ref_commits: HashSet<String>,
}

struct CommitRefs {
    bookmarks: Vec<String>,
    tags: Vec<String>,
    workspaces: Vec<String>,
    is_working_copy: bool,
    has_remote_ref: bool,
}

#[derive(Default)]
pub(crate) struct ChangeInfoContext<'a> {
    pub immutable_ids: Option<&'a ImmutableIds>,
    pub ref_index: Option<&'a CommitRefIndex>,
    pub divergent_change_ids: Option<&'a HashSet<String>>,
    pub is_empty: Option<bool>,
}

impl CommitRefIndex {
    pub(crate) fn build(
        repo: &Arc<ReadonlyRepo>,
        workspace_name: &jj_lib::ref_name::WorkspaceName,
        displayed_commit_ids: &HashSet<String>,
    ) -> Self {
        let mut index = Self {
            working_copy_commit_id: repo
                .view()
                .get_wc_commit_id(workspace_name)
                .map(ObjectId::hex),
            ..Self::default()
        };
        for (name, target) in repo.view().local_bookmarks() {
            for id in target.added_ids() {
                insert_ref(
                    &mut index.bookmarks,
                    &id.hex(),
                    name.as_str(),
                    displayed_commit_ids,
                );
            }
        }
        for (name, target) in repo.view().local_tags() {
            for id in target.added_ids() {
                insert_ref(
                    &mut index.tags,
                    &id.hex(),
                    name.as_str(),
                    displayed_commit_ids,
                );
            }
        }
        for (name, id) in repo.view().wc_commit_ids() {
            if name.as_str() != workspace_name.as_str() {
                insert_ref(
                    &mut index.workspaces,
                    &id.hex(),
                    name.as_str(),
                    displayed_commit_ids,
                );
            }
        }
        for (_, remote_ref) in repo.view().all_remote_bookmarks() {
            for id in remote_ref.target.added_ids() {
                let id = id.hex();
                if displayed_commit_ids.contains(&id) {
                    index.remote_ref_commits.insert(id);
                }
            }
        }
        index
    }

    fn refs_for(&self, commit_id: &str) -> CommitRefs {
        CommitRefs {
            bookmarks: self.bookmarks.get(commit_id).cloned().unwrap_or_default(),
            tags: self.tags.get(commit_id).cloned().unwrap_or_default(),
            workspaces: self.workspaces.get(commit_id).cloned().unwrap_or_default(),
            is_working_copy: self.working_copy_commit_id.as_deref() == Some(commit_id),
            has_remote_ref: self.remote_ref_commits.contains(commit_id),
        }
    }
}

fn insert_ref(
    index: &mut HashMap<String, Vec<String>>,
    commit_id: &str,
    name: &str,
    displayed_commit_ids: &HashSet<String>,
) {
    if displayed_commit_ids.contains(commit_id) {
        index
            .entry(commit_id.to_owned())
            .or_default()
            .push(name.to_owned());
    }
}

impl Repo {
    pub(crate) fn commit_to_change_info(
        &self,
        repo: &Arc<ReadonlyRepo>,
        commit: &JjCommit,
        context: ChangeInfoContext<'_>,
    ) -> ChangeInfo {
        let change_id = encode_reverse_hex(commit.change_id().as_bytes());
        // Shortest prefix that still uniquely identifies this change. The index
        // is cached on the ReadonlyRepo, so per-commit calls stay cheap.
        let change_id_short_len =
            block_on(repo.shortest_unique_change_id_prefix_len(commit.change_id()))
                .unwrap_or(change_id.len()) as u32;
        let commit_id = commit.id().hex();
        let commit_id_short_len = block_on(
            repo.index()
                .shortest_unique_commit_id_prefix_len(commit.id()),
        )
        .unwrap_or(commit_id.len()) as u32;
        let author = commit.author();
        let refs = context.ref_index.map_or_else(
            || self.commit_refs(repo, commit),
            |index| index.refs_for(&commit_id),
        );
        let has_conflict = commit.has_conflict();
        let is_empty = context
            .is_empty
            .unwrap_or_else(|| block_on(commit.is_empty(repo.as_ref())).unwrap_or(false));
        // Keep display loading resilient to an invalid immutable() revset; mutation paths still enforce immutability.
        let (is_immutable, has_immutable_child) = match context.immutable_ids {
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
        let discardable_working_copy = refs.is_working_copy
            && is_empty
            && commit.description().is_empty()
            && refs.bookmarks.is_empty()
            && refs.tags.is_empty()
            && refs.workspaces.is_empty()
            && !has_children
            && !refs.has_remote_ref;
        let new_change = NewChangeEligibility {
            on_top: !discardable_working_copy,
            before: !is_immutable,
            after: has_children && !has_immutable_child,
        };
        let is_divergent = context
            .divergent_change_ids
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
            bookmarks: refs.bookmarks,
            tags: refs.tags,
            workspaces: refs.workspaces,
            is_working_copy: refs.is_working_copy,
            has_conflict,
            is_empty,
            is_immutable,
            is_divergent,
            new_change,
        }
    }

    fn commit_refs(&self, repo: &Arc<ReadonlyRepo>, commit: &JjCommit) -> CommitRefs {
        CommitRefs {
            bookmarks: repo
                .view()
                .local_bookmarks_for_commit(commit.id())
                .map(|(name, _)| name.as_str().to_owned())
                .collect(),
            tags: repo
                .view()
                .local_tags()
                .filter(|(_, target)| target.added_ids().any(|id| id == commit.id()))
                .map(|(name, _)| name.as_str().to_owned())
                .collect(),
            workspaces: repo
                .view()
                .workspaces_for_wc_commit_id(commit.id())
                .into_iter()
                .filter(|name| **name != *self.workspace_name)
                .map(|name| name.as_str().to_owned())
                .collect(),
            is_working_copy: repo
                .view()
                .get_wc_commit_id(self.workspace_name.as_ref())
                .is_some_and(|id| id == commit.id()),
            has_remote_ref: repo
                .view()
                .all_remote_bookmarks()
                .any(|(_, remote_ref)| remote_ref.target.added_ids().any(|id| id == commit.id())),
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
