use std::collections::{HashMap, HashSet};

use jayjay_primitives::{ReviewFileState, ReviewGroupState, hex_sha256};
use jj_diff::{REVIEW_FINGERPRINT_VERSION, ReviewGroupFingerprint};

use crate::store::{BASELINE_SCHEMA_VERSION, ReviewBaseline, ReviewEntry, StoredBaselineGroup};

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

pub(crate) fn mirror_digest(entry: &ReviewEntry) -> String {
    let mut hunks = entry.hunks.clone();
    hunks.sort_unstable();
    hex_sha256(
        format!(
            "{}\0{}\0{}",
            entry.identity,
            u8::from(entry.file_marked),
            hunks
                .iter()
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join(",")
        )
        .as_bytes(),
    )
}

pub(crate) fn reconcile(
    reviewed: Option<&ReviewEntry>,
    baseline: Option<&ReviewBaseline>,
    current_identity: &str,
    current: Option<&[ReviewGroupFingerprint]>,
) -> ReviewFileState {
    let Some(entry) = reviewed else {
        return unreviewed(current);
    };
    if let Some(baseline) = trusted_baseline(entry, baseline) {
        return reconcile_baseline(baseline, current_identity, current);
    }
    reconcile_legacy(entry, current_identity, current)
}

fn unreviewed(current: Option<&[ReviewGroupFingerprint]>) -> ReviewFileState {
    ReviewFileState::filled(
        ReviewGroupState::Unreviewed,
        current.map(|groups| groups.len()).unwrap_or(0),
        0,
    )
}

fn trusted_baseline<'a>(
    entry: &ReviewEntry,
    baseline: Option<&'a ReviewBaseline>,
) -> Option<&'a ReviewBaseline> {
    let baseline = baseline?;
    if baseline.mirror_digest.is_empty() {
        return None;
    }
    (baseline.mirror_digest == mirror_digest(entry)).then_some(baseline)
}

fn reconcile_baseline(
    baseline: &ReviewBaseline,
    current_identity: &str,
    current: Option<&[ReviewGroupFingerprint]>,
) -> ReviewFileState {
    let version_ok = baseline.algorithm_version == REVIEW_FINGERPRINT_VERSION;
    if current_identity == baseline.identity && version_ok {
        return fast_path(baseline, current);
    }
    let Some(current) = current else {
        return identity_mismatch_without_snapshot(baseline);
    };
    if !version_ok {
        return conservative_changed(current, reviewed_prior_count(baseline));
    }
    match_fingerprints(baseline, current)
}

fn fast_path(
    baseline: &ReviewBaseline,
    current: Option<&[ReviewGroupFingerprint]>,
) -> ReviewFileState {
    let group_states: Vec<ReviewGroupState> =
        baseline.groups.iter().map(|group| group.state).collect();
    if group_states.is_empty() {
        return match current {
            Some(current) if !current.is_empty() => ReviewFileState::fully_reviewed(current.len()),
            _ => identity_only_reviewed(baseline),
        };
    }
    if let Some(current) = current
        && current.len() != group_states.len()
    {
        return match_fingerprints(baseline, current);
    }
    ReviewFileState::from_groups(group_states, baseline.removed_reviewed.len() as u32)
}

fn identity_only_reviewed(baseline: &ReviewBaseline) -> ReviewFileState {
    ReviewFileState {
        group_states: Vec::new(),
        removed_reviewed_count: baseline.removed_reviewed.len() as u32,
        file_marked: baseline.removed_reviewed.is_empty(),
    }
}

fn identity_mismatch_without_snapshot(baseline: &ReviewBaseline) -> ReviewFileState {
    let had_reviewed = baseline.groups.is_empty()
        || baseline
            .groups
            .iter()
            .any(|group| group.state == ReviewGroupState::Reviewed)
        || !baseline.removed_reviewed.is_empty();
    ReviewFileState::from_groups(
        Vec::new(),
        if had_reviewed {
            1.max(baseline.removed_reviewed.len() as u32)
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

fn reviewed_prior_count(baseline: &ReviewBaseline) -> u32 {
    let current = baseline
        .groups
        .iter()
        .filter(|group| group.state == ReviewGroupState::Reviewed)
        .count() as u32;
    current + baseline.removed_reviewed.len() as u32
}

fn match_fingerprints(
    baseline: &ReviewBaseline,
    current: &[ReviewGroupFingerprint],
) -> ReviewFileState {
    let mut prior: Vec<(&str, ReviewGroupState)> = baseline
        .groups
        .iter()
        .map(|group| (group.digest.as_str(), group.state))
        .collect();
    for digest in &baseline.removed_reviewed {
        if prior
            .iter()
            .all(|(existing, _)| *existing != digest.as_str())
        {
            prior.push((digest, ReviewGroupState::Reviewed));
        }
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

fn reconcile_legacy(
    entry: &ReviewEntry,
    current_identity: &str,
    current: Option<&[ReviewGroupFingerprint]>,
) -> ReviewFileState {
    if entry.identity != current_identity {
        return unreviewed(current);
    }
    if entry.file_marked {
        return ReviewFileState::fully_reviewed(current.map(|groups| groups.len()).unwrap_or(0));
    }
    ReviewFileState::from_groups(
        match current {
            Some(current) => current
                .iter()
                .enumerate()
                .map(|(index, _)| {
                    if entry.hunks.contains(&(index as u32)) {
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

pub(crate) fn persist_reconciled(
    identity: &str,
    fingerprints: &[ReviewGroupFingerprint],
    state: &ReviewFileState,
    removed_reviewed: Vec<String>,
) -> (ReviewEntry, ReviewBaseline) {
    let file_marked = state.is_fully_reviewed();
    let hunks = if file_marked {
        Vec::new()
    } else {
        state.reviewed_indices()
    };
    let entry = ReviewEntry {
        identity: identity.to_string(),
        file_marked,
        hunks,
        extra: serde_json::Map::new(),
    };
    let groups = fingerprints
        .iter()
        .zip(state.group_states.iter())
        .map(|(fingerprint, group_state)| StoredBaselineGroup {
            digest: fingerprint.digest.clone(),
            state: *group_state,
            extra: serde_json::Map::new(),
        })
        .collect();
    let baseline = ReviewBaseline {
        schema_version: BASELINE_SCHEMA_VERSION,
        algorithm_version: fingerprints
            .first()
            .map(|fingerprint| fingerprint.algorithm_version)
            .unwrap_or(REVIEW_FINGERPRINT_VERSION),
        identity: identity.to_string(),
        groups,
        removed_reviewed,
        mirror_digest: mirror_digest(&entry),
        extra: serde_json::Map::new(),
    };
    (entry, baseline)
}

pub(crate) fn unmatched_reviewed_digests(
    reviewed: Option<&ReviewEntry>,
    baseline: Option<&ReviewBaseline>,
    current: &[ReviewGroupFingerprint],
    group_states: &[ReviewGroupState],
) -> Vec<String> {
    let Some(baseline) = reviewed.and_then(|entry| trusted_baseline(entry, baseline)) else {
        return Vec::new();
    };
    let counts = count_digests(current.iter().map(|fp| fp.digest.as_str()));
    let current_unique: HashSet<&str> = current
        .iter()
        .zip(group_states)
        .filter(|(fp, _)| counts.get(fp.digest.as_str()) == Some(&1))
        .map(|(fp, _)| fp.digest.as_str())
        .collect();
    let mut removed = Vec::new();
    for group in &baseline.groups {
        if group.state == ReviewGroupState::Reviewed
            && !current_unique.contains(group.digest.as_str())
        {
            removed.push(group.digest.clone());
        }
    }
    for digest in &baseline.removed_reviewed {
        if !current_unique.contains(digest.as_str()) {
            removed.push(digest.clone());
        }
    }
    removed.sort();
    removed.dedup();
    removed
}

pub(crate) fn copy_group_extras(
    existing: &[StoredBaselineGroup],
    groups: &mut [StoredBaselineGroup],
) {
    let mut used = vec![false; existing.len()];
    for group in groups {
        if let Some(index) = existing
            .iter()
            .enumerate()
            .position(|(i, candidate)| !used[i] && candidate.digest == group.digest)
        {
            used[index] = true;
            group.extra = existing[index].extra.clone();
        }
    }
}
