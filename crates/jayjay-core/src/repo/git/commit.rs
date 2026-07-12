use crate::repo::Repo;
use crate::repo::commit_ai::generate_commit_message_cli;
use crate::types::*;

/// Byte budget for the diff body sent to the AI; keeps prompts within LLM context.
const DIFF_SUMMARY_MAX_BYTES: usize = 4000;

impl Repo {
    /// `jj commit -m <message>` = describe @ + new empty change on top.
    pub fn jj_commit(&self, message: &str) -> CoreResult<()> {
        self.run_jj_reload(&["commit", "-m", message])
    }

    /// Generate a commit message via external AI CLIs (codex, then claude); `None` if all fail.
    pub fn generate_commit_message(&self, diff_summary: &str) -> Option<String> {
        generate_commit_message_cli(diff_summary)
    }

    /// Get a summary of the working copy diff for AI message generation.
    pub fn diff_summary(&self) -> CoreResult<String> {
        let diff_text = self.run_jj(&["diff"])?;
        if diff_text.trim().is_empty() {
            return Ok(String::new());
        }
        let stat_text = self.run_jj(&["diff", "--stat"])?;
        let truncated = truncate_diff_for_ai(&diff_text, DIFF_SUMMARY_MAX_BYTES);

        Ok(format!("{stat_text}\n{truncated}"))
    }
}

/// Truncate `diff` to at most `max_bytes`, cutting on a UTF-8 char boundary so multibyte content never panics the slice.
fn truncate_diff_for_ai(diff: &str, max_bytes: usize) -> String {
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
    use super::truncate_diff_for_ai;

    #[test]
    fn truncate_diff_for_ai_cuts_on_char_boundary() {
        // Short input is returned unchanged.
        assert_eq!(truncate_diff_for_ai("hi", 4000), "hi");

        // A 3-byte char straddling the limit must not panic and must not split it.
        // "界" is 3 bytes; with limit 4 the only valid cut <= 4 is byte 3.
        let s = "abc界def";
        let out = truncate_diff_for_ai(s, 4);
        assert_eq!(out, "abc...\n(truncated)");

        // Limit landing exactly on a boundary keeps the char.
        assert_eq!(truncate_diff_for_ai("abc界def", 6), "abc界...\n(truncated)");

        // Emoji (4 bytes) near the limit: no panic, cut before the emoji.
        let emoji = format!("{}🚀tail", "x".repeat(3998));
        let out = truncate_diff_for_ai(&emoji, 4000);
        assert!(out.starts_with(&"x".repeat(3998)));
        assert!(out.ends_with("...\n(truncated)"));
        assert!(!out.contains('🚀'));
    }
}
