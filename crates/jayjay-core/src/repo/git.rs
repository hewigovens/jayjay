pub use super::git_ai::{COMMIT_MESSAGE_PROMPT, detect_ai_provider};

use super::Repo;
use super::git_ai::generate_commit_message_cli;
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

        let output = self.command_output(
            "git",
            &[
                "submodule",
                "foreach",
                "--quiet",
                r#"if [ -n "$(git status --porcelain)" ]; then echo "$sm_path"; fi"#,
            ],
            "git submodule foreach",
        )?;

        Ok(Self::stdout_text(&output)
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
            let output = self.command_output_in(
                &abs,
                "git",
                &["add", "."],
                &format!("git add in {sub_path}"),
            )?;
            self.ensure_success(&output, &format!("git add in {sub_path}"))?;

            let output = self.command_output_in(
                &abs,
                "git",
                &["commit", "-m", message],
                &format!("git commit in {sub_path}"),
            )?;
            if !output.status.success() {
                let stderr = Self::stderr_text(&output);
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

        let mut args = vec!["git", "push"];
        if !bookmark.is_empty() {
            args.extend(["--bookmark", bookmark]);
        }
        let output = self.run_jj_output(&args)?;
        self.ensure_success(&output, "git push failed")?;
        self.reload()?;
        let result = combine_output(&Self::stdout_text(&output), &Self::stderr_text(&output));
        if result.contains("No bookmarks found") || result.contains("Nothing changed") {
            Ok("Nothing to push — create a bookmark first".to_owned())
        } else {
            Ok(result)
        }
    }

    /// Get the remote URL for the git repo (origin).
    pub fn git_remote_url(&self) -> CoreResult<String> {
        let output = self.command_output(
            "git",
            &["remote", "get-url", "origin"],
            "git remote get-url",
        )?;
        self.ensure_success(&output, "git remote get-url")?;
        let url = Self::stdout_text(&output);
        if url.is_empty() {
            return Err(CoreError::Internal {
                message: "No remote 'origin' configured".to_owned(),
            });
        }
        Ok(url)
    }

    /// Returns a message describing what happened.
    /// Fetch all remotes and rebase @ onto trunk (git pull --rebase).
    pub fn git_fetch(&self, remote: &str) -> CoreResult<String> {
        let msg = self.git_fetch_raw(remote, "")?;
        self.rebase_to_trunk();
        Ok(msg)
    }

    /// Fetch a specific bookmark and rebase @ onto trunk.
    pub fn git_pull_bookmark(&self, bookmark: &str) -> CoreResult<String> {
        let msg = self.git_fetch_raw("", bookmark)?;
        self.rebase_to_trunk();
        Ok(msg)
    }

    fn git_fetch_raw(&self, remote: &str, bookmark: &str) -> CoreResult<String> {
        let mut args = vec!["git", "fetch"];
        if !remote.is_empty() {
            args.extend(["--remote", remote]);
        }
        if !bookmark.is_empty() {
            args.extend(["-b", bookmark]);
        }
        let output = self.run_jj_output(&args)?;
        self.ensure_success(&output, "git fetch failed")?;

        self.reload()?;
        Ok(combine_output(
            &Self::stdout_text(&output),
            &Self::stderr_text(&output),
        ))
    }

    fn rebase_to_trunk(&self) {
        let _ = self.run_jj(&["rebase", "-d", "trunk()"]);
    }
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
