//! Split/join commit messages on the first-line-is-summary convention, so the
//! shells can offer separate Summary + Description fields (GitHub Desktop style)
//! that still map to jj's single change description.

/// First line (summary) of a message, trimmed.
pub fn summary(message: &str) -> String {
    message.lines().next().unwrap_or("").trim().to_owned()
}

/// Everything after the first line (body), with the leading blank line trimmed.
pub fn body(message: &str) -> String {
    let mut lines = message.lines();
    lines.next();
    lines.collect::<Vec<_>>().join("\n").trim().to_owned()
}

/// Combine a summary and optional body into one message: `summary\n\nbody`.
/// An empty body yields just the summary (no trailing blank lines).
pub fn join(summary: &str, body: &str) -> String {
    let summary = summary.trim();
    let body = body.trim();
    if body.is_empty() {
        summary.to_owned()
    } else {
        format!("{summary}\n\n{body}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_summary_and_body() {
        assert_eq!(summary("feat: x\n\ndetails\nmore"), "feat: x");
        assert_eq!(body("feat: x\n\ndetails\nmore"), "details\nmore");
        // Single line: all summary, no body.
        assert_eq!(summary("only summary"), "only summary");
        assert_eq!(body("only summary"), "");
        // No blank line between summary and body still splits.
        assert_eq!(body("summary\nbody"), "body");
    }

    #[test]
    fn joins_with_blank_separator() {
        assert_eq!(join("feat: x", "details\nmore"), "feat: x\n\ndetails\nmore");
        assert_eq!(join("feat: x", ""), "feat: x");
        assert_eq!(join("  feat: x  ", "  body  "), "feat: x\n\nbody");
    }
}
