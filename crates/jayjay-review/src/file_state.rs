use std::collections::{HashMap, HashSet};

use jayjay_primitives::{ReviewFileState, ReviewGroupState};
use jj_diff::{REVIEW_FINGERPRINT_VERSION, ReviewGroupFingerprint};

use crate::store::{ReviewEntry, ReviewEntryState, StoredReviewGroup};

pub fn display_group_states(
    canonical: &ReviewFileState,
    mapping: &[Vec<u32>],
) -> Vec<ReviewGroupState> {
    mapping
        .iter()
        .map(|indices| combine_mapped_states(canonical, indices))
        .collect()
}

fn combine_mapped_states(canonical: &ReviewFileState, indices: &[u32]) -> ReviewGroupState {
    if indices.is_empty() {
        return ReviewGroupState::Unreviewed;
    }
    let mut saw_reviewed = false;
    let mut saw_unreviewed = false;
    for index in indices {
        match canonical.state_at(*index) {
            ReviewGroupState::ChangedSinceReview => return ReviewGroupState::ChangedSinceReview,
            ReviewGroupState::Reviewed => saw_reviewed = true,
            ReviewGroupState::Unreviewed => saw_unreviewed = true,
        }
    }
    if saw_reviewed && !saw_unreviewed {
        ReviewGroupState::Reviewed
    } else {
        ReviewGroupState::Unreviewed
    }
}

pub(crate) fn reconcile(
    reviewed: Option<&ReviewEntry>,
    current_identity: &str,
    current: Option<&[ReviewGroupFingerprint]>,
) -> ReviewFileState {
    let Some(entry) = reviewed else {
        return unreviewed(current);
    };
    match &entry.state {
        ReviewEntryState::File => reconcile_file(entry, current_identity, current),
        ReviewEntryState::Hunks { indices } => {
            reconcile_hunks(entry, indices, current_identity, current)
        }
        ReviewEntryState::Groups {
            algorithm_version,
            groups,
            removed_reviewed,
        } => reconcile_groups(
            entry,
            *algorithm_version,
            groups,
            removed_reviewed,
            current_identity,
            current,
        ),
    }
}

fn unreviewed(current: Option<&[ReviewGroupFingerprint]>) -> ReviewFileState {
    ReviewFileState::filled(
        ReviewGroupState::Unreviewed,
        current.map(|groups| groups.len()).unwrap_or(0),
        0,
    )
}

fn reconcile_file(
    entry: &ReviewEntry,
    current_identity: &str,
    current: Option<&[ReviewGroupFingerprint]>,
) -> ReviewFileState {
    if entry.identity != current_identity {
        return match current {
            Some(current) if !current.is_empty() => conservative_changed(current, 0),
            Some(_) | None => ReviewFileState::whole_file(false, 1),
        };
    }
    ReviewFileState::fully_reviewed(current.map(|groups| groups.len()).unwrap_or(0))
}

fn reconcile_hunks(
    entry: &ReviewEntry,
    indices: &[u32],
    current_identity: &str,
    current: Option<&[ReviewGroupFingerprint]>,
) -> ReviewFileState {
    if entry.identity != current_identity {
        return unreviewed(current);
    }
    ReviewFileState::from_groups(
        match current {
            Some(current) => current
                .iter()
                .enumerate()
                .map(|(index, _)| {
                    if indices.contains(&(index as u32)) {
                        ReviewGroupState::Reviewed
                    } else {
                        ReviewGroupState::Unreviewed
                    }
                })
                .collect(),
            None => Vec::new(),
        },
        0,
    )
}

fn reconcile_groups(
    entry: &ReviewEntry,
    algorithm_version: u32,
    groups: &[StoredReviewGroup],
    removed_reviewed: &[String],
    current_identity: &str,
    current: Option<&[ReviewGroupFingerprint]>,
) -> ReviewFileState {
    let version_ok = algorithm_version == REVIEW_FINGERPRINT_VERSION;
    if current_identity == entry.identity && version_ok {
        return fast_path(groups, removed_reviewed, current);
    }
    let Some(current) = current else {
        return identity_mismatch_without_snapshot(groups, removed_reviewed);
    };
    if !version_ok {
        return conservative_changed(current, reviewed_prior_count(groups, removed_reviewed));
    }
    match_fingerprints(groups, removed_reviewed, current)
}

fn fast_path(
    groups: &[StoredReviewGroup],
    removed_reviewed: &[String],
    current: Option<&[ReviewGroupFingerprint]>,
) -> ReviewFileState {
    let group_states: Vec<ReviewGroupState> = groups.iter().map(|group| group.state).collect();
    if group_states.is_empty() {
        return match current {
            Some(current) if !current.is_empty() => ReviewFileState::fully_reviewed(current.len()),
            _ => identity_only_reviewed(removed_reviewed),
        };
    }
    ReviewFileState::from_groups(group_states, removed_reviewed.len() as u32)
}

fn identity_only_reviewed(removed_reviewed: &[String]) -> ReviewFileState {
    ReviewFileState::whole_file(removed_reviewed.is_empty(), removed_reviewed.len() as u32)
}

fn identity_mismatch_without_snapshot(
    groups: &[StoredReviewGroup],
    removed_reviewed: &[String],
) -> ReviewFileState {
    let had_reviewed = groups.is_empty()
        || groups
            .iter()
            .any(|group| group.state == ReviewGroupState::Reviewed)
        || !removed_reviewed.is_empty();
    ReviewFileState::from_groups(
        Vec::new(),
        if had_reviewed {
            1.max(removed_reviewed.len() as u32)
        } else {
            0
        },
    )
}

fn conservative_changed(
    current: &[ReviewGroupFingerprint],
    removed_reviewed_count: u32,
) -> ReviewFileState {
    ReviewFileState::filled(
        ReviewGroupState::ChangedSinceReview,
        current.len(),
        removed_reviewed_count,
    )
}

fn reviewed_prior_count(groups: &[StoredReviewGroup], removed_reviewed: &[String]) -> u32 {
    let current = groups
        .iter()
        .filter(|group| group.state == ReviewGroupState::Reviewed)
        .count() as u32;
    current + removed_reviewed.len() as u32
}

fn match_fingerprints(
    groups: &[StoredReviewGroup],
    removed_reviewed: &[String],
    current: &[ReviewGroupFingerprint],
) -> ReviewFileState {
    let mut prior: Vec<(&str, ReviewGroupState)> = groups
        .iter()
        .map(|group| (group.digest.as_str(), group.state))
        .collect();
    for digest in removed_reviewed {
        prior.push((digest, ReviewGroupState::Reviewed));
    }

    let prior_counts = count_digests(prior.iter().map(|(digest, _)| *digest));
    let current_counts = count_digests(current.iter().map(|fp| fp.digest.as_str()));
    let mut matched_prior = HashSet::new();
    let mut group_states = Vec::with_capacity(current.len());

    for fingerprint in current {
        let digest = fingerprint.digest.as_str();
        let unique = current_counts.get(digest) == Some(&1) && prior_counts.get(digest) == Some(&1);
        if unique {
            let state = prior
                .iter()
                .find(|(candidate, _)| *candidate == digest)
                .map(|(_, state)| *state)
                .unwrap_or(ReviewGroupState::ChangedSinceReview);
            group_states.push(state);
            matched_prior.insert(digest);
        } else {
            group_states.push(ReviewGroupState::ChangedSinceReview);
        }
    }

    let removed_reviewed_count = prior
        .iter()
        .filter(|(digest, state)| {
            *state == ReviewGroupState::Reviewed && !matched_prior.contains(*digest)
        })
        .count() as u32;

    ReviewFileState::from_groups(group_states, removed_reviewed_count)
}

fn count_digests<'a>(digests: impl Iterator<Item = &'a str>) -> HashMap<&'a str, usize> {
    let mut counts = HashMap::new();
    for digest in digests {
        *counts.entry(digest).or_insert(0) += 1;
    }
    counts
}

pub(crate) fn persisted_entry(
    identity: &str,
    fingerprints: &[ReviewGroupFingerprint],
    state: &ReviewFileState,
    removed_reviewed: Vec<String>,
) -> ReviewEntry {
    if fingerprints.is_empty() && removed_reviewed.is_empty() {
        return if state.is_fully_reviewed() {
            ReviewEntry::file(identity)
        } else {
            ReviewEntry::hunks(identity, state.reviewed_indices())
        };
    }
    let groups = fingerprints
        .iter()
        .zip(state.group_states())
        .map(|(fingerprint, group_state)| StoredReviewGroup {
            digest: fingerprint.digest.clone(),
            state: *group_state,
        })
        .collect();
    ReviewEntry::new(
        identity,
        ReviewEntryState::Groups {
            algorithm_version: REVIEW_FINGERPRINT_VERSION,
            groups,
            removed_reviewed,
        },
    )
}

pub(crate) fn unmatched_reviewed_digests(
    reviewed: Option<&ReviewEntry>,
    current_identity: &str,
    current: &[ReviewGroupFingerprint],
) -> Vec<String> {
    let Some(entry) = reviewed else {
        return Vec::new();
    };
    let ReviewEntryState::Groups {
        groups,
        removed_reviewed,
        ..
    } = &entry.state
    else {
        return Vec::new();
    };
    // Same identity means same bytes: stored groups still correspond to current groups by index, so digest matching could only invent removals.
    if entry.identity == current_identity {
        return removed_reviewed.clone();
    }
    let prior_counts = count_digests(
        groups
            .iter()
            .map(|group| group.digest.as_str())
            .chain(removed_reviewed.iter().map(String::as_str)),
    );
    let current_counts = count_digests(current.iter().map(|fp| fp.digest.as_str()));
    let uniquely_matched: HashSet<&str> = current
        .iter()
        .filter(|fingerprint| {
            prior_counts.get(fingerprint.digest.as_str()) == Some(&1)
                && current_counts.get(fingerprint.digest.as_str()) == Some(&1)
        })
        .map(|fingerprint| fingerprint.digest.as_str())
        .collect();
    let mut removed = Vec::new();
    for group in groups {
        if group.state == ReviewGroupState::Reviewed
            && !uniquely_matched.contains(group.digest.as_str())
        {
            removed.push(group.digest.clone());
        }
    }
    for digest in removed_reviewed {
        if !uniquely_matched.contains(digest.as_str()) {
            removed.push(digest.clone());
        }
    }
    removed.sort();
    removed
}
