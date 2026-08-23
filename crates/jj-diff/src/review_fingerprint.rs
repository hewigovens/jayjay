use std::collections::HashSet;

use jayjay_primitives::hex_sha256;

use super::change_groups::change_group_ranges;
use super::compute::{compute_file_diff, compute_file_diff_full_plain};
use super::conflicts::build_diff_display_lines;
use super::types::{DiffLine, DiffSpanStyle};

pub const REVIEW_FINGERPRINT_VERSION: u32 = 1;
const REVIEW_FINGERPRINT_CONTEXT_LINES: usize = 3;

/// Stable identity for one canonical contiguous changed-line group.
///
/// Digests hash changed-line payload (side, exact source text, missing-final-newline) plus a bounded amount of surrounding unchanged context. They omit group index, absolute line numbers, highlighting, wrapping, and collapse.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReviewGroupFingerprint {
    pub algorithm_version: u32,
    pub digest: String,
}

/// Canonical group fingerprints for a text file pair, in change-group order.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReviewFileSnapshot {
    pub algorithm_version: u32,
    pub fingerprints: Vec<ReviewGroupFingerprint>,
}

impl ReviewFileSnapshot {
    pub fn empty() -> Self {
        Self {
            algorithm_version: REVIEW_FINGERPRINT_VERSION,
            fingerprints: Vec::new(),
        }
    }

    pub fn digests(&self) -> Vec<&str> {
        self.fingerprints
            .iter()
            .map(|fingerprint| fingerprint.digest.as_str())
            .collect()
    }
}

/// Exact-whitespace, uncollapsed, unhighlighted snapshot used for persisted review identity.
pub fn canonical_review_snapshot(old: &str, new: &str) -> ReviewFileSnapshot {
    let diff = compute_file_diff_full_plain("", old, new, false);
    fingerprint_snapshot(old, new, &diff.lines)
}

pub fn review_group_fingerprints(
    old: &str,
    new: &str,
    lines: &[DiffLine],
) -> Vec<ReviewGroupFingerprint> {
    fingerprint_snapshot(old, new, lines).fingerprints
}

/// For each display change group, the canonical exact-whitespace group indices it overlaps.
///
/// Ignore-whitespace display can hide or split canonical groups. An empty inner list means the display group did not uniquely overlap any canonical group.
pub fn display_group_canonical_indices(
    old: &str,
    new: &str,
    ignore_whitespace: bool,
) -> Vec<Vec<u32>> {
    let canonical = compute_file_diff_full_plain("", old, new, false);
    let display = compute_file_diff("", old, new, ignore_whitespace);
    let display_lines = build_diff_display_lines(&display.lines);
    map_display_groups_to_canonical(&canonical.lines, &display_lines)
}

pub fn map_display_groups_to_canonical(
    canonical_lines: &[DiffLine],
    display_lines: &[DiffLine],
) -> Vec<Vec<u32>> {
    let canonical_keys: Vec<HashSet<ChangedLineKey>> = change_group_ranges(canonical_lines)
        .into_iter()
        .map(|(_, start, end)| changed_line_keys(canonical_lines, start, end))
        .collect();
    change_group_ranges(display_lines)
        .into_iter()
        .map(|(_, start, end)| {
            let display_keys = changed_line_keys(display_lines, start, end);
            canonical_keys
                .iter()
                .enumerate()
                .filter(|(_, keys)| !keys.is_disjoint(&display_keys))
                .map(|(index, _)| index as u32)
                .collect()
        })
        .collect()
}

fn fingerprint_snapshot(old: &str, new: &str, lines: &[DiffLine]) -> ReviewFileSnapshot {
    let old_lines = SourceLines::from_text(old);
    let new_lines = SourceLines::from_text(new);
    let fingerprints = change_group_ranges(lines)
        .into_iter()
        .map(|(_, start, end)| fingerprint_group(lines, start, end, &old_lines, &new_lines))
        .collect();
    ReviewFileSnapshot {
        algorithm_version: REVIEW_FINGERPRINT_VERSION,
        fingerprints,
    }
}

fn fingerprint_group(
    lines: &[DiffLine],
    start: usize,
    end: usize,
    old_lines: &SourceLines<'_>,
    new_lines: &SourceLines<'_>,
) -> ReviewGroupFingerprint {
    let mut encoded = Vec::new();
    encoded.extend_from_slice(b"jayjay-review-group-v1");
    encoded.push(0);
    encoded.extend_from_slice(&REVIEW_FINGERPRINT_VERSION.to_le_bytes());
    encoded.extend_from_slice(b"\0payload\0");
    for line in &lines[start..=end] {
        if !line.is_changed() {
            continue;
        }
        let added = line.style == DiffSpanStyle::Added;
        encoded.push(if added { b'+' } else { b'-' });
        let text = if added {
            new_lines.get(line.new_line_no)
        } else {
            old_lines.get(line.old_line_no)
        };
        encoded.extend_from_slice(&(text.len() as u32).to_le_bytes());
        encoded.extend_from_slice(text.as_bytes());
        encoded.push(if line.no_eof_newline { 1 } else { 0 });
    }
    encoded.extend_from_slice(b"\0before\0");
    encode_context(
        &mut encoded,
        context_before(lines, start, old_lines, new_lines),
    );
    encoded.extend_from_slice(b"\0after\0");
    encode_context(
        &mut encoded,
        context_after(lines, end, old_lines, new_lines),
    );
    ReviewGroupFingerprint {
        algorithm_version: REVIEW_FINGERPRINT_VERSION,
        digest: hex_sha256(&encoded),
    }
}

fn encode_context(encoded: &mut Vec<u8>, lines: Vec<&str>) {
    encoded.extend_from_slice(&(lines.len() as u32).to_le_bytes());
    for line in lines {
        encoded.extend_from_slice(&(line.len() as u32).to_le_bytes());
        encoded.extend_from_slice(line.as_bytes());
    }
}

fn context_before<'a>(
    lines: &[DiffLine],
    start: usize,
    old_lines: &'a SourceLines<'a>,
    new_lines: &'a SourceLines<'a>,
) -> Vec<&'a str> {
    let mut collected = Vec::new();
    for line in lines[..start].iter().rev() {
        if line.style == DiffSpanStyle::Separator {
            continue;
        }
        if line.is_changed() {
            break;
        }
        collected.push(context_text(line, old_lines, new_lines));
        if collected.len() == REVIEW_FINGERPRINT_CONTEXT_LINES {
            break;
        }
    }
    collected.reverse();
    collected
}

fn context_after<'a>(
    lines: &[DiffLine],
    end: usize,
    old_lines: &'a SourceLines<'a>,
    new_lines: &'a SourceLines<'a>,
) -> Vec<&'a str> {
    let mut collected = Vec::new();
    for line in &lines[end + 1..] {
        if line.style == DiffSpanStyle::Separator {
            continue;
        }
        if line.is_changed() {
            break;
        }
        collected.push(context_text(line, old_lines, new_lines));
        if collected.len() == REVIEW_FINGERPRINT_CONTEXT_LINES {
            break;
        }
    }
    collected
}

fn context_text<'a>(
    line: &DiffLine,
    old_lines: &'a SourceLines<'a>,
    new_lines: &'a SourceLines<'a>,
) -> &'a str {
    if line.new_line_no.is_some() {
        new_lines.get(line.new_line_no)
    } else {
        old_lines.get(line.old_line_no)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct ChangedLineKey {
    old: Option<u32>,
    new: Option<u32>,
    added: bool,
}

fn changed_line_keys(lines: &[DiffLine], start: usize, end: usize) -> HashSet<ChangedLineKey> {
    lines[start..=end]
        .iter()
        .filter(|line| line.is_changed())
        .map(|line| ChangedLineKey {
            old: line.old_line_no,
            new: line.new_line_no,
            added: line.style == DiffSpanStyle::Added,
        })
        .collect()
}

struct SourceLines<'a> {
    lines: Vec<&'a str>,
}

impl<'a> SourceLines<'a> {
    fn from_text(text: &'a str) -> Self {
        Self {
            lines: text.lines().collect(),
        }
    }

    fn get(&self, line_no: Option<u32>) -> &str {
        let Some(line_no) = line_no else {
            return "";
        };
        self.lines
            .get(line_no.saturating_sub(1) as usize)
            .copied()
            .unwrap_or("")
    }
}
