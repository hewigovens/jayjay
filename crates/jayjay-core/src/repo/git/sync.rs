use std::collections::HashSet;

use crate::repo::{DEFAULT_REVSET_DEPTH, Repo, SyncToken};
use crate::types::*;

impl Repo {
    /// Returns a message describing what happened (warnings, errors, or success).
    /// Tracks only an explicitly requested bookmark before pushing.
    pub fn git_push(&self, bookmark: &str, sync: &SyncToken) -> CoreResult<String> {
        let _enter = sync.enter();
        if !bookmark.is_empty() {
            let _ = self.track_bookmark(bookmark, "origin");
        }

        let mut args = vec!["git", "push"];
        if !bookmark.is_empty() {
            args.extend(["--bookmark", bookmark]);
        }
        let output = self.run_jj_output(&args)?;
        self.ensure_success(&output, "git push failed")?;
        self.reload()?;
        sync.check()?;
        Ok(combine_output(
            &Self::stdout_text(&output),
            &Self::stderr_text(&output),
        ))
    }

    /// Track and push several bookmarks in one `jj git push`, with a single
    /// reload. Used by the stacked-PR submit so each PR head/base exists at once.
    pub(crate) fn git_push_bookmarks(&self, bookmarks: &[&str]) -> CoreResult<String> {
        if bookmarks.is_empty() {
            return Ok("Nothing to push.".to_owned());
        }
        for bookmark in bookmarks {
            let _ = self.track_bookmark(bookmark, "origin");
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

    /// Fetch without changing bookmark tracking, rebase, and clean up merged bookmarks.
    pub fn git_fetch(&self, remote: &str, sync: &SyncToken) -> CoreResult<FetchResult> {
        self.pull(sync, |repo| repo.git_fetch_raw(remote, ""), None)
    }

    /// Fetch a specific bookmark, auto-track it, rebase, and clean up.
    pub fn git_pull_bookmark(&self, bookmark: &str, sync: &SyncToken) -> CoreResult<FetchResult> {
        self.pull(
            sync,
            |repo| repo.git_fetch_raw("", bookmark),
            Some(bookmark),
        )
    }

    fn pull(
        &self,
        sync: &SyncToken,
        fetch: impl FnOnce(&Self) -> CoreResult<String>,
        track: Option<&str>,
    ) -> CoreResult<FetchResult> {
        let _enter = sync.enter();
        let tracking_before = self.tracking_bookmark_names();
        let msg = fetch(self)?;
        let _write = self.write_guard()?;
        if let Some(bookmark) = track {
            let _ = self.track_bookmark(bookmark, "origin");
        }
        // The in-process rebase cannot be interrupted, so honour a cancel before it instead of reporting one after it has landed.
        sync.check()?;
        self.rebase_to_trunk();
        self.post_fetch_cleanup(msg, &tracking_before, sync)
    }

    pub(crate) fn git_fetch_raw(&self, remote: &str, bookmark: &str) -> CoreResult<String> {
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
        let _ = self.rebase("@", "trunk()", RebaseMode::Branch);
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
                sync.check()?;
                // 100% safe: empty after rebase = content already in parent
                let _ = self.abandon(&c.commit_id.id);
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

#[cfg(test)]
mod tests {
    use jj_test::{init_jj_repo, run_jj_in};

    use crate::repo::Repo;
    use crate::types::CoreError;

    #[test]
    fn a_pull_canceled_after_its_fetch_does_not_rebase() {
        let temp_dir = init_jj_repo();
        let repo_path = temp_dir.path().join("repo");
        run_jj_in(
            &repo_path,
            &[
                "config",
                "set",
                "--repo",
                "revset-aliases.'trunk()'",
                "main",
            ],
        );
        run_jj_in(&repo_path, &["describe", "-m", "trunk"]);
        run_jj_in(&repo_path, &["bookmark", "create", "main", "-r", "@"]);
        run_jj_in(&repo_path, &["new", "root()", "-m", "work"]);
        let repo = Repo::open(&repo_path).expect("open repo");
        let before = repo.op_log().expect("op log").len();
        let sync = repo.sync_token();

        let result = repo.pull(
            &sync,
            |_| {
                sync.cancel();
                Ok(String::new())
            },
            None,
        );

        assert!(matches!(result, Err(CoreError::Canceled)));
        assert_eq!(repo.op_log().expect("op log").len(), before);
    }
}
