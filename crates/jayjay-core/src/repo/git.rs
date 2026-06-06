pub use super::commit_ai::{COMMIT_MESSAGE_PROMPT, detect_ai_provider};

use std::io::Write;
use std::process::{Command, Stdio};

use gix_url::Scheme;

use super::commit_ai::generate_commit_message_cli;
use super::{JJ_CONFIG_USER_EMAIL, JJ_CONFIG_USER_NAME, Repo};
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

    /// List submodule paths that appear changed in the parent repository status.
    pub fn changed_submodules(&self) -> CoreResult<Vec<String>> {
        Ok(self
            .submodule_statuses()?
            .into_iter()
            .map(|status| status.path)
            .collect())
    }

    pub fn submodule_statuses(&self) -> CoreResult<Vec<GitSubmoduleStatus>> {
        let output = self.command_output(
            "git",
            &[
                "status",
                "--porcelain=v2",
                "--ignore-submodules=none",
                "--untracked-files=no",
            ],
            "git status",
        )?;
        self.ensure_success(&output, "git status")?;

        Ok(parse_git_status_submodule_statuses(&Self::stdout_text(
            &output,
        )))
    }

    pub fn commit_safe_submodule_updates(
        &self,
        message: &str,
        paths: &[String],
    ) -> CoreResult<String> {
        if paths.is_empty() {
            return Ok("No safe submodule updates to commit.".to_owned());
        }

        let mut add_args = vec!["add", "--"];
        add_args.extend(paths.iter().map(String::as_str));
        let output = self.command_output("git", &add_args, "git add submodule updates")?;
        self.ensure_success(&output, "git add submodule updates")?;

        let mut commit_args = vec!["commit", "-m", message, "--only", "--"];
        commit_args.extend(paths.iter().map(String::as_str));
        let output = self.command_output("git", &commit_args, "git commit submodule updates")?;
        self.ensure_success(&output, "git commit submodule updates")?;

        self.run_jj_reload(&["git", "import"])?;

        if self.has_jj_working_copy_changes()? {
            self.jj_commit(message)?;
            Ok("Committed safe submodule updates and working-copy changes.".to_owned())
        } else {
            Ok("Committed safe submodule updates.".to_owned())
        }
    }

    /// List files currently tracked by Git LFS in the checked-out tree.
    pub fn tracked_git_lfs_files(&self) -> CoreResult<Vec<String>> {
        let output = self.command_output(
            "git",
            &["lfs", "ls-files", "--name-only"],
            "git lfs ls-files",
        )?;
        if !output.status.success() {
            return Ok(vec![]);
        }

        Ok(Self::stdout_text(&output)
            .lines()
            .map(|l| l.trim().to_owned())
            .filter(|l| !l.is_empty())
            .collect())
    }

    /// Filter the provided repo-relative paths down to those matched by Git LFS attributes.
    pub fn git_lfs_paths(&self, paths: &[String]) -> CoreResult<Vec<String>> {
        if paths.is_empty() {
            return Ok(vec![]);
        }

        let mut child = Command::new("git")
            .current_dir(&self.path)
            .args(["check-attr", "--stdin", "filter"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| CoreError::Internal {
                message: format!("git check-attr: {e}"),
            })?;

        // Drain stdin on a side thread so wait_with_output can read stdout concurrently — otherwise git blocks writing stdout while we block writing stdin.
        let stdin = child.stdin.take().ok_or_else(|| CoreError::Internal {
            message: "git check-attr: failed to open stdin".to_owned(),
        })?;
        let paths_to_send: Vec<String> = paths.to_vec();
        let writer = std::thread::spawn(move || -> std::io::Result<()> {
            let mut stdin = stdin;
            for path in &paths_to_send {
                writeln!(stdin, "{path}")?;
            }
            Ok(())
        });

        let output = child.wait_with_output().map_err(|e| CoreError::Internal {
            message: format!("git check-attr: {e}"),
        })?;
        writer
            .join()
            .map_err(|_| CoreError::Internal {
                message: "git check-attr: stdin writer thread panicked".to_owned(),
            })?
            .map_err(|e| CoreError::Internal {
                message: format!("git check-attr stdin: {e}"),
            })?;
        self.ensure_success(&output, "git check-attr")?;

        Ok(parse_git_check_attr_lfs_paths(&Self::stdout_text(&output)))
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

    /// Check whether `user.name` and `user.email` are set in jj config.
    /// Returns a warning message when either is missing, or `None` if both are set.
    pub fn check_user_config(&self) -> Option<String> {
        let has_name = self.run_jj(&["config", "get", JJ_CONFIG_USER_NAME]).is_ok();
        let has_email = self
            .run_jj(&["config", "get", JJ_CONFIG_USER_EMAIL])
            .is_ok();
        if has_name && has_email {
            return None;
        }
        let mut missing = Vec::new();
        if !has_name {
            missing.push(JJ_CONFIG_USER_NAME);
        }
        if !has_email {
            missing.push(JJ_CONFIG_USER_EMAIL);
        }
        Some(format!(
            "jj is not fully configured — {} not set. \
             Run `jj config set --user {} <value>` or edit your config file.",
            missing.join(" and "),
            missing.join("` and `jj config set --user "),
        ))
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
            self.run_jj_quiet(&["bookmark", "track", "glob:*"]);
        } else {
            self.run_jj_quiet(&["bookmark", "track", bookmark, "--remote=origin"]);
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

    /// The origin remote as an https web URL, for "open in browser". None if there's
    /// no origin or it can't be parsed.
    pub fn remote_web_url(&self) -> Option<String> {
        git_remote_to_web_url(&self.git_remote_url().ok()?)
    }

    /// Fetch all remotes, auto-track, rebase, and clean up merged bookmarks.
    pub fn git_fetch(&self, remote: &str) -> CoreResult<FetchResult> {
        let tracking_before = self.tracking_bookmark_names();
        let msg = self.git_fetch_raw(remote, "")?;
        let _ = self.run_jj_reload(&["bookmark", "track", "glob:*"]);
        self.rebase_to_trunk();
        let _ = self.reload();
        Ok(self.post_fetch_cleanup(msg, &tracking_before))
    }

    /// Fetch a specific bookmark, auto-track it, rebase, and clean up.
    pub fn git_pull_bookmark(&self, bookmark: &str) -> CoreResult<FetchResult> {
        let tracking_before = self.tracking_bookmark_names();
        let msg = self.git_fetch_raw("", bookmark)?;
        let _ = self.run_jj_reload(&["bookmark", "track", bookmark, "--remote=origin"]);
        self.rebase_to_trunk();
        let _ = self.reload();
        Ok(self.post_fetch_cleanup(msg, &tracking_before))
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

    fn tracking_bookmark_names(&self) -> std::collections::HashSet<String> {
        self.list_bookmarks()
            .unwrap_or_default()
            .into_iter()
            .filter(|b| b.is_tracking_remote)
            .map(|b| b.name)
            .collect()
    }

    fn post_fetch_cleanup(
        &self,
        message: String,
        tracking_before: &std::collections::HashSet<String>,
    ) -> FetchResult {
        let graph = self
            .log_graph(&format!(
                "present(@) | ancestors(immutable_heads().., {})",
                super::DEFAULT_REVSET_DEPTH
            ))
            .unwrap_or_default();
        let tracking_after = self.tracking_bookmark_names();
        let lost_tracking: std::collections::HashSet<&str> = tracking_before
            .iter()
            .filter(|name| !tracking_after.contains(name.as_str()))
            .map(|s| s.as_str())
            .collect();

        let mut abandoned = Vec::new();
        let mut suggest_abandon = Vec::new();

        for entry in &graph {
            let c = &entry.change;
            if c.is_immutable || c.is_working_copy || c.bookmarks.is_empty() {
                continue;
            }
            let lost_on_commit: Vec<_> = c
                .bookmarks
                .iter()
                .filter(|b| lost_tracking.contains(b.as_str()))
                .cloned()
                .collect();
            if lost_on_commit.is_empty() {
                continue;
            }
            if c.is_empty {
                // 100% safe: empty after rebase = content already in parent
                self.run_jj_quiet(&["abandon", &c.change_id]);
                abandoned.extend(lost_on_commit);
            } else if c.has_conflict {
                // High confidence but user should confirm
                suggest_abandon.extend(lost_on_commit);
            }
        }

        FetchResult {
            message,
            abandoned_bookmarks: abandoned,
            suggest_abandon_bookmarks: suggest_abandon,
        }
    }

    fn has_jj_working_copy_changes(&self) -> CoreResult<bool> {
        self.refresh_working_copy()?;
        let detail = self.show_summary("@")?;
        if !detail.diff.is_empty() {
            return Ok(true);
        }
        let conflicts = self.resolve_list("@").unwrap_or_default();
        Ok(!conflicts.is_empty())
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

fn parse_git_check_attr_lfs_paths(stdout: &str) -> Vec<String> {
    stdout
        .lines()
        .filter_map(|line| line.strip_suffix(": filter: lfs"))
        .map(str::to_owned)
        .collect()
}

fn parse_git_status_submodule_statuses(stdout: &str) -> Vec<GitSubmoduleStatus> {
    stdout
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            if parts.next()? != "1" {
                return None;
            }

            let _xy = parts.next()?;
            let submodule_state = parts.next()?;
            if !submodule_state.starts_with('S') {
                return None;
            }

            let path = parts.last()?.to_owned();
            let state: Vec<char> = submodule_state.chars().collect();
            Some(GitSubmoduleStatus {
                path,
                has_new_commits: state.get(1) == Some(&'C'),
                has_modified_content: state.get(2) == Some(&'M'),
                has_untracked_content: state.get(3) == Some(&'U'),
            })
        })
        .collect()
}

/// Normalize a git remote — scp (`git@host:owner/repo`), `ssh://`, `git://`, or
/// `http(s)://` — to its https web URL. None if it can't be parsed.
fn git_remote_to_web_url(raw: &str) -> Option<String> {
    let url = gix_url::parse(raw.trim().as_bytes().into()).ok()?;
    if !matches!(
        &url.scheme,
        Scheme::Http | Scheme::Https | Scheme::Ssh | Scheme::Git
    ) {
        return None;
    }
    let host = url.host()?;
    let path = std::str::from_utf8(url.path.as_ref())
        .ok()?
        .trim_matches('/');
    let path = path.strip_suffix(".git").unwrap_or(path);
    if path.is_empty() {
        return None;
    }
    Some(format!("https://{host}/{path}"))
}

#[cfg(test)]
mod tests {
    use super::{
        git_remote_to_web_url, parse_git_check_attr_lfs_paths, parse_git_status_submodule_statuses,
    };

    #[test]
    fn git_remote_to_web_url_normalizes_every_form() {
        let cases = [
            (
                "ssh://git@codeberg.org/hewig/jj-test.git",
                "https://codeberg.org/hewig/jj-test",
            ),
            (
                "git@github.com:owner/repo.git",
                "https://github.com/owner/repo",
            ),
            (
                "git://example.com/owner/repo",
                "https://example.com/owner/repo",
            ),
            ("https://codeberg.org/o/r.git", "https://codeberg.org/o/r"),
            ("ssh://git@host:2222/o/r.git", "https://host/o/r"),
        ];
        for (raw, expected) in cases {
            assert_eq!(
                git_remote_to_web_url(raw).as_deref(),
                Some(expected),
                "{raw}"
            );
        }
        assert_eq!(git_remote_to_web_url("not-a-remote"), None);
        assert_eq!(git_remote_to_web_url(""), None);
    }

    #[test]
    fn parses_git_check_attr_lfs_paths() {
        let paths = parse_git_check_attr_lfs_paths(
            "packages/foo.bin: filter: lfs\nREADME.md: filter: unspecified\npackages/bar.bin: filter: lfs\n",
        );

        assert_eq!(
            paths,
            vec!["packages/foo.bin".to_owned(), "packages/bar.bin".to_owned()]
        );
    }

    #[test]
    fn parses_git_status_submodule_statuses() {
        let statuses = parse_git_status_submodule_statuses(
            "1 .M SC.. 160000 160000 160000 2196000605e45d91097147c9c71f26b72af58003 2196000605e45d91097147c9c71f26b72af58003 crypto/tests/wycheproof\n\
             1 .M S..U 160000 160000 160000 a660a4976efe880bae7982ee410b9e0dc59ac983 a660a4976efe880bae7982ee410b9e0dc59ac983 vendor/secp256k1-zkp\n",
        );

        assert_eq!(statuses.len(), 2);
        assert_eq!(statuses[0].path, "crypto/tests/wycheproof");
        assert!(statuses[0].has_new_commits);
        assert!(!statuses[0].has_modified_content);
        assert!(!statuses[0].has_untracked_content);
        assert_eq!(statuses[1].path, "vendor/secp256k1-zkp");
        assert!(!statuses[1].has_new_commits);
        assert!(!statuses[1].has_modified_content);
        assert!(statuses[1].has_untracked_content);
    }
}
