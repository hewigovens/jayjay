use std::io::Write;
use std::process::Stdio;
use std::time::Duration;

use crate::repo::branch_name_slug;
use crate::repo::environment::{self, find_existing_binary};

const COMMIT_MESSAGE_PROMPT: &str = "\
Generate a commit message. Output ONLY the message, nothing else.\n\
Format: one summary line, then blank line, then bullet points.\n\
Summary line: \"Category: what changed\" (under 72 chars).\n\
Valid categories: Add, Update, Fix, Refactor, Remove, Docs, Test, Chore.\n\
Example:\n\
Fix: resolve crash on empty diff view\n\
\n\
- Handle nil layout manager in side-by-side diff\n\
- Add bounds check for lane index in DAG rendering";

const BRANCH_NAME_PROMPT: &str = "\
Generate a concise git branch name in kebab-case (lowercase words separated by hyphens, no spaces or punctuation, at most 5 words) that summarizes this change. Output only the branch name.";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AiProvider {
    Codex,
    Claude,
    /// Runs in the SwiftUI shell through Foundation Models; core never runs it.
    AppleIntelligence,
}

impl AiProvider {
    pub const ALL: [Self; 3] = [Self::Codex, Self::Claude, Self::AppleIntelligence];

    pub fn id(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
            Self::AppleIntelligence => "apple-intelligence",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Codex => "Codex",
            Self::Claude => "Claude",
            Self::AppleIntelligence => "Apple Intelligence",
        }
    }

    /// Saved ids first, then the rest; unknown and repeated ids are dropped.
    pub fn ordered(ids: &[String]) -> Vec<Self> {
        let named = ids
            .iter()
            .filter_map(|id| Self::ALL.into_iter().find(|provider| provider.id() == id));
        let mut order = Vec::with_capacity(Self::ALL.len());
        for provider in named.chain(Self::ALL) {
            if !order.contains(&provider) {
                order.push(provider);
            }
        }
        order
    }

    /// Probes the filesystem, so keep it off the UI thread.
    pub fn is_installed(self) -> bool {
        self.cli()
            .is_some_and(|(name, _)| find_existing_binary(name).is_some())
    }

    pub fn generate_commit_message(self, diff_summary: &str) -> Option<String> {
        self.respond(COMMIT_MESSAGE_PROMPT, diff_summary)
    }

    pub fn generate_branch_name(self, description: &str) -> Option<String> {
        let slug = branch_name_slug(&self.respond(BRANCH_NAME_PROMPT, description)?);
        (!slug.is_empty()).then_some(slug)
    }

    fn cli(self) -> Option<(&'static str, &'static [&'static str])> {
        match self {
            Self::Codex => Some((
                "codex",
                &["exec", "--skip-git-repo-check", "-s", "read-only", "-"],
            )),
            Self::Claude => Some(("claude", &["--print"])),
            Self::AppleIntelligence => None,
        }
    }

    fn respond(self, prompt: &str, input: &str) -> Option<String> {
        let (name, args) = self.cli()?;
        let mut child = environment::command(&find_existing_binary(name)?)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .ok()?;
        if let Some(mut stdin) = child.stdin.take() {
            let _ = write!(stdin, "{prompt}\n\n{input}");
        }
        let reply = environment::wait_for_stdout(child, Duration::from_secs(30))?;
        let reply = reply
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();
        (!reply.is_empty()).then(|| reply.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::AiProvider::{self, AppleIntelligence, Claude, Codex};

    fn ordered(ids: &[&str]) -> Vec<AiProvider> {
        let ids: Vec<String> = ids.iter().map(|id| (*id).to_owned()).collect();
        AiProvider::ordered(&ids)
    }

    #[test]
    fn ordered_follows_saved_ids_then_the_default_order() {
        assert_eq!(ordered(&[]), [Codex, Claude, AppleIntelligence]);
        assert_eq!(
            ordered(&["apple-intelligence", "claude"]),
            [AppleIntelligence, Claude, Codex]
        );
        assert_eq!(
            ordered(&["gemini", "claude", "claude"]),
            [Claude, Codex, AppleIntelligence]
        );
    }
}
