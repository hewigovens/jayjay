use jayjay_review::{NoteSide, NoteStatus};

use super::super::notes_fingerprint;
use super::fixtures::note;

#[test]
fn notes_fingerprint_changes_when_a_note_updates_and_is_stable_otherwise() {
    let base = vec![note(
        "n1",
        NoteSide::New,
        2,
        "body",
        NoteStatus::Current,
        false,
    )];
    let same = vec![note(
        "n1",
        NoteSide::New,
        2,
        "body",
        NoteStatus::Current,
        false,
    )];
    assert_eq!(notes_fingerprint(&base), notes_fingerprint(&same));

    let mut resolved = base.clone();
    resolved[0].note.resolved = true;
    resolved[0].status = NoteStatus::Resolved;
    assert_ne!(notes_fingerprint(&base), notes_fingerprint(&resolved));

    let mut edited = base.clone();
    edited[0].note.updated_at_ms = 1;
    assert_ne!(notes_fingerprint(&base), notes_fingerprint(&edited));
}
