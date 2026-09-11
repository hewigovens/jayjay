//! Split/join commit messages on the first-line-is-summary convention, so the
//! shells can offer separate Summary + Description fields (GitHub Desktop style)
//! that still map to jj's single change description.

/// First line (summary) of a message, trimmed.
pub fn summary(message: &str) -> String {
    message.lines().next().unwrap_or("").trim().to_owned()
}

/// Body without the optional blank separator and final line ending.
pub fn body(message: &str) -> String {
    let Some((_, body)) = message.split_once('\n') else {
        return String::new();
    };
    let body = body
        .strip_prefix("\r\n")
        .or_else(|| body.strip_prefix('\n'))
        .unwrap_or(body);
    body.strip_suffix("\r\n")
        .or_else(|| body.strip_suffix('\n'))
        .unwrap_or(body)
        .to_owned()
}

/// Combine a summary and optional body into one message: `summary\n\nbody`.
/// An empty body yields just the summary (no trailing blank lines).
pub fn join(summary: &str, body: &str) -> String {
    let summary = summary.trim();
    if body.trim().is_empty() {
        summary.to_owned()
    } else {
        format!("{summary}\n\n{body}")
    }
}

/// Keep the original formatting when the editor fields have not changed.
pub fn update(original: &str, edited_summary: &str, edited_body: &str) -> String {
    if edited_summary == summary(original) && edited_body == body(original) {
        original.to_owned()
    } else {
        join(edited_summary, edited_body)
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
    fn editing_preserves_existing_message_and_body_whitespace() {
        for original in [
            "summary\nbody\n",
            " summary \n\n    code\n    more\n",
            "summary\r\n\r\nbody\r\n",
            "summary\n\n\nbody\n\n",
        ] {
            let title = summary(original);
            let details = body(original);
            assert_eq!(update(original, &title, &details), original);
            assert_eq!(
                update(original, "edited", &details),
                format!("edited\n\n{details}")
            );
        }
        assert_eq!(
            body("summary\n\n    code\n    more\n"),
            "    code\n    more"
        );
        assert_eq!(
            update("summary\nbody\n", "summary", "    edited  "),
            "summary\n\n    edited  "
        );
    }

    #[test]
    fn joins_with_blank_separator() {
        assert_eq!(join("feat: x", "details\nmore"), "feat: x\n\ndetails\nmore");
        assert_eq!(join("feat: x", ""), "feat: x");
        assert_eq!(join("  feat: x  ", "  body  "), "feat: x\n\n  body  ");
    }
}
