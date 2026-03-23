use super::Repo;
use crate::types::*;

impl Repo {
    /// `jj commit -m <message>` = describe @ + new empty change on top.
    pub fn jj_commit(&self, message: &str) -> CoreResult<()> {
        self.run_jj_reload(&["commit", "-m", message])
    }

    /// List submodule paths that have uncommitted changes.
    pub fn dirty_submodules(&self) -> CoreResult<Vec<String>> {
        if !self.path.join(".gitmodules").exists() {
            return Ok(vec![]);
        }

        // Use `git submodule foreach` to reliably detect dirty working trees.
        // `git submodule status` only shows '+' when HEAD changed, not for uncommitted edits.
        let output = std::process::Command::new("git")
            .current_dir(&self.path)
            .args([
                "submodule",
                "foreach",
                "--quiet",
                r#"if [ -n "$(git status --porcelain)" ]; then echo "$sm_path"; fi"#,
            ])
            .output()
            .map_err(|e| CoreError::Internal {
                message: format!("git submodule foreach: {e}"),
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout
            .lines()
            .map(|l| l.trim().to_owned())
            .filter(|l| !l.is_empty())
            .collect())
    }

    /// Commit changes in dirty submodules, then do `jj commit`.
    pub fn commit_with_submodules(&self, message: &str) -> CoreResult<()> {
        // First, commit any dirty submodules
        let dirty = self.dirty_submodules()?;
        for sub_path in &dirty {
            let abs = self.path.join(sub_path);
            let output = std::process::Command::new("git")
                .current_dir(&abs)
                .args(["add", "."])
                .output()
                .map_err(|e| CoreError::Internal {
                    message: format!("git add in {sub_path}: {e}"),
                })?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(CoreError::Internal {
                    message: format!("git add in {sub_path}: {stderr}"),
                });
            }
            let output = std::process::Command::new("git")
                .current_dir(&abs)
                .args(["commit", "-m", message])
                .output()
                .map_err(|e| CoreError::Internal {
                    message: format!("git commit in {sub_path}: {e}"),
                })?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                // "nothing to commit" is ok
                if !stderr.contains("nothing to commit") {
                    return Err(CoreError::Internal {
                        message: format!("git commit in {sub_path}: {stderr}"),
                    });
                }
            }
        }

        // Now jj commit (snapshot will pick up updated submodule pointers)
        self.jj_commit(message)
    }

    /// Get jj configuration as a list of key=value pairs.
    pub fn jj_config(&self) -> CoreResult<String> {
        self.run_jj(&["config", "list"])
    }

    /// Get jj config file path.
    pub fn jj_config_path(&self) -> CoreResult<String> {
        self.run_jj(&["config", "path", "--user"])
    }

    /// Try to generate a commit message using external AI CLIs (codex, then claude).
    /// Returns `None` if no CLI is available or all fail.
    pub fn generate_commit_message(&self, diff_summary: &str) -> Option<String> {
        generate_commit_message_cli(diff_summary)
    }

    /// Get a summary of the working copy diff for AI message generation.
    pub fn diff_summary(&self) -> CoreResult<String> {
        // Stats overview
        let stat_text = self.run_jj(&["diff", "--stat"])?;

        // Actual diff content (truncated to ~4000 chars to stay within LLM context)
        let diff_text = self.run_jj(&["diff"])?;
        let truncated: String = if diff_text.len() > 4000 {
            format!("{}...\n(truncated)", &diff_text[..4000])
        } else {
            diff_text
        };

        Ok(format!("{stat_text}\n{truncated}"))
    }
    /// Returns a message describing what happened (warnings, errors, or success).
    /// Auto-tracks untracked bookmarks before pushing.
    pub fn git_push(&self, bookmark: &str) -> CoreResult<String> {
        // Auto-track untracked remote bookmarks before push
        if bookmark.is_empty() {
            self.run_jj_quiet(&["bookmark", "track", "--all-remotes"]);
        } else {
            self.run_jj_quiet(&["bookmark", "track", &format!("{bookmark}@origin")]);
        }

        let mut cmd = std::process::Command::new(super::environment::jj_binary());
        cmd.current_dir(&self.path);
        cmd.args(["git", "push"]);
        if !bookmark.is_empty() {
            cmd.args(["--bookmark", bookmark]);
        }
        let output = cmd.output().map_err(|e| CoreError::Internal {
            message: format!("run jj git push: {e}"),
        })?;
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        if !output.status.success() {
            return Err(CoreError::Internal {
                message: format!("git push failed: {stderr}"),
            });
        }
        self.reload()?;
        let result = combine_output(&stdout, &stderr);
        if result.contains("No bookmarks found") || result.contains("Nothing changed") {
            Ok("Nothing to push — create a bookmark first".to_owned())
        } else {
            Ok(result)
        }
    }

    /// Get the remote URL for the git repo (origin).
    pub fn git_remote_url(&self) -> CoreResult<String> {
        let output = std::process::Command::new("git")
            .current_dir(&self.path)
            .args(["remote", "get-url", "origin"])
            .output()
            .map_err(|e| CoreError::Internal {
                message: format!("git remote get-url: {e}"),
            })?;
        let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if url.is_empty() {
            return Err(CoreError::Internal {
                message: "No remote 'origin' configured".to_owned(),
            });
        }
        Ok(url)
    }

    /// Returns a message describing what happened.
    pub fn git_fetch(&self, remote: &str) -> CoreResult<String> {
        let mut cmd = std::process::Command::new(super::environment::jj_binary());
        cmd.current_dir(&self.path);
        cmd.args(["git", "fetch"]);
        if !remote.is_empty() {
            cmd.args(["--remote", remote]);
        }
        let output = cmd.output().map_err(|e| CoreError::Internal {
            message: format!("run jj git fetch: {e}"),
        })?;
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        if !output.status.success() {
            return Err(CoreError::Internal {
                message: format!("git fetch failed: {stderr}"),
            });
        }
        self.reload()?;
        Ok(combine_output(&stdout, &stderr))
    }
}

/// Find a CLI binary by name, checking common macOS paths.
/// Same pattern as `jj_binary()` — macOS app bundles don't inherit shell PATH.
fn find_binary(name: &str) -> Option<String> {
    // Check home-local bins first
    if let Ok(home) = std::env::var("HOME") {
        let local_bin = format!("{home}/.local/bin/{name}");
        if std::path::Path::new(&local_bin).exists() {
            return Some(local_bin);
        }
    }
    let candidates = [
        format!("/opt/homebrew/bin/{name}"),
        format!("/usr/local/bin/{name}"),
        format!("/usr/bin/{name}"),
    ];
    candidates
        .into_iter()
        .find(|path| std::path::Path::new(&path).exists())
}

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

/// Try to generate a commit message using an external AI CLI (codex, then claude).
/// Returns `None` if no CLI is available or all fail.
pub fn generate_commit_message_cli(diff_summary: &str) -> Option<String> {
    let prompt = COMMIT_MESSAGE_PROMPT;

    // 1. Try codex
    if let Some(codex) = find_binary("codex") {
        if let Some(msg) = run_ai_cli(&codex, diff_summary, prompt, AiCliMode::Codex) {
            return Some(msg);
        }
    }

    // 2. Try claude
    if let Some(claude) = find_binary("claude") {
        if let Some(msg) = run_ai_cli(&claude, diff_summary, prompt, AiCliMode::Claude) {
            return Some(msg);
        }
    }

    None
}

enum AiCliMode {
    Codex,
    Claude,
}

fn run_ai_cli(binary: &str, diff_summary: &str, prompt: &str, mode: AiCliMode) -> Option<String> {
    use std::io::Write;
    use std::time::Duration;

    // Combine prompt + diff into a single input
    let full_input = format!("{prompt}\n\nChanged files:\n\n{diff_summary}");

    let mut cmd = std::process::Command::new(binary);
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
    let raw = String::from_utf8_lossy(&output.stdout);
    // Strip markdown fences and prompt echo that models sometimes add
    let text = raw
        .trim()
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim()
        .to_string();
    if text.is_empty() { None } else { Some(text) }
}

fn combine_output(stdout: &str, stderr: &str) -> String {
    let mut parts = Vec::new();
    let s = stdout.trim();
    let e = stderr.trim();
    if !s.is_empty() {
        parts.push(s);
    }
    if !e.is_empty() {
        parts.push(e);
    }
    if parts.is_empty() {
        "Done.".to_owned()
    } else {
        parts.join("\n")
    }
}

/// Returns the name of the first available AI CLI provider ("Codex" or "Claude"), or empty string.
pub fn detect_ai_provider() -> String {
    if find_binary("codex").is_some() {
        "Codex".to_owned()
    } else if find_binary("claude").is_some() {
        "Claude".to_owned()
    } else {
        String::new()
    }
}
