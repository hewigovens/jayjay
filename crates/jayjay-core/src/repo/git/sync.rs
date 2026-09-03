use std::collections::HashSet;

use crate::repo::{Repo, SyncToken};
use crate::types::*;

const POST_FETCH_SCAN_DEPTH: u32 = 20;

impl Repo {
    /// Returns a message describing what happened (warnings, errors, or success).
    /// Auto-tracks untracked bookmarks before pushing.
    pub fn git_push(&self, bookmark: &str, sync: &SyncToken) -> CoreResult<String> {
        let _enter = sync.enter();
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
        sync.check()?;
        let result = combine_output(&Self::stdout_text(&output), &Self::stderr_text(&output));
        if result.contains("No bookmarks found") || result.contains("Nothing changed") {
            Ok("Nothing to push — create a bookmark first".to_owned())
        } else {
            Ok(result)
        }
    }

    /// Track and push several bookmarks in one `jj git push`, with a single
    /// reload. Used by the stacked-PR submit so each PR head/base exists at once.
    pub(crate) fn git_push_bookmarks(&self, bookmarks: &[&str]) -> CoreResult<String> {
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
    pub fn git_fetch(&self, remote: &str, sync: &SyncToken) -> CoreResult<FetchResult> {
        self.pull(
            sync,
            |repo| repo.git_fetch_raw(remote, ""),
            &["bookmark", "track", "glob:*"],
        )
    }

    /// Fetch a specific bookmark, auto-track it, rebase, and clean up.
    pub fn git_pull_bookmark(&self, bookmark: &str, sync: &SyncToken) -> CoreResult<FetchResult> {
        self.pull(
            sync,
            |repo| repo.git_fetch_raw("", bookmark),
            &["bookmark", "track", "--remote=origin", "--", bookmark],
        )
    }

    fn pull(
        &self,
        sync: &SyncToken,
        fetch: impl FnOnce(&Self) -> CoreResult<String>,
        track_args: &[&str],
    ) -> CoreResult<FetchResult> {
        let _enter = sync.enter();
        let tracking_before = self.tracking_bookmark_names();
        let msg = fetch(self)?;
        let _ = self.run_jj_reload(track_args);
        self.rebase_to_trunk();
        let _ = self.reload();
        sync.check()?;
        self.post_fetch_cleanup(msg, &tracking_before, sync)
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
        sync: &SyncToken,
    ) -> CoreResult<FetchResult> {
        let graph = self
            .log_graph(&format!(
                "present(@) | ancestors(immutable_heads().., {POST_FETCH_SCAN_DEPTH})"
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
                sync.check()?;
                // 100% safe: empty after rebase = content already in parent
                self.run_jj_quiet(&["abandon", &c.change_id.id]);
                abandoned.extend(lost_on_commit);
            } else if c.has_conflict {
                // High confidence but user should confirm
                suggest_abandon.extend(lost_on_commit);
            }
        }

        sync.check()?;
        Ok(FetchResult {
            message,
            abandoned_bookmarks: abandoned,
            suggest_abandon_bookmarks: suggest_abandon,
        })
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
