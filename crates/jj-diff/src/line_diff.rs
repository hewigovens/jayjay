use std::borrow::Cow;

use similar::{DiffTag, capture_diff_slices};

/// Line-level diff operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LineOp {
    Equal,
    Remove,
    Add,
}

/// Line-level diff. Algorithm chosen in [`crate::DIFF_ALGORITHM`].
pub(super) fn line_diff(old: &str, new: &str, ignore_whitespace: bool) -> Vec<LineOp> {
    let old = line_tokens(old, ignore_whitespace);
    let new = line_tokens(new, ignore_whitespace);
    let mut ops = Vec::with_capacity(old.len().max(new.len()));
    for op in capture_diff_slices(crate::DIFF_ALGORITHM, &old, &new) {
        let (tag, old_range, new_range) = op.as_tag_tuple();
        match tag {
            DiffTag::Equal => ops.extend(old_range.map(|_| LineOp::Equal)),
            DiffTag::Delete => ops.extend(old_range.map(|_| LineOp::Remove)),
            DiffTag::Insert => ops.extend(new_range.map(|_| LineOp::Add)),
            DiffTag::Replace => {
                ops.extend(old_range.map(|_| LineOp::Remove));
                ops.extend(new_range.map(|_| LineOp::Add));
            }
        }
    }
    ops
}

/// Each line paired with whether a newline ends it, so only an unterminated last line differs from its terminated twin.
fn line_tokens(text: &str, ignore_whitespace: bool) -> Vec<(Cow<'_, str>, bool)> {
    let count = text.lines().count();
    let terminated = text.ends_with('\n');
    text.lines()
        .enumerate()
        .map(|(ix, line)| {
            let line = if ignore_whitespace {
                Cow::Owned(normalize_ws(line))
            } else {
                Cow::Borrowed(line)
            };
            (line, terminated || ix + 1 < count)
        })
        .collect()
}

fn normalize_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}
