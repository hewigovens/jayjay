use jayjay_primitives::{ReviewGroupState, ReviewMarkSource};
use jj_diff::ReviewFileSnapshot;

use crate::store::{ReviewEntryState, ReviewStore, key};

impl ReviewStore {
    /// An agent never takes over a group a person reviewed; a person takes the whole file.
    pub fn mark_reviewed_as(
        &mut self,
        change_id: &str,
        path: &str,
        identity: &str,
        snapshot: Option<&ReviewFileSnapshot>,
        source: ReviewMarkSource,
    ) -> std::io::Result<()> {
        self.mutate_checked(|store| {
            store.mark_as(change_id, path, identity, snapshot, None, source)
        })
    }

    /// Canonical group indices, so a snapshot is required.
    pub fn mark_groups_reviewed_as(
        &mut self,
        change_id: &str,
        path: &str,
        identity: &str,
        snapshot: &ReviewFileSnapshot,
        indices: &[u32],
        source: ReviewMarkSource,
    ) -> std::io::Result<()> {
        self.mutate_checked(|store| {
            store.mark_as(
                change_id,
                path,
                identity,
                Some(snapshot),
                Some(indices),
                source,
            )
        })
    }

    fn mark_as(
        &mut self,
        change_id: &str,
        path: &str,
        identity: &str,
        snapshot: Option<&ReviewFileSnapshot>,
        indices: Option<&[u32]>,
        source: ReviewMarkSource,
    ) {
        if identity.is_empty() {
            return;
        }
        let k = key(change_id, path);
        let previous = self.state.reviewed.get(&k).cloned();
        let prior = self.file_review_state(change_id, path, identity, snapshot);
        match indices {
            Some(indices) => {
                let Some(snapshot) = snapshot.filter(|s| !s.fingerprints.is_empty()) else {
                    return;
                };
                self.set_hunk_states(
                    change_id,
                    path,
                    identity,
                    &snapshot.fingerprints,
                    indices,
                    ReviewGroupState::Reviewed,
                );
            }
            None => self.mark_reviewed_snapshot(change_id, path, identity, snapshot),
        }
        // The writes above already record a person's ownership.
        if source == ReviewMarkSource::User {
            return;
        }
        let Some(entry) = self.state.reviewed.get_mut(&k) else {
            return;
        };
        entry.inherit_group_sources(previous.as_ref(), &[]);
        match &mut entry.state {
            ReviewEntryState::Groups { groups, .. } => {
                for (index, group) in groups.iter_mut().enumerate() {
                    if indices.is_none_or(|indices| indices.contains(&(index as u32))) {
                        let human_reviewed = prior.state_at(index as u32)
                            == ReviewGroupState::Reviewed
                            && !group.agent_owned();
                        group.source = (!human_reviewed).then_some(ReviewMarkSource::Agent);
                    }
                }
                entry.refresh_source();
            }
            _ => {
                if !prior.is_fully_reviewed()
                    || previous.is_some_and(|e| e.source == ReviewMarkSource::Agent)
                {
                    entry.source = ReviewMarkSource::Agent;
                }
            }
        }
    }

    pub fn unmark_owned_by(
        &mut self,
        change_id: &str,
        path: Option<&str>,
        source: ReviewMarkSource,
    ) -> std::io::Result<Vec<String>> {
        let cleared: Vec<String> = self
            .paths_owned_by(change_id, source)
            .into_iter()
            .filter(|p| path.is_none_or(|path| path == p))
            .collect();
        if cleared.is_empty() {
            return Ok(cleared);
        }
        let agent = source == ReviewMarkSource::Agent;
        self.mutate_checked(|store| {
            for path in &cleared {
                let k = key(change_id, path);
                let entry = store.state.reviewed.get_mut(&k).expect("listed entry");
                let mut agent_removed = entry.agent_removed().to_vec();
                let ReviewEntryState::Groups {
                    groups,
                    removed_reviewed,
                    ..
                } = &mut entry.state
                else {
                    store.state.reviewed.remove(&k);
                    continue;
                };
                for group in groups.iter_mut().filter(|g| g.agent_owned() == agent) {
                    group.state = ReviewGroupState::Unreviewed;
                    group.source = None;
                }
                removed_reviewed.retain(|digest| {
                    let agent_owned = agent_removed
                        .iter()
                        .position(|candidate| candidate == digest)
                        .map(|index| agent_removed.remove(index))
                        .is_some();
                    agent_owned != agent
                });
                let keep = groups
                    .iter()
                    .any(|g| g.state != ReviewGroupState::Unreviewed)
                    || !removed_reviewed.is_empty();
                if !keep {
                    store.state.reviewed.remove(&k);
                    continue;
                }
                if agent {
                    entry.agent_removed_reviewed = None;
                }
                entry.refresh_source();
            }
            cleared
        })
    }

    // A failed save must not leave the mutation in memory for the next write.
    fn mutate_checked<T>(&mut self, mutate: impl FnOnce(&mut Self) -> T) -> std::io::Result<T> {
        let mut staged = Self::from_state(self.state.clone());
        let result = mutate(&mut staged);
        let previous = std::mem::replace(&mut self.state, staged.state);
        if let Err(error) = self.save_checked() {
            self.state = previous;
            return Err(error);
        }
        Ok(result)
    }
}
