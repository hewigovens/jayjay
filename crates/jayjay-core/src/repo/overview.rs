use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use futures::StreamExt as _;
use jj_lib::backend::CommitId;
use jj_lib::commit::Commit;
use jj_lib::object_id::ObjectId;
use jj_lib::repo::ReadonlyRepo;
use jj_lib::repo::Repo as _;

use super::Repo;
use super::support::{
    block_on, block_on_result, on_worker_stack, short_change_id, short_commit_id,
};
use crate::types::*;

impl Repo {
    /// Every mutable change grouped into lanes: maximal chains with no fork inside, newest lane first. Reads the
    /// current operation and keeps the handle on it, so changes made by other windows or the CLI show up and the
    /// commits shown resolve for the reads that follow.
    pub fn overview(&self) -> CoreResult<Overview> {
        let repo = block_on_result("load overview", self.get_repo().loader().load_at_head())?;
        self.set_repo(repo.clone());
        on_worker_stack(|| self.overview_of(&repo))
    }

    fn overview_of(&self, repo: &Arc<ReadonlyRepo>) -> CoreResult<Overview> {
        let ordered = self.revset_commits(repo, "mutable()")?;
        let mutable: HashSet<CommitId> = ordered.iter().cloned().collect();
        let mut mutable_children: HashMap<CommitId, u32> = HashMap::new();
        let mut commits: HashMap<CommitId, Commit> = HashMap::new();
        for id in &ordered {
            let commit = self.commit(repo, id)?;
            for parent in commit.parent_ids() {
                if mutable.contains(parent) {
                    *mutable_children.entry(parent.clone()).or_default() += 1;
                }
            }
            commits.insert(id.clone(), commit);
        }

        let trunk_ids = self.revset_commit_ids(repo, "trunk()");
        let trunk_ancestors = self.evaluate_revset(repo, "::trunk()")?;
        let on_trunk = trunk_ancestors.containing_fn();
        let mut workspaces_by_commit: HashMap<CommitId, Vec<String>> = HashMap::new();
        for (name, id) in repo.view().wc_commit_ids() {
            workspaces_by_commit
                .entry(id.clone())
                .or_default()
                .push(name.as_str().to_owned());
        }

        let mut assigned: HashSet<CommitId> = HashSet::new();
        let mut behind_trunk_by_base: HashMap<CommitId, u32> = HashMap::new();
        let mut lanes = Vec::new();
        for head in &ordered {
            if assigned.contains(head) {
                continue;
            }
            let mut chain = vec![head.clone()];
            let mut current = head.clone();
            loop {
                let parents: Vec<&CommitId> = commits[&current]
                    .parent_ids()
                    .iter()
                    .filter(|parent| mutable.contains(*parent))
                    .collect();
                let [parent] = parents[..] else { break };
                if mutable_children.get(parent).copied().unwrap_or(0) != 1
                    || assigned.contains(parent)
                {
                    break;
                }
                chain.push(parent.clone());
                current = parent.clone();
            }
            assigned.extend(chain.iter().cloned());

            let changes: Vec<OverviewChange> = chain
                .iter()
                .map(|id| self.overview_change(repo, &commits[id], &workspaces_by_commit))
                .collect();
            let base_id = commits[&current]
                .parent_ids()
                .first()
                .cloned()
                .ok_or_else(|| CoreError::Internal {
                    message: "mutable change without a parent".to_owned(),
                })?;
            let base_commit = match commits.get(&base_id) {
                Some(commit) => commit.clone(),
                None => self.commit(repo, &base_id)?,
            };
            let kind = if trunk_ids.contains(&base_id.hex()) {
                OverviewBaseKind::Trunk
            } else if mutable.contains(&base_id) {
                OverviewBaseKind::Mutable
            } else if block_on(on_trunk(&base_id)).unwrap_or(false) {
                OverviewBaseKind::OlderTrunk
            } else {
                OverviewBaseKind::Other
            };
            let behind_trunk = match kind {
                OverviewBaseKind::OlderTrunk => *behind_trunk_by_base
                    .entry(base_id.clone())
                    .or_insert_with(|| {
                        self.count_revset(repo, &format!("{}..trunk()", base_id.hex()))
                    }),
                _ => 0,
            };
            let base = OverviewBase {
                change_id: short_change_id(repo, &base_commit),
                commit_id: short_commit_id(repo, &base_commit),
                description: first_line(base_commit.description()),
                timestamp_millis: base_commit.committer().timestamp.timestamp.0,
                kind,
                bookmarks: local_bookmarks(repo, base_commit.id()),
                behind_trunk,
            };
            let workspaces = changes
                .iter()
                .enumerate()
                .flat_map(|(above, change)| {
                    change.workspaces.iter().map(move |name| OverviewWorkspace {
                        name: name.clone(),
                        is_current: *name == *self.workspace_name.as_str(),
                        changes_above: above as u32,
                    })
                })
                .collect();
            let latest_timestamp_millis = changes
                .iter()
                .map(|change| change.timestamp_millis)
                .max()
                .unwrap_or(0);
            let attention = lane_attention(&changes);
            lanes.push(OverviewLane {
                changes,
                base,
                workspaces,
                latest_timestamp_millis,
                attention,
            });
        }

        Ok(Overview {
            lanes,
            workspace_count: repo.view().wc_commit_ids().len() as u32,
        })
    }

    fn overview_change(
        &self,
        repo: &Arc<ReadonlyRepo>,
        commit: &Commit,
        workspaces_by_commit: &HashMap<CommitId, Vec<String>>,
    ) -> OverviewChange {
        OverviewChange {
            change_id: short_change_id(repo, commit),
            commit_id: short_commit_id(repo, commit),
            description: first_line(commit.description()),
            full_description: commit.description().trim_end().to_owned(),
            timestamp_millis: commit.committer().timestamp.timestamp.0,
            is_empty: block_on(commit.is_empty(repo.as_ref())).unwrap_or(false),
            has_conflict: commit.has_conflict(),
            bookmarks: local_bookmarks(repo, commit.id()),
            workspaces: workspaces_by_commit
                .get(commit.id())
                .cloned()
                .unwrap_or_default(),
        }
    }

    fn commit(&self, repo: &Arc<ReadonlyRepo>, id: &CommitId) -> CoreResult<Commit> {
        repo.store()
            .get_commit(id)
            .map_err(|e| CoreError::Internal {
                message: format!("get commit: {e}"),
            })
    }

    /// Commit ids of `revset_str` in the revset's own order (descendants before ancestors).
    fn revset_commits(
        &self,
        repo: &Arc<ReadonlyRepo>,
        revset_str: &str,
    ) -> CoreResult<Vec<CommitId>> {
        let revset = self.evaluate_revset(repo, revset_str)?;
        let mut stream = revset.stream();
        let mut ids = Vec::new();
        while let Some(result) = block_on(stream.next()) {
            ids.push(result.map_err(|e| CoreError::Internal {
                message: format!("revset stream: {e}"),
            })?);
        }
        Ok(ids)
    }
}

fn first_line(description: &str) -> String {
    description.lines().next().unwrap_or("").to_owned()
}

fn local_bookmarks(repo: &Arc<ReadonlyRepo>, id: &CommitId) -> Vec<String> {
    repo.view()
        .local_bookmarks_for_commit(id)
        .map(|(name, _)| name.as_str().to_owned())
        .collect()
}

fn lane_attention(changes: &[OverviewChange]) -> Vec<String> {
    let mut attention = Vec::new();
    let head = &changes[0];
    if head.is_empty && head.description.is_empty() && changes.len() > 1 {
        let below = changes.len() - 1;
        attention.push(format!(
            "Empty, undescribed checkout above {below} change{}",
            if below == 1 { "" } else { "s" }
        ));
    }
    let conflicted = changes.iter().filter(|change| change.has_conflict).count();
    if conflicted > 0 {
        attention.push(format!(
            "{conflicted} change{} with conflicts",
            if conflicted == 1 { "" } else { "s" }
        ));
    }
    let undescribed = changes
        .iter()
        .filter(|change| change.description.is_empty() && !change.is_empty)
        .count();
    if undescribed > 0 {
        attention.push(format!(
            "{undescribed} change{} without a description",
            if undescribed == 1 { "" } else { "s" }
        ));
    }
    attention
}
