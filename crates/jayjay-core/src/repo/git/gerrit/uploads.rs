use std::collections::{BTreeMap, HashSet};

use jj_lib::backend::CommitId;
use jj_lib::object_id::ObjectId as _;
use serde::{Deserialize, Serialize};

use crate::json_store::JsonStore;
use crate::repo::Repo;
use crate::repositories::normalize_repository_path;
use crate::types::*;

/// The patch set JayJay last uploaded; a newer one in Gerrit is someone else's edit.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub(super) struct GerritUpload {
    /// Change numbers are per server.
    pub(super) url: String,
    pub(super) number: u64,
    pub(super) commit: String,
}

/// Keyed by primary workspace root, then by jj change ID in hex.
#[derive(Clone, Default, Deserialize, Serialize)]
struct GerritUploads {
    repositories: BTreeMap<String, BTreeMap<String, GerritUpload>>,
}

impl GerritUploads {
    fn normalize(&mut self) {
        self.repositories
            .retain(|path, changes| !path.is_empty() && !changes.is_empty());
    }
}

pub(super) struct GerritUploadStore(JsonStore<GerritUploads>);

impl GerritUploadStore {
    pub(super) fn load() -> Self {
        Self(JsonStore::load(
            "gerrit-uploads",
            "JAYJAY_GERRIT_UPLOADS_PATH",
            "gerrit_uploads.json",
            GerritUploads::normalize,
        ))
    }

    pub(super) fn get(&mut self, repository: &str, change: &str) -> Option<GerritUpload> {
        self.0
            .read()
            .repositories
            .get(repository)?
            .get(change)
            .cloned()
    }

    /// Also drops changes no longer in `live`.
    pub(super) fn record(
        &mut self,
        repository: &str,
        uploads: Vec<(String, GerritUpload)>,
        live: &HashSet<String>,
    ) {
        self.0.update(|state| {
            let changes = state.repositories.entry(repository.to_owned()).or_default();
            let before = changes.clone();
            changes.retain(|change, _| live.contains(change));
            changes.extend(uploads);
            let changed = *changes != before;
            if changes.is_empty() {
                state.repositories.remove(repository);
            }
            changed
        });
    }
}

impl Repo {
    pub(super) fn gerrit_repository_key(&self) -> String {
        normalize_repository_path(&self.primary_root())
            .to_string_lossy()
            .into_owned()
    }

    pub(crate) fn record_gerrit_upload(&self, summary: &str) {
        let Ok(url) = self.gerrit_push_url() else {
            return;
        };
        let repo = self.get_repo();
        let numbers = change_numbers(summary);
        let Ok(mut latest) = self.latest_patch_sets(&numbers) else {
            return;
        };
        let mut uploads = Vec::new();
        for number in numbers {
            let Some(patch_set) = latest.remove(&number) else {
                continue;
            };
            let Some(commit) = CommitId::try_from_hex(&patch_set.commit)
                .and_then(|id| self.commit(&repo, &id).ok())
            else {
                continue;
            };
            uploads.push((
                commit.change_id().hex(),
                GerritUpload {
                    url: url.clone(),
                    number,
                    commit: patch_set.commit,
                },
            ));
        }
        if uploads.is_empty() {
            return;
        }
        let Ok(live) = self.live_change_ids() else {
            return;
        };
        GerritUploadStore::load().record(&self.gerrit_repository_key(), uploads, &live);
    }

    /// JayJay's last upload of each change to the current Gerrit server, tagged with the caller's `T`.
    pub(super) fn gerrit_uploads<T>(
        &self,
        changes: impl IntoIterator<Item = (String, T)>,
    ) -> JayResult<Vec<(T, GerritUpload)>> {
        let mut store = GerritUploadStore::load();
        let repository = self.gerrit_repository_key();
        let mut push_url = None;
        let mut uploads = Vec::new();
        for (change, tag) in changes {
            let Some(upload) = store.get(&repository, &change) else {
                continue;
            };
            if push_url.is_none() {
                push_url = Some(self.gerrit_push_url()?);
            }
            if push_url.as_ref() == Some(&upload.url) {
                uploads.push((tag, upload));
            }
        }
        Ok(uploads)
    }

    pub(super) fn live_change_ids(&self) -> JayResult<HashSet<String>> {
        let repo = self.get_repo();
        self.revset_commits(&repo, "mutable()")?
            .iter()
            .map(|id| Ok(self.commit(&repo, id)?.change_id().hex()))
            .collect()
    }
}

fn change_numbers(summary: &str) -> Vec<u64> {
    let mut numbers: Vec<u64> = summary
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .filter_map(|url| url.rsplit_once("/+/"))
        .filter_map(|(_, number)| number.trim_end_matches('/').parse().ok())
        .collect();
    numbers.dedup();
    numbers
}

#[cfg(test)]
mod tests {
    use super::change_numbers;

    #[test]
    fn change_numbers_come_from_the_links_gerrit_prints() {
        let summary = "https://review.example.org/c/demo/+/12 Add hello [NEW]\n\
            https://review.example.org/c/demo/+/13/ Add world";
        assert_eq!(change_numbers(summary), [12, 13]);
    }
}
