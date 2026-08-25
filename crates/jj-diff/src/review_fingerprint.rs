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
/// Digests hash changed-line payload (side, exact source text, missing-final-newline) plus a bounded amount of surrounding unchanged context, skipping other groups' changed lines. They omit group index, absolute line numbers, highlighting, wrapping, and collapse.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReviewGroupFingerprint {
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
}

/// Exact-whitespace, uncollapsed, unhighlighted snapshot used for persisted review identity.
pub fn canonical_review_snapshot(old: &str, new: &str) -> ReviewFileSnapshot {
    let diff = compute_file_diff_full_plain("", old, new, false);
    fingerprint_snapshot(old, new, &diff.lines)
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

pub(crate) fn map_display_groups_to_canonical(
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
        encode_source_line(
            &mut encoded,
            if added {
                new_lines.get(line.new_line_no)
            } else {
                old_lines.get(line.old_line_no)
            },
        );
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
        digest: hex_sha256(&encoded),
    }
}

fn encode_source_line(encoded: &mut Vec<u8>, line: Option<SourceLine<'_>>) {
    let Some(line) = line else {
        encoded.extend_from_slice(&0u32.to_le_bytes());
        return;
    };
    let extra_cr = matches!(line.ending, LineEnding::CrLf);
    let len = line.text.len() + usize::from(extra_cr);
    encoded.extend_from_slice(&(len as u32).to_le_bytes());
    encoded.extend_from_slice(line.text.as_bytes());
    if extra_cr {
        encoded.push(b'\r');
    }
}

fn encode_context(encoded: &mut Vec<u8>, lines: Vec<Option<SourceLine<'_>>>) {
    encoded.extend_from_slice(&(lines.len() as u32).to_le_bytes());
    for line in lines {
        encode_source_line(encoded, line);
    }
}

fn context_before<'a>(
    lines: &[DiffLine],
    start: usize,
    old_lines: &SourceLines<'a>,
    new_lines: &SourceLines<'a>,
) -> Vec<Option<SourceLine<'a>>> {
    let mut collected = adjacent_context(lines[..start].iter().rev(), old_lines, new_lines);
    collected.reverse();
    collected
}

fn context_after<'a>(
    lines: &[DiffLine],
    end: usize,
    old_lines: &SourceLines<'a>,
    new_lines: &SourceLines<'a>,
) -> Vec<Option<SourceLine<'a>>> {
    adjacent_context(lines[end + 1..].iter(), old_lines, new_lines)
}

fn adjacent_context<'a, 'b, I>(
    lines: I,
    old_lines: &SourceLines<'a>,
    new_lines: &SourceLines<'a>,
) -> Vec<Option<SourceLine<'a>>>
where
    I: Iterator<Item = &'b DiffLine>,
{
    let mut collected = Vec::new();
    // Other groups' changed lines are skipped, not stopped at, so a hunk inserted nearby leaves a reviewed group's digest alone.
    for line in lines {
        if line.style == DiffSpanStyle::Separator || line.is_changed() {
            continue;
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
    old_lines: &SourceLines<'a>,
    new_lines: &SourceLines<'a>,
) -> Option<SourceLine<'a>> {
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

#[derive(Clone, Copy)]
enum LineEnding {
    Lf,
    CrLf,
    None,
}

#[derive(Clone, Copy)]
struct SourceLine<'a> {
    text: &'a str,
    ending: LineEnding,
}

struct SourceLines<'a> {
    lines: Vec<SourceLine<'a>>,
}

impl<'a> SourceLines<'a> {
    fn from_text(text: &'a str) -> Self {
        Self {
            lines: split_source_lines(text),
        }
    }

    fn get(&self, line_no: Option<u32>) -> Option<SourceLine<'a>> {
        let line_no = line_no?;
        self.lines.get(line_no.saturating_sub(1) as usize).copied()
    }
}

/// Same line boundaries as `str::lines()`, but CRLF keeps a trailing `\r` in the hashed payload so an ending-only edit cannot match a reviewed LF group.
fn split_source_lines(text: &str) -> Vec<SourceLine<'_>> {
    let bytes = text.as_bytes();
    let mut lines = Vec::new();
    let mut start = 0;
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'\n' {
            let crlf = index > start && bytes[index - 1] == b'\r';
            lines.push(SourceLine {
                text: &text[start..if crlf { index - 1 } else { index }],
                ending: if crlf {
                    LineEnding::CrLf
                } else {
                    LineEnding::Lf
                },
            });
            index += 1;
            start = index;
        } else {
            index += 1;
        }
    }
    if start < text.len() {
        lines.push(SourceLine {
            text: &text[start..],
            ending: LineEnding::None,
        });
    }
    lines
}
