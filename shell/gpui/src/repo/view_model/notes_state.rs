use jayjay_review::ReviewNoteStatus;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Default)]
pub struct NotesState {
    /// Every file's notes for the selected change, including resolved ones.
    pub all: Vec<ReviewNoteStatus>,
    /// Recomputed only where `all` is written, not on every render.
    pub(super) active_counts: Arc<HashMap<String, usize>>,
}
