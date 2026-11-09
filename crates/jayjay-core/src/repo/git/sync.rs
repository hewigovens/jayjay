use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use jj_lib::backend::CommitId;
use jj_lib::dsl_util::format_string;
use jj_lib::git::REMOTE_NAME_FOR_LOCAL_GIT_REPO;
use jj_lib::object_id::ObjectId as _;
use jj_lib::ref_name::{RefName, RemoteName};
use jj_lib::repo::{ReadonlyRepo, Repo as _};
use jj_lib::settings::UserSettings;

use crate::repo::bookmarks::bookmark_target_without_commit;
use crate::repo::support::block_on_result;
use crate::repo::tags::tag_remotes;
use crate::repo::{Repo, SyncToken};
use crate::types::*;

impl Repo {
    /// Returns a message describing what happened (warnings, errors, or success).
    /// Tracks only an explicitly requested bookmark before pushing.
    pub fn git_push(&self, bookmark: &str, sync: &SyncToken) -> JayResult<String> {
        let _enter = sync.enter();
        if !bookmark.is_empty() {
            let _ = self.track_bookmark(bookmark, &self.default_remote());
        }

        let pattern = exact_pattern(bookmark);
        let mut args = vec!["git", "push"];
        if !bookmark.is_empty() {
            args.extend(["--bookmark", &pattern]);
        }
        self.run_remote_command(&args, "git push failed", sync)
    }

    /// A locally deleted tag that a remote still tracks pushes as a deletion; jj targets one remote per call, so each tracked remote gets its own push.
    pub fn git_push_tag(&self, tag: &str, sync: &SyncToken) -> JayResult<String> {
        let _enter = sync.enter();
        let pattern = exact_pattern(tag);
        let remotes = tag_remotes(self.get_repo().view(), RefName::new(tag));
        if remotes.is_empty() {
            return self.run_remote_command(
                &["git", "push", "--tag", &pattern],
                "git push failed",
                sync,
            );
        }
        let mut messages = Vec::new();
        for remote in &remotes {
            let remote = format!("--remote={remote}");
            messages.push(self.run_remote_command(
                &["git", "push", &remote, "--tag", &pattern],
                "git push failed",
                sync,
            )?);
        }
        Ok(messages.join("\n"))
    }

    pub(super) fn run_remote_command(
        &self,
        args: &[&str],
        failure: &str,
        sync: &SyncToken,
    ) -> JayResult<String> {
        let output = self.run_jj_output(args)?;
        self.ensure_success(&output, failure)?;
        self.reload()?;
        sync.check()?;
        Ok(combine_output(
            &Self::stdout_text(&output),
            &Self::stderr_text(&output),
        ))
    }

    /// Track and push several bookmarks in one `jj git push`, with a single
    /// reload. Used by the stacked-PR submit so each PR head/base exists at once.
    pub(crate) fn git_push_bookmarks(&self, bookmarks: &[&str]) -> JayResult<String> {
        if bookmarks.is_empty() {
            return Ok("Nothing to push.".to_owned());
        }
        let remote = self.default_remote();
        for bookmark in bookmarks {
            let _ = self.track_bookmark(bookmark, &remote);
        }
        // `--bookmark` creates and tracks new remote bookmarks on its own; jj 0.42
        // has no `--allow-new` flag.
        let patterns: Vec<String> = bookmarks.iter().map(|name| exact_pattern(name)).collect();
        let mut args = vec!["git", "push"];
        for pattern in &patterns {
            args.extend(["--bookmark", pattern]);
        }
        let output = self.run_jj_output(&args)?;
        self.ensure_success(&output, "git push failed")?;
        self.reload()?;
        Ok(combine_output(
            &Self::stdout_text(&output),
            &Self::stderr_text(&output),
        ))
    }

    /// Fetch without changing bookmark tracking, rebase, and clean up merged bookmarks, unless `jayjay.sync-command` replaces all of that.
    pub fn git_fetch(&self, sync: &SyncToken) -> JayResult<FetchResult> {
        if let Some(args) = configured_command(self.get_repo().settings(), SYNC_COMMAND) {
            let _enter = sync.enter();
            let args: Vec<&str> = args.iter().map(String::as_str).collect();
            return Ok(FetchResult {
                message: self.run_remote_command(&args, "sync failed", sync)?,
                abandoned_changes: Vec::new(),
                abandoned_bookmarks: Vec::new(),
                suggest_abandon_bookmarks: Vec::new(),
            });
        }
        self.pull(sync, |repo| repo.git_fetch_raw("", ""), None)
    }

    /// Fetch a specific bookmark, auto-track it, rebase, and clean up.
    pub fn git_pull_bookmark(&self, bookmark: &str, sync: &SyncToken) -> JayResult<FetchResult> {
        self.pull(
            sync,
            |repo| repo.git_fetch_raw("", bookmark),
            Some(bookmark),
        )
    }

    fn pull(
        &self,
        sync: &SyncToken,
        fetch: impl FnOnce(&Self) -> JayResult<String>,
        track: Option<&str>,
    ) -> JayResult<FetchResult> {
        let _enter = sync.enter();
        let before = self.get_repo();
        let msg = fetch(self)?;
        // The in-process rebase cannot be interrupted, so honour a cancel before it instead of reporting one after it has landed.
        sync.check()?;
        let cleanup = self.write_guard().and_then(|_write| {
            if let Some((bookmark, remote)) =
                track.and_then(|bookmark| Some((bookmark, self.bookmark_remote(bookmark)?)))
            {
                let _ = self.track_bookmark(bookmark, &remote);
            }
            self.rebase_to_trunk();
            let landed = self.abandon_landed_changes(&before)?;
            let (abandoned, suggested) = self.post_fetch_cleanup(&before, sync)?;
            Ok((landed, abandoned, suggested))
        });
        fetch_result(msg, cleanup)
    }

    /// `git.fetch` may name remotes other than the push default.
    fn bookmark_remote(&self, bookmark: &str) -> Option<String> {
        let repo = self.get_repo();
        let name = RefName::new(bookmark);
        self.fetch_remotes().into_iter().find(|remote| {
            !repo
                .view()
                .get_remote_bookmark(name.to_remote_symbol(RemoteName::new(remote)))
                .is_absent()
        })
    }

    pub(crate) fn git_fetch_raw(&self, remote: &str, bookmark: &str) -> JayResult<String> {
        let mut args = vec!["git", "fetch"];
        if !remote.is_empty() {
            args.extend(["--remote", remote]);
        }
        let pattern = exact_pattern(bookmark);
        if !bookmark.is_empty() {
            args.extend(["-b", &pattern]);
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

    fn deleted_remote_bookmarks(
        &self,
        before: &ReadonlyRepo,
    ) -> BTreeMap<String, HashSet<CommitId>> {
        let after = self.get_repo();
        let present_names: HashSet<_> = after
            .view()
            .all_remote_bookmarks()
            .filter(|(symbol, remote)| {
                symbol.remote != REMOTE_NAME_FOR_LOCAL_GIT_REPO
                    && remote.is_tracked()
                    && remote.target.is_present()
            })
            .map(|(symbol, _)| symbol.name)
            .collect();
        // Deleted remote refs stay tracked in jj, so tracking alone cannot identify a deletion.
        before
            .view()
            .all_remote_bookmarks()
            .filter(|(symbol, remote)| {
                symbol.remote != REMOTE_NAME_FOR_LOCAL_GIT_REPO
                    && remote.is_tracked()
                    && remote.target.is_present()
                    && after.view().get_remote_bookmark(*symbol).is_tracked()
                    && after.view().get_remote_bookmark(*symbol).target.is_absent()
                    && !present_names.contains(symbol.name)
            })
            .fold(BTreeMap::new(), |mut deleted, (symbol, remote)| {
                deleted
                    .entry(symbol.name.as_str().to_owned())
                    .or_default()
                    .extend(remote.target.present_adds().cloned());
                deleted
            })
    }

    fn post_fetch_cleanup(
        &self,
        before: &ReadonlyRepo,
        sync: &SyncToken,
    ) -> JayResult<(Vec<String>, Vec<String>)> {
        sync.check()?;
        self.refresh_working_copy()?;
        let repo = self.get_repo();
        let deleted = self.deleted_remote_bookmarks(before);
        let mut candidates: HashMap<CommitId, BTreeSet<String>> = HashMap::new();
        for (name, old_targets) in &deleted {
            for id in repo
                .view()
                .get_local_bookmark(RefName::new(name))
                .present_adds()
            {
                candidates
                    .entry(id.clone())
                    .or_default()
                    .insert(name.clone());
            }
            for id in old_targets {
                let commit = repo.store().get_commit(id).map_err(|error| {
                    JayError::internal(format!("load deleted remote target: {error}"))
                })?;
                let targets = block_on_result(
                    "resolve deleted remote target",
                    repo.resolve_change_id(commit.change_id()),
                )?;
                if targets.is_some_and(|targets| targets.visible_with_offsets().next().is_some()) {
                    // A locally split remote commit has no single successor; its bookmark targets still cover the moved case.
                    let Ok(commit) = self.follow_rewrites(&repo, commit, &id.hex()) else {
                        continue;
                    };
                    candidates
                        .entry(commit.id().clone())
                        .or_default()
                        .insert(name.clone());
                }
            }
        }
        let mut tx = repo.start_transaction();
        let mut abandoned = BTreeSet::new();
        let mut suggested = BTreeSet::new();
        for (id, names) in candidates {
            let commit = repo
                .store()
                .get_commit(&id)
                .map_err(|error| JayError::internal(format!("load cleanup candidate: {error}")))?;
            if repo.view().wc_commit_ids().values().any(|wc| *wc == id)
                || self.is_commit_immutable(&repo, &commit)?
            {
                continue;
            }
            if commit.has_conflict() {
                suggested.extend(names);
            } else if block_on_result("check cleanup candidate", commit.is_empty(repo.index()))? {
                for name in &names {
                    let target = tx.repo().view().get_local_bookmark(RefName::new(name));
                    if let Some(target) = bookmark_target_without_commit(target, &id) {
                        if target.is_absent() {
                            abandoned.insert(name.clone());
                        }
                        self.set_bookmark_target(tx.repo_mut(), name, target);
                    }
                }
                tx.repo_mut().record_abandoned_commit(&commit);
            }
        }
        sync.check()?;
        if tx.repo().has_changes() {
            self.commit_transaction_rebase(tx, "clean up merged bookmarks")?;
        }
        Ok((
            abandoned.into_iter().collect(),
            suggested.into_iter().collect(),
        ))
    }
}

const SYNC_COMMAND: &str = "jayjay.sync-command";

/// A jj argv from config, such as `["sync"]` for a backend with its own sync command.
pub(super) fn configured_command(
    settings: &UserSettings,
    key: &'static str,
) -> Option<Vec<String>> {
    settings
        .get::<Vec<String>>(key)
        .ok()
        .filter(|args| !args.is_empty())
}

/// A cleanup failure is reported inside the successful fetch; only a cancel is still an error, since nothing was applied.
fn fetch_result(
    message: String,
    cleanup: JayResult<(Vec<String>, Vec<String>, Vec<String>)>,
) -> JayResult<FetchResult> {
    let mut result = FetchResult {
        message,
        abandoned_changes: Vec::new(),
        abandoned_bookmarks: Vec::new(),
        suggest_abandon_bookmarks: Vec::new(),
    };
    match cleanup {
        Ok((landed, abandoned, suggested)) => {
            result.abandoned_changes = landed;
            result.abandoned_bookmarks = abandoned;
            result.suggest_abandon_bookmarks = suggested;
        }
        Err(JayError::Canceled) => return Err(JayError::Canceled),
        Err(error) => result
            .message
            .push_str(&format!("\nPost-fetch cleanup failed: {error}")),
    }
    Ok(result)
}

fn exact_pattern(name: &str) -> String {
    format!("exact:{}", format_string(name))
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
    use jj_test::{init_jj_repo, run_git, run_jj_in};

    use crate::repo::Repo;
    use crate::types::JayError;

    #[test]
    fn a_cleanup_failure_keeps_the_successful_fetch_result() {
        let dir = init_jj_repo();
        let path = dir.path().join("repo");
        let origin = dir.path().join("origin.git");
        run_git(dir.path(), &["init", "--bare", origin.to_str().unwrap()]);
        run_jj_in(&path, &["describe", "-m", "main"]);
        run_jj_in(&path, &["bookmark", "create", "main"]);
        run_jj_in(
            &path,
            &["git", "remote", "add", "origin", origin.to_str().unwrap()],
        );
        run_jj_in(&path, &["git", "push", "-b", "main"]);
        run_jj_in(&path, &["new"]);
        let target = run_git(&origin, &["rev-parse", "refs/heads/main"]);
        run_git(
            &origin,
            &[
                "update-ref",
                "refs/heads/fresh",
                String::from_utf8(target.stdout).unwrap().trim(),
            ],
        );
        let repo = Repo::open(&path).unwrap();
        let wc = path.join(".jj/working_copy");
        let backup = dir.path().join("working_copy");
        let result = repo.pull(
            &repo.sync_token(),
            |repo| {
                let message = repo.git_fetch_raw("origin", "")?;
                std::fs::rename(&wc, &backup).unwrap();
                Ok(message)
            },
            None,
        );
        std::fs::rename(&backup, &wc).unwrap();
        let result = result.expect("fetch succeeded despite cleanup failure");
        assert!(result.message.contains("fresh"), "{}", result.message);
        assert!(
            result.message.contains("Post-fetch cleanup failed:"),
            "{}",
            result.message
        );
        assert!(result.abandoned_bookmarks.is_empty());
        assert!(
            repo.list_bookmarks()
                .unwrap()
                .iter()
                .any(|b| b.name == "fresh")
        );
    }

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

        assert!(matches!(result, Err(JayError::Canceled)));
        assert_eq!(repo.op_log().expect("op log").len(), before);
    }
}
