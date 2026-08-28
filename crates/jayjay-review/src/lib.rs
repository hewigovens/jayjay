mod anchor;
mod file_state;
mod marks;
mod note_store;
mod reconcile;
mod replay;
pub mod store;
#[cfg(any(test, feature = "test-util"))]
pub mod test_util;

pub use anchor::build_note_anchor;
pub use file_state::display_group_states;
pub use jayjay_primitives::{
    HunkType, JayJayError, NoteAnchor, NoteEntry, NoteSide, NoteStatus, ReviewDiffProvider,
    ReviewError, ReviewFileDiff, ReviewFileRollup, ReviewFileState, ReviewGroupState, ReviewHunk,
    ReviewNoteStatus, ReviewResult, ReviewStoreSummary,
};
pub use jj_diff::{ReviewFileSnapshot, ReviewGroupFingerprint};
pub use marks::ReviewFileMarks;
pub use reconcile::reconcile_notes;
pub use store::{IdSource, ReviewStore, UuidIdSource};

#[cfg(test)]
mod baseline_tests;
#[cfg(test)]
mod tests;
