use jayjay_primitives::{ChangeInfo, CommitAuthor, NewChangeEligibility, ShortId};

use super::*;

fn change(change_id: &str) -> ChangeInfo {
    ChangeInfo {
        change_id: ShortId::new(change_id.to_owned(), 1),
        commit_id: ShortId::new(format!("{change_id}-commit"), 1),
        description: String::new(),
        author: CommitAuthor::empty(0),
        co_authors: Vec::new(),
        committed_at_millis: 0,
        parents: Vec::new(),
        bookmarks: Vec::new(),
        tags: Vec::new(),
        workspaces: Vec::new(),
        is_working_copy: false,
        has_conflict: false,
        is_empty: false,
        is_immutable: false,
        is_divergent: false,
        has_immutable_parent: false,
        new_change: NewChangeEligibility {
            on_top: true,
            before: true,
            after: true,
        },
    }
}

#[test]
fn review_mode_starts_on_only_for_changes_with_marks_or_notes() {
    let mut store = note_store();
    store
        .mark_reviewed_as("marked", "a.txt", "id", None, ReviewMarkSource::Agent)
        .unwrap();
    store.add_note(anchor(), "check this");

    assert_eq!(store.review_mode(&change("plain")), ReviewMode::Off);
    assert_eq!(store.review_mode(&change("marked")), ReviewMode::On);
    assert_eq!(store.review_mode(&change("c1")), ReviewMode::On);
}

#[test]
fn explicit_review_mode_overrides_the_default_both_ways() {
    let mut store = make_store();
    store.mark_reviewed("marked", "a.txt", "id");
    store.set_review_mode("marked", false);
    store.set_review_mode("plain", true);

    assert_eq!(store.review_mode(&change("marked")), ReviewMode::Off);
    assert_eq!(store.review_mode(&change("plain")), ReviewMode::On);
}

#[test]
fn working_copy_and_immutable_changes_ignore_review_mode() {
    let mut store = make_store();
    store.set_review_mode("wc", false);
    store.set_review_mode("old", true);
    let mut working_copy = change("wc");
    working_copy.is_working_copy = true;
    let mut immutable = change("old");
    immutable.is_immutable = true;

    assert_eq!(store.review_mode(&working_copy), ReviewMode::WorkingCopy);
    assert_eq!(store.review_mode(&immutable), ReviewMode::Unavailable);
}

#[test]
fn only_a_persons_marks_keep_review_mode_on_after_unmarking() {
    let mut store = make_store();
    store.mark_reviewed("person", "a.txt", "id");
    store.mark_unreviewed("person", "a.txt");
    store
        .mark_reviewed_as("agent", "a.txt", "id", None, ReviewMarkSource::Agent)
        .unwrap();
    store
        .unmark_owned_by("agent", None, ReviewMarkSource::Agent)
        .unwrap();

    assert_eq!(store.review_mode(&change("person")), ReviewMode::On);
    assert_eq!(store.review_mode(&change("agent")), ReviewMode::Off);
}

#[test]
fn clearing_or_pruning_a_change_drops_its_review_mode() {
    let mut store = make_store();
    store.set_review_mode("committed", true);
    store.set_review_mode("landed", true);
    store.set_review_mode("kept", true);

    store.clear_change("committed");
    store.prune(|change_id| change_id != "landed").unwrap();

    assert_eq!(store.review_mode(&change("committed")), ReviewMode::Off);
    assert_eq!(store.review_mode(&change("landed")), ReviewMode::Off);
    assert_eq!(store.review_mode(&change("kept")), ReviewMode::On);
}
