use jayjay_primitives::{ChangeInfo, ReviewMode};

use crate::store::{ReviewStore, StoredNote};

impl ReviewStore {
    /// A stored choice wins; otherwise a change starts in review mode once it has marks or notes, so an agent's triage is never hidden.
    pub fn review_mode(&self, change: &ChangeInfo) -> ReviewMode {
        if !change.accepts_review_marks() {
            return ReviewMode::Unavailable;
        }
        if change.is_working_copy {
            return ReviewMode::WorkingCopy;
        }
        let change_id = change.change_id.as_str();
        let on = self
            .state
            .review_modes
            .get(change_id)
            .copied()
            .unwrap_or_else(|| self.has_review_state(change_id));
        if on { ReviewMode::On } else { ReviewMode::Off }
    }

    pub fn set_review_mode(&mut self, change_id: &str, on: bool) {
        self.mutate(|store| {
            store.state.review_modes.insert(change_id.to_owned(), on);
        })
    }

    /// A person's mark records review mode as chosen, so unmarking the last file does not drop the change out of review mode.
    pub(crate) fn mutate_marks<T>(
        &mut self,
        change_id: &str,
        mutate: impl FnOnce(&mut Self) -> T,
    ) -> T {
        self.mutate(|store| {
            store.keep_review_mode(change_id);
            mutate(store)
        })
    }

    pub(crate) fn keep_review_mode(&mut self, change_id: &str) {
        self.state
            .review_modes
            .entry(change_id.to_owned())
            .or_insert(true);
    }

    fn has_review_state(&self, change_id: &str) -> bool {
        let prefix = format!("{change_id}|");
        self.state
            .reviewed
            .keys()
            .any(|key| key.starts_with(&prefix))
            || self
                .state
                .notes
                .iter()
                .filter_map(StoredNote::parsed)
                .any(|note| note.change_id == change_id)
    }
}
