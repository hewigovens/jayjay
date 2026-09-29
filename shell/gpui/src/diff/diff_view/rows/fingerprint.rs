use std::hash::{Hash, Hasher};

use jayjay_review::ReviewNoteStatus;

/// Cache key for `DiffWrapCache::rows`; changes on any mutation and on external reloads that flip a reconciled status.
pub(crate) fn notes_fingerprint(notes: &[ReviewNoteStatus]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    notes.len().hash(&mut hasher);
    for status in notes {
        status.note.id.hash(&mut hasher);
        status.note.updated_at_ms.hash(&mut hasher);
        status.note.resolved.hash(&mut hasher);
        status.status.as_str().hash(&mut hasher);
        status.group_index.hash(&mut hasher);
    }
    hasher.finish()
}
