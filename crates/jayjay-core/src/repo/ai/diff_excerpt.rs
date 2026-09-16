use crate::repo::Repo;
use crate::types::*;

/// Lock files drown out the real change; they stay in the stat unless nothing else changed.
const WITHOUT_LOCK_FILES: &str = r#"~(root-glob:"**/*.lock" | root-glob:"**/package-lock.json" | root-glob:"**/pnpm-lock.yaml" | root-glob:"**/go.sum" | root-glob:"**/Package.resolved")"#;

#[derive(Debug)]
pub struct DiffExcerpt {
    pub stat: String,
    pub diff: String,
}

impl DiffExcerpt {
    /// Fits the 4K on-device window before macOS 26.4; the agent CLIs get the same.
    pub const DEFAULT_MAX_BYTES: usize = 4000;
    /// Keeps what crosses FFI bounded.
    const MAX_DIFF_BYTES: usize = 64 * 1024;

    fn new(stat: String, diff: &str) -> Self {
        Self {
            stat,
            diff: truncate(diff, Self::MAX_DIFF_BYTES),
        }
    }

    /// The stat keeps at most half of `max_bytes`; the diff gets whatever the stat left.
    pub fn text(&self, max_bytes: usize) -> String {
        let stat = truncate(&self.stat, max_bytes / 2);
        let diff = truncate(&self.diff, max_bytes.saturating_sub(stat.len()));
        format!("{stat}\n{diff}")
    }
}

impl Repo {
    pub fn diff_excerpt(&self) -> CoreResult<Option<DiffExcerpt>> {
        let mut diff = self.run_jj(&["diff", "--", WITHOUT_LOCK_FILES])?;
        if diff.trim().is_empty() {
            diff = self.run_jj(&["diff"])?;
        }
        if diff.trim().is_empty() {
            return Ok(None);
        }
        let stat = self.run_jj(&["diff", "--stat"])?;
        Ok(Some(DiffExcerpt::new(stat, &diff)))
    }
}

/// Cuts on a UTF-8 char boundary so multibyte content never panics the slice.
fn truncate(diff: &str, max_bytes: usize) -> String {
    if diff.len() <= max_bytes {
        return diff.to_owned();
    }
    let cut = (0..=max_bytes)
        .rev()
        .find(|i| diff.is_char_boundary(*i))
        .unwrap_or(0);
    format!("{}...\n(truncated)", &diff[..cut])
}

#[cfg(test)]
mod tests {
    use super::{DiffExcerpt, truncate};

    #[test]
    fn truncate_cuts_on_char_boundary() {
        assert_eq!(truncate("hi", 4000), "hi");
        assert_eq!(truncate("abc界def", 4), "abc...\n(truncated)");
        assert_eq!(truncate("abc界def", 6), "abc界...\n(truncated)");

        let emoji = format!("{}🚀tail", "x".repeat(3998));
        let out = truncate(&emoji, 4000);
        assert!(out.starts_with(&"x".repeat(3998)));
        assert!(out.ends_with("...\n(truncated)"));
        assert!(!out.contains('🚀'));
    }

    #[test]
    fn text_shares_the_budget_between_stat_and_diff() {
        let excerpt = DiffExcerpt::new("1 file changed".to_owned(), "abcdefghijklmnopqrst");
        assert_eq!(excerpt.text(100), "1 file changed\nabcdefghijklmnopqrst");
        assert_eq!(
            excerpt.text(30),
            "1 file changed\nabcdefghijklmnop...\n(truncated)"
        );

        let wide = DiffExcerpt::new("x".repeat(40), "abcdef");
        let text = wide.text(20);
        assert!(text.starts_with(&format!("{}...\n(truncated)\n", "x".repeat(10))));
        assert!(text.ends_with("(truncated)"));
        assert!(!text.contains("abcdef"));
    }
}
