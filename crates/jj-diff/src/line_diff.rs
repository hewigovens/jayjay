/// Line-level diff operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LineOp {
    Equal,
    Remove,
    Add,
}

/// Line-level diff. Algorithm chosen in [`crate::DIFF_ALGORITHM`].
pub(super) fn line_diff(old: &[&str], new: &[&str], ignore_whitespace: bool) -> Vec<LineOp> {
    use crate::text_diff_config;
    use similar::ChangeTag;

    if ignore_whitespace {
        // Normalize whitespace for comparison
        let old_normalized: Vec<String> = old.iter().map(|l| normalize_ws(l)).collect();
        let new_normalized: Vec<String> = new.iter().map(|l| normalize_ws(l)).collect();
        let old_joined = old_normalized.join("\n");
        let new_joined = new_normalized.join("\n");

        let diff = text_diff_config().diff_lines(&old_joined, &new_joined);

        diff.ops()
            .iter()
            .flat_map(|op| diff.iter_changes(op))
            .map(|change| match change.tag() {
                ChangeTag::Equal => LineOp::Equal,
                ChangeTag::Delete => LineOp::Remove,
                ChangeTag::Insert => LineOp::Add,
            })
            .collect()
    } else {
        let old_joined = old.join("\n");
        let new_joined = new.join("\n");

        let diff = text_diff_config().diff_lines(&old_joined, &new_joined);

        diff.ops()
            .iter()
            .flat_map(|op| diff.iter_changes(op))
            .map(|change| match change.tag() {
                ChangeTag::Equal => LineOp::Equal,
                ChangeTag::Delete => LineOp::Remove,
                ChangeTag::Insert => LineOp::Add,
            })
            .collect()
    }
}

fn normalize_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}
