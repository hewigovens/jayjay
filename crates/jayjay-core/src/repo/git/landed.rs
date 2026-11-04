use std::collections::HashSet;
use std::sync::Arc;

use jj_lib::backend::{ChangeId, CommitId};
use jj_lib::object_id::ObjectId as _;
use jj_lib::repo::{ReadonlyRepo, Repo as _};

use crate::repo::Repo;
use crate::repo::support::block_on_result;
use crate::types::*;

impl Repo {
    /// Abandons changes a pull left empty, matched to landed commits by change ID or Gerrit trailer.
    pub(super) fn abandon_landed_changes(
        &self,
        before: &Arc<ReadonlyRepo>,
    ) -> JayResult<Vec<String>> {
        let old_trunk = self.revset_commit_ids(before, "trunk()");
        if old_trunk.is_empty() {
            return Ok(Vec::new());
        }
        let repo = self.get_repo();
        let old_trunk: Vec<&str> = old_trunk.iter().map(String::as_str).collect();
        let landed =
            self.revset_commits(&repo, &format!("::trunk() ~ ::({})", old_trunk.join("|")))?;
        if landed.is_empty() {
            return Ok(Vec::new());
        }
        let mut landed_change_ids = HashSet::new();
        let mut landed_trailers = HashSet::new();
        for id in landed {
            let commit = self.commit(&repo, &id)?;
            landed_change_ids.insert(commit.change_id().clone());
            landed_trailers.extend(trailer_change_ids(commit.description()).map(str::to_owned));
        }

        let own_wc = repo
            .view()
            .get_wc_commit_id(self.workspace_name.as_ref())
            .cloned();
        let other_wcs: HashSet<&CommitId> = repo
            .view()
            .wc_commit_ids()
            .values()
            .filter(|id| Some(*id) != own_wc.as_ref())
            .collect();
        let mut tx = repo.start_transaction();
        let mut abandoned = Vec::new();
        for id in self.revset_commits(&repo, "mutable() & trunk()..")? {
            if other_wcs.contains(&id) {
                continue;
            }
            let commit = self.commit(&repo, &id)?;
            let change_id = commit.change_id();
            let landed = landed_change_ids.contains(change_id)
                || gerrit_change_ids(change_id)
                    .iter()
                    .any(|id| landed_trailers.contains(id));
            if landed && block_on_result("check landed change", commit.is_empty(repo.index()))? {
                abandoned.push(change_label(&commit));
                tx.repo_mut().record_abandoned_commit(&commit);
            }
        }
        if !abandoned.is_empty() {
            self.commit_transaction_rebase(tx, "abandon landed changes")?;
        }
        Ok(abandoned)
    }
}

/// `Change-Id: <id>`, or `Link: <review-url>/id/<id>` with `gerrit.review-url`.
fn trailer_change_ids(description: &str) -> impl Iterator<Item = &str> {
    description.lines().filter_map(|line| {
        if let Some(id) = line.strip_prefix("Change-Id:") {
            return Some(id.trim());
        }
        let url = line.strip_prefix("Link:")?;
        url.trim().rsplit_once("/id/").map(|(_, id)| id)
    })
}

/// Maps a commit Gerrit rewrote back to its local change.
pub(super) fn trailer_jj_change_ids(description: &str) -> impl Iterator<Item = ChangeId> {
    trailer_change_ids(description).filter_map(|id| {
        match id
            .strip_prefix('I')
            .and_then(|id| id.strip_suffix("6a6a6964"))
        {
            Some(hex) => ChangeId::try_from_hex(hex),
            None => ChangeId::try_from_reverse_hex(id),
        }
    })
}

pub(super) fn without_gerrit_trailer(description: &str, change_id: &ChangeId) -> String {
    let own = gerrit_change_ids(change_id);
    description
        .lines()
        .filter(|line| !trailer_change_ids(line).any(|id| own.iter().any(|own| own == id)))
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_owned()
}

/// jj-cli's `format_gerrit_change_id_trailer`, and the reverse-hex form `Link` trailers use.
fn gerrit_change_ids(change_id: &ChangeId) -> [String; 2] {
    [
        format!("I{}6a6a6964", change_id.hex()),
        change_id.reverse_hex(),
    ]
}

pub(super) fn change_label(commit: &jj_lib::commit::Commit) -> String {
    let short_id: String = commit.change_id().reverse_hex().chars().take(8).collect();
    match commit.description().lines().next() {
        Some(subject) if !subject.is_empty() => format!("{short_id} {subject}"),
        _ => short_id,
    }
}

#[cfg(test)]
mod tests {
    use jj_lib::backend::ChangeId;

    use super::{gerrit_change_ids, trailer_jj_change_ids, without_gerrit_trailer};

    #[test]
    fn gerrit_trailers_map_back_to_the_jj_change() {
        let change_id = ChangeId::from_hex("4a384355dfd8d2accd75fe1e8f6c7907");
        let [change_trailer, link_id] = gerrit_change_ids(&change_id);
        let uploaded = format!("Add hello\n\nBug: 42\nChange-Id: {change_trailer}\n");
        assert_eq!(
            without_gerrit_trailer(&uploaded, &change_id),
            "Add hello\n\nBug: 42"
        );
        let linked = format!("Add hello\n\nLink: https://review.example.org/id/{link_id}\n");
        assert_eq!(without_gerrit_trailer(&linked, &change_id), "Add hello");
        assert_eq!(
            trailer_jj_change_ids(&uploaded)
                .chain(trailer_jj_change_ids(&linked))
                .collect::<Vec<_>>(),
            [change_id.clone(), change_id]
        );
    }
}
