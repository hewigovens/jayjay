use std::io::Write;
use std::process::Stdio;
use std::time::Duration;

use super::environment;

pub const COMMIT_MESSAGE_PROMPT: &str = "\
Generate a commit message. Output ONLY the message, nothing else.\n\
Format: one summary line, then blank line, then bullet points.\n\
Summary line: \"Category: what changed\" (under 72 chars).\n\
Valid categories: Add, Update, Fix, Refactor, Remove, Docs, Test, Chore.\n\
Example:\n\
Fix: resolve crash on empty diff view\n\
\n\
- Handle nil layout manager in side-by-side diff\n\
- Add bounds check for lane index in DAG rendering";

pub const BRANCH_NAME_PROMPT: &str = "\
Generate a concise git branch name in kebab-case (lowercase words separated by hyphens, no spaces or punctuation, at most 5 words) that summarizes this change. Output only the branch name.";

const PROVIDERS: [(&str, &str, &[&str]); 2] = [
    (
        "codex",
        "Codex",
        &["exec", "--skip-git-repo-check", "-s", "read-only", "-"],
    ),
    ("claude", "Claude", &["--print"]),
];

pub fn generate_commit_message_cli(diff_summary: &str) -> Option<String> {
    generate_with_cli_chain(diff_summary, COMMIT_MESSAGE_PROMPT)
}

pub fn generate_branch_name_cli(description: &str) -> Option<String> {
    let reply = generate_with_cli_chain(description, BRANCH_NAME_PROMPT)?;
    let slug = super::branch_name_slug(&reply);
    (!slug.is_empty()).then_some(slug)
}

pub fn detect_ai_provider() -> String {
    PROVIDERS
        .iter()
        .find(|(binary, ..)| environment::find_existing_binary(binary).is_some())
        .map(|(_, label, _)| (*label).to_owned())
        .unwrap_or_default()
}

fn generate_with_cli_chain(input: &str, prompt: &str) -> Option<String> {
    let full_input = format!("{prompt}\n\n{input}");
    PROVIDERS.iter().find_map(|(binary, _, args)| {
        let binary = environment::find_existing_binary(binary)?;
        run_ai_cli(&binary, args, &full_input)
    })
}

fn run_ai_cli(binary: &str, args: &[&str], input: &str) -> Option<String> {
    let mut child = environment::command(binary)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(input.as_bytes());
    }
    let reply = environment::wait_for_stdout(child, Duration::from_secs(30))?;
    let reply = reply
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();
    (!reply.is_empty()).then(|| reply.to_owned())
}
