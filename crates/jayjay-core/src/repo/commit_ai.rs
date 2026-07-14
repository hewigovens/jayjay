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

/// Try to generate a commit message using an external AI CLI (codex, then claude).
/// Returns `None` if no CLI is available or all fail.
pub fn generate_commit_message_cli(diff_summary: &str) -> Option<String> {
    let prompt = COMMIT_MESSAGE_PROMPT;

    if let Some(codex) = environment::find_existing_binary("codex")
        && let Some(message) = run_ai_cli(&codex, diff_summary, prompt, AiCliMode::Codex)
    {
        return Some(message);
    }

    if let Some(claude) = environment::find_existing_binary("claude")
        && let Some(message) = run_ai_cli(&claude, diff_summary, prompt, AiCliMode::Claude)
    {
        return Some(message);
    }

    None
}

/// Generate and sanitize a short branch-name slug using the configured AI CLI chain.
pub fn generate_branch_name_cli(description: &str) -> Option<String> {
    let reply = generate_with_cli_chain(description, BRANCH_NAME_PROMPT)?;
    branch_name_slug(&reply)
}

fn generate_with_cli_chain(input: &str, prompt: &str) -> Option<String> {
    if let Some(codex) = environment::find_existing_binary("codex")
        && let Some(reply) = run_ai_cli(&codex, input, prompt, AiCliMode::Codex)
    {
        return Some(reply);
    }
    if let Some(claude) = environment::find_existing_binary("claude")
        && let Some(reply) = run_ai_cli(&claude, input, prompt, AiCliMode::Claude)
    {
        return Some(reply);
    }
    None
}

fn branch_name_slug(raw: &str) -> Option<String> {
    let mut words = Vec::new();
    for word in raw.split(|ch: char| !ch.is_ascii_alphanumeric()) {
        if !word.is_empty() {
            words.push(word.to_ascii_lowercase());
            if words.len() == 5 {
                break;
            }
        }
    }
    (!words.is_empty()).then(|| words.join("-"))
}

/// Returns the name of the first available AI CLI provider ("Codex" or "Claude"), or empty string.
pub fn detect_ai_provider() -> String {
    if environment::find_existing_binary("codex").is_some() {
        "Codex".to_owned()
    } else if environment::find_existing_binary("claude").is_some() {
        "Claude".to_owned()
    } else {
        String::new()
    }
}

enum AiCliMode {
    Codex,
    Claude,
}

fn run_ai_cli(binary: &str, diff_summary: &str, prompt: &str, mode: AiCliMode) -> Option<String> {
    use std::io::Write;
    use std::time::Duration;

    let full_input = format!("{prompt}\n\n{diff_summary}");

    let mut cmd = environment::command(binary);
    cmd.stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null());

    match mode {
        AiCliMode::Codex => {
            cmd.args(["--quiet", "-"]);
        }
        AiCliMode::Claude => {
            cmd.arg("--print");
        }
    }

    let mut child = cmd.spawn().ok()?;

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(full_input.as_bytes());
    }

    let timeout = Duration::from_secs(30);
    let start = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    return None;
                }
                break;
            }
            Ok(None) => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return None;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(_) => return None,
        }
    }

    let output = child.wait_with_output().ok()?;
    let text = String::from_utf8_lossy(&output.stdout)
        .trim()
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim()
        .to_string();
    if text.is_empty() { None } else { Some(text) }
}

#[cfg(test)]
mod tests {
    use super::branch_name_slug;

    #[test]
    fn branch_name_reply_is_sanitized_and_capped() {
        assert_eq!(
            branch_name_slug("**Add stacked PR names, safely now**").as_deref(),
            Some("add-stacked-pr-names-safely")
        );
        assert_eq!(branch_name_slug(" -- "), None);
    }
}
