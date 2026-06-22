use std::collections::HashSet;

use crate::repo::{DEFAULT_REVSET_DEPTH, Repo};
use crate::types::*;

impl Repo {
    /// Returns a message describing what happened (warnings, errors, or success).
    /// Auto-tracks untracked bookmarks before pushing.
    pub fn git_push(&self, bookmark: &str) -> CoreResult<String> {
        if bookmark.is_empty() {
            self.run_jj_quiet(&["bookmark", "track", "glob:*"]);
        } else {
            self.run_jj_quiet(&["bookmark", "track", "--remote=origin", "--", bookmark]);
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

    /// Track and push several bookmarks in one `jj git push`, with a single
    /// reload. Used by the stacked-PR submit so each PR head/base exists at once.
    pub fn git_push_bookmarks(&self, bookmarks: &[&str]) -> CoreResult<String> {
        if bookmarks.is_empty() {
            return Ok("Nothing to push.".to_owned());
        }
        for bookmark in bookmarks {
            self.run_jj_quiet(&["bookmark", "track", "--remote=origin", "--", bookmark]);
        }
        // `--bookmark` creates and tracks new remote bookmarks on its own; jj 0.42
        // has no `--allow-new` flag.
        let mut args = vec!["git", "push"];
        for bookmark in bookmarks {
            args.extend(["--bookmark", bookmark]);
        }
        let output = self.run_jj_output(&args)?;
        self.ensure_success(&output, "git push failed")?;
        self.reload()?;
        Ok(combine_output(
            &Self::stdout_text(&output),
            &Self::stderr_text(&output),
        ))
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
        let _ = self.run_jj_reload(&["bookmark", "track", "--remote=origin", "--", bookmark]);
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

    fn tracking_bookmark_names(&self) -> HashSet<String> {
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
        tracking_before: &HashSet<String>,
    ) -> FetchResult {
        let graph = self
            .log_graph(&format!(
                "present(@) | ancestors(immutable_heads().., {DEFAULT_REVSET_DEPTH})"
            ))
            .unwrap_or_default();
        let tracking_after = self.tracking_bookmark_names();
        let lost_tracking: HashSet<&str> = tracking_before
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
                self.run_jj_quiet(&["abandon", &c.change_id.id]);
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
