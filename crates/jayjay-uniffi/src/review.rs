use std::sync::{Arc, Mutex};

use jayjay_primitives::{NoteAnchor, NoteEntry};
use jayjay_review::{ReviewFileMarks, ReviewStore};

use crate::JayJayError;

#[derive(uniffi::Object)]
pub struct ReviewSession {
    store: Mutex<ReviewStore>,
}

impl ReviewSession {
    fn store(&self) -> Result<std::sync::MutexGuard<'_, ReviewStore>, JayJayError> {
        self.store.lock().map_err(lock_error)
    }
}

#[uniffi::export]
impl ReviewSession {
    #[uniffi::constructor]
    fn new(snapshot: Option<String>) -> Result<Arc<Self>, JayJayError> {
        let store = match snapshot {
            Some(snapshot) => ReviewStore::in_memory_from_json(&snapshot).map_err(review_error)?,
            None => ReviewStore::in_memory(),
        };
        Ok(Arc::new(Self {
            store: Mutex::new(store),
        }))
    }

    fn snapshot(&self) -> Result<String, JayJayError> {
        self.store()?.snapshot_json().map_err(review_error)
    }

    fn is_reviewed(
        &self,
        change_id: String,
        path: String,
        identity: String,
    ) -> Result<bool, JayJayError> {
        Ok(self.store()?.is_reviewed(&change_id, &path, &identity))
    }

    fn file_marks(
        &self,
        change_id: String,
        path: String,
        identity: String,
    ) -> Result<ReviewFileMarks, JayJayError> {
        Ok(self.store()?.file_marks(&change_id, &path, &identity, None))
    }

    fn mark_reviewed(
        &self,
        change_id: String,
        path: String,
        identity: String,
    ) -> Result<(), JayJayError> {
        self.store()?.mark_reviewed(&change_id, &path, &identity);
        Ok(())
    }

    fn mark_unreviewed(&self, change_id: String, path: String) -> Result<(), JayJayError> {
        self.store()?.mark_unreviewed(&change_id, &path);
        Ok(())
    }

    fn toggle_reviewed(
        &self,
        change_id: String,
        path: String,
        identity: String,
    ) -> Result<(), JayJayError> {
        self.store()?.toggle(&change_id, &path, &identity);
        Ok(())
    }

    fn reviewed_paths(
        &self,
        change_id: String,
        paths: Vec<String>,
        identities: Vec<String>,
    ) -> Result<Vec<String>, JayJayError> {
        let store = self.store()?;
        Ok(paths
            .into_iter()
            .zip(identities)
            .filter_map(|(path, identity)| {
                store
                    .is_reviewed(&change_id, &path, &identity)
                    .then_some(path)
            })
            .collect())
    }

    fn is_hunk_reviewed(
        &self,
        change_id: String,
        path: String,
        identity: String,
        hunk_index: u32,
    ) -> Result<bool, JayJayError> {
        Ok(self
            .store()?
            .is_hunk_reviewed(&change_id, &path, &identity, hunk_index))
    }

    fn mark_hunk_reviewed(
        &self,
        change_id: String,
        path: String,
        identity: String,
        hunk_index: u32,
    ) -> Result<(), JayJayError> {
        self.store()?
            .mark_hunk_reviewed(&change_id, &path, &identity, hunk_index);
        Ok(())
    }

    fn mark_hunk_unreviewed(
        &self,
        change_id: String,
        path: String,
        hunk_index: u32,
    ) -> Result<(), JayJayError> {
        self.store()?
            .mark_hunk_unreviewed(&change_id, &path, hunk_index);
        Ok(())
    }

    fn toggle_hunk(
        &self,
        change_id: String,
        path: String,
        identity: String,
        hunk_index: u32,
    ) -> Result<(), JayJayError> {
        self.store()?
            .toggle_hunk(&change_id, &path, &identity, hunk_index);
        Ok(())
    }

    fn set_reviewed_hunks(
        &self,
        change_id: String,
        path: String,
        identity: String,
        hunk_indices: Vec<u32>,
    ) -> Result<(), JayJayError> {
        self.store()?
            .set_reviewed_hunks(&change_id, &path, &identity, hunk_indices);
        Ok(())
    }

    fn clear_change(&self, change_id: String) -> Result<(), JayJayError> {
        self.store()?.clear_change(&change_id);
        Ok(())
    }

    fn list_notes(
        &self,
        change_id: String,
        include_resolved: bool,
    ) -> Result<Vec<NoteEntry>, JayJayError> {
        Ok(self.store()?.list_notes(&change_id, include_resolved))
    }

    fn add_note(&self, anchor: NoteAnchor, body: String) -> Result<NoteEntry, JayJayError> {
        Ok(self.store()?.add_note(anchor, &body))
    }

    fn update_note(&self, id: String, body: String) -> Result<Option<NoteEntry>, JayJayError> {
        Ok(self.store()?.update_note(&id, &body))
    }

    fn delete_note(&self, id: String) -> Result<bool, JayJayError> {
        Ok(self.store()?.delete_note(&id))
    }

    fn resolve_note(&self, id: String) -> Result<Option<NoteEntry>, JayJayError> {
        Ok(self.store()?.resolve_note(&id))
    }
}

fn lock_error<T>(_: std::sync::PoisonError<T>) -> JayJayError {
    JayJayError::Review {
        message: "review session is unavailable".to_owned(),
    }
}

fn review_error(error: impl std::fmt::Display) -> JayJayError {
    JayJayError::Review {
        message: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_restores_marks() {
        let session = ReviewSession::new(None).unwrap();
        session
            .mark_hunk_reviewed("change".into(), "src/lib.rs".into(), "identity".into(), 2)
            .unwrap();

        let restored = ReviewSession::new(Some(session.snapshot().unwrap())).unwrap();
        assert!(
            restored
                .is_hunk_reviewed("change".into(), "src/lib.rs".into(), "identity".into(), 2,)
                .unwrap()
        );
    }
}
