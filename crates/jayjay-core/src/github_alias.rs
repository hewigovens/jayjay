//! Well-known commit emails mapped to the GitHub account whose avatar represents them.

const ALIASES: &[(&str, u64, &str)] = &[
    ("noreply@anthropic.com", 81847, "claude"),
    ("codex@openai.com", 267193182, "codex"),
    ("cursoragent@cursor.com", 199161495, "cursoragent"),
    ("droid@factory.ai", 238568704, "factorydroid"),
    ("amp@ampcode.com", 289058786, "ampagent"),
    // Copilot is a bot account without the `[bot]` suffix, so its `u/<id>` avatar is only an identicon.
    (
        "198982749+Copilot@users.noreply.github.com",
        198982749,
        "Copilot[bot]",
    ),
];

/// The GitHub noreply address that stands in for a well-known commit email.
pub fn noreply_email(email: &str) -> Option<String> {
    let email = email.trim();
    ALIASES
        .iter()
        .find(|(alias, _, _)| alias.eq_ignore_ascii_case(email))
        .map(|(_, id, login)| format!("{id}+{login}@users.noreply.github.com"))
}

#[cfg(test)]
mod tests {
    use super::noreply_email;

    #[test]
    fn maps_alias_email_case_insensitively() {
        assert_eq!(
            noreply_email(" NoReply@Anthropic.com ").as_deref(),
            Some("81847+claude@users.noreply.github.com")
        );
        assert_eq!(noreply_email("dev@example.com"), None);
    }
}
