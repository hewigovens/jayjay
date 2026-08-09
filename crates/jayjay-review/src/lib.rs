mod anchor;
mod marks;
mod note_store;
mod reconcile;
mod replay;
pub mod store;
#[cfg(any(test, feature = "test-util"))]
pub mod test_util;

pub use anchor::build_note_anchor;
pub use jayjay_primitives::{
    HunkType, JayJayError, NoteAnchor, NoteEntry, NoteSide, NoteStatus, ReviewDiffProvider,
    ReviewError, ReviewFileDiff, ReviewHunk, ReviewNoteStatus, ReviewResult,
};
pub use marks::ReviewFileMarks;
pub use reconcile::reconcile_notes;
pub use store::{IdSource, ReviewStore, UuidIdSource};

#[cfg(test)]
mod tests;
