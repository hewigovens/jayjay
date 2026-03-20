/// Line-level diff operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LineOp {
    Equal,
    Remove,
    Add,
}

/// Myers diff on lines as atomic units. No word-level splitting.
pub(super) fn line_diff(old: &[&str], new: &[&str]) -> Vec<LineOp> {
    let n = old.len();
    let m = new.len();

    if n == 0 {
        return vec![LineOp::Add; m];
    }
    if m == 0 {
        return vec![LineOp::Remove; n];
    }

    // Build LCS table
    let mut dp = vec![vec![0u32; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            dp[i][j] = if old[i] == new[j] {
                dp[i + 1][j + 1] + 1
            } else {
                dp[i + 1][j].max(dp[i][j + 1])
            };
        }
    }

    // Trace back to produce ops
    let mut ops = Vec::with_capacity(n + m);
    let mut i = 0;
    let mut j = 0;
    while i < n && j < m {
        if old[i] == new[j] {
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
