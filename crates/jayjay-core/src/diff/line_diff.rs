/// Line-level diff operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LineOp {
    Equal,
    Remove,
    Add,
}

fn lines_equal(a: &str, b: &str, ignore_whitespace: bool) -> bool {
    if ignore_whitespace {
        let norm = |s: &str| -> String { s.split_whitespace().collect::<Vec<_>>().join(" ") };
        norm(a) == norm(b)
    } else {
        a == b
    }
}

/// Line diff with prefix/suffix trimming for fast common case.
pub(super) fn line_diff(old: &[&str], new: &[&str], ignore_whitespace: bool) -> Vec<LineOp> {
    let n = old.len();
    let m = new.len();

    if n == 0 {
        return vec![LineOp::Add; m];
    }
    if m == 0 {
        return vec![LineOp::Remove; n];
    }

    // Trim common prefix
    let prefix_len = old
        .iter()
        .zip(new.iter())
        .take_while(|(a, b)| lines_equal(a, b, ignore_whitespace))
        .count();

    // Trim common suffix (don't overlap with prefix)
    let max_suffix = n.min(m) - prefix_len;
    let suffix_len = old[n - max_suffix..]
        .iter()
        .rev()
        .zip(new[m - max_suffix..].iter().rev())
        .take_while(|(a, b)| lines_equal(a, b, ignore_whitespace))
        .count();

    let old_mid = &old[prefix_len..n - suffix_len];
    let new_mid = &new[prefix_len..m - suffix_len];

    // If the middle is empty or tiny, no need for expensive LCS
    let mid_ops = if old_mid.is_empty() && new_mid.is_empty() {
        vec![]
    } else if old_mid.is_empty() {
        vec![LineOp::Add; new_mid.len()]
    } else if new_mid.is_empty() {
        vec![LineOp::Remove; old_mid.len()]
    } else {
        lcs_diff(old_mid, new_mid, ignore_whitespace)
    };

    let mut ops = Vec::with_capacity(n + m);
    ops.extend(std::iter::repeat_n(LineOp::Equal, prefix_len));
    ops.extend(mid_ops);
    ops.extend(std::iter::repeat_n(LineOp::Equal, suffix_len));
    ops
}

/// LCS-based diff for the remaining middle section after prefix/suffix trim.
fn lcs_diff(old: &[&str], new: &[&str], ignore_whitespace: bool) -> Vec<LineOp> {
    let n = old.len();
    let m = new.len();

    // Build LCS table
    let mut dp = vec![vec![0u32; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            dp[i][j] = if lines_equal(old[i], new[j], ignore_whitespace) {
                dp[i + 1][j + 1] + 1
            } else {
                dp[i + 1][j].max(dp[i][j + 1])
            };
        }
    }

    // Trace back
    let mut ops = Vec::with_capacity(n + m);
    let mut i = 0;
    let mut j = 0;
    while i < n && j < m {
        if lines_equal(old[i], new[j], ignore_whitespace) {
            ops.push(LineOp::Equal);
            i += 1;
            j += 1;
        } else if dp[i + 1][j] >= dp[i][j + 1] {
            ops.push(LineOp::Remove);
            i += 1;
        } else {
            ops.push(LineOp::Add);
            j += 1;
        }
    }
    while i < n {
        ops.push(LineOp::Remove);
        i += 1;
    }
    while j < m {
        ops.push(LineOp::Add);
        j += 1;
    }
    ops
}
