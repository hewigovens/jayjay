use std::io;

use jayjay_primitives::ReviewStoreSummary;

use super::persist::ReviewStore;

impl ReviewStore {
    /// Marks and notes whose change `keep` rejects; unparseable notes from newer versions are always kept.
    pub fn prunable(&self, keep: impl Fn(&str) -> bool) -> ReviewStoreSummary {
        ReviewStoreSummary {
            marks: self
                .state
                .reviewed
                .keys()
                .filter(|key| !keep(change_of(key)))
                .count() as u32,
            notes: self
                .state
                .notes
                .iter()
                .filter(|note| note.parsed().is_some_and(|note| !keep(&note.change_id)))
                .count() as u32,
        }
    }

    /// Drops what `prunable` reports under the store lock and returns what was removed.
    pub fn prune(&mut self, keep: impl Fn(&str) -> bool) -> io::Result<ReviewStoreSummary> {
        self.transaction(|store| {
            let removed = store.prunable(&keep);
            store.state.reviewed.retain(|key, _| keep(change_of(key)));
            store
                .state
                .notes
                .retain(|note| note.parsed().is_none_or(|note| keep(&note.change_id)));
            Ok(removed)
        })
    }
}

fn change_of(key: &str) -> &str {
    key.split_once('|').map_or(key, |(change_id, _)| change_id)
}
