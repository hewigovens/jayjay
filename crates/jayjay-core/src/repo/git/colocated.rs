use std::collections::HashSet;
use std::fmt::Display;
use std::io::BufWriter;
use std::time::{Duration, Instant};

use futures::StreamExt as _;
use gix::bstr::BString;
use gix::index::entry::{Flags, Mode, Stat};
use jj_lib::backend::TreeValue;
use jj_lib::git::GitResetHeadError;
use jj_lib::matchers::EverythingMatcher;
use jj_lib::merged_tree::{MergedTree, TreeDiffEntry};
use jj_lib::repo::{ReadonlyRepo, Repo as _};
use jj_lib::store::Store;
use jj_lib::transaction::Transaction;

use crate::repo::Repo;
use crate::repo::support::block_on;
use crate::types::*;

const GIT_LOCK_TIMEOUT: Duration = Duration::from_secs(1);

impl Repo {
    pub(crate) fn is_colocated(&self, store: &Store) -> bool {
        jj_lib::git::get_git_backend(store)
            .is_ok_and(|backend| backend.open_git_repo_at_workdir(&self.path).is_ok())
    }

    /// A colocated checkout expects Git HEAD at @'s parent and Git refs at the bookmarks; the CLI does this after every command, jj-lib leaves it to the caller.
    pub(crate) fn sync_colocated_git(&self, tx: &mut Transaction) -> JayResult<()> {
        let repo_mut = tx.repo_mut();
        if !self.is_colocated(repo_mut.store()) {
            return Ok(());
        }
        if let Some(commit_id) = repo_mut
            .view()
            .get_wc_commit_id(self.workspace_name.as_ref())
            .cloned()
        {
            let wc_commit = repo_mut.store().get_commit(&commit_id).map_err(|error| {
                JayError::internal(format!("load working-copy commit: {error}"))
            })?;
            reset_head_with_lock_retry(|| {
                block_on(jj_lib::git::reset_head(
                    repo_mut,
                    &self.workspace_name,
                    &self.path,
                    &wc_commit,
                ))
            })?;
        }
        jj_lib::git::export_refs(repo_mut)
            .map_err(|error| JayError::internal(format!("export git refs: {error}")))?;
        Ok(())
    }

    /// Files a snapshot started tracking get intent-to-add entries so `git status` and `git diff` see them, as they would after a CLI snapshot.
    ///
    /// jj-lib's `update_intent_to_add` reads the index before locking it and can overwrite a concurrent `git add`; this reads it under `index.lock`.
    pub(crate) fn sync_colocated_index(
        &self,
        repo: &ReadonlyRepo,
        old_tree: &MergedTree,
        new_tree: &MergedTree,
    ) -> JayResult<()> {
        let Some(git_repo) = jj_lib::git::get_git_backend(repo.store())
            .ok()
            .and_then(|backend| backend.open_git_repo_at_workdir(&self.path).ok())
        else {
            return Ok(());
        };
        let (added_paths, removed_paths) = block_on(intent_to_add_changes(old_tree, new_tree))?;
        if added_paths.is_empty() && removed_paths.is_empty() {
            return Ok(());
        }
        // `jj util gc` errors on index entries whose blob is missing.
        let empty_blob = git_repo.write_blob(b"").map_err(index_error)?.detach();

        let index_path = git_repo.index_path();
        let lock = gix::lock::File::acquire_to_update_resource(
            &index_path,
            gix::lock::acquire::Fail::AfterDurationWithBackoff(GIT_LOCK_TIMEOUT),
            None,
        )
        .map_err(index_error)?;
        let mut index = if index_path.exists() {
            git_repo.open_index().map_err(index_error)?
        } else {
            gix::index::File::from_state(gix::index::State::new(git_repo.object_hash()), index_path)
        };
        // Look up before pushing: lookups need sorted entries.
        let added_paths: Vec<_> = added_paths
            .into_iter()
            .filter(|(path, _)| index.entry_index_by_path(path.as_ref()).is_err())
            .collect();
        for (path, mode) in added_paths {
            index.dangerously_push_entry(
                Stat::default(),
                empty_blob,
                Flags::INTENT_TO_ADD | Flags::EXTENDED,
                mode,
                path.as_ref(),
            );
        }
        index.remove_entries(|_, path, entry| {
            entry.flags.contains(Flags::INTENT_TO_ADD) && removed_paths.contains(path)
        });
        index.sort_entries();
        // A stale cache-tree makes a later `git add` write duplicate tree entries.
        index.remove_tree();

        let mut out = BufWriter::new(lock);
        index
            .write_to(&mut out, gix::index::write::Options::default())
            .map_err(index_error)?;
        out.into_inner()
            .map_err(|error| index_error(error.into_error()))?
            .commit()
            .map_err(|error| index_error(error.error))?;
        Ok(())
    }
}

async fn intent_to_add_changes(
    old_tree: &MergedTree,
    new_tree: &MergedTree,
) -> JayResult<(Vec<(BString, Mode)>, HashSet<BString>)> {
    let mut added_paths = Vec::new();
    let mut removed_paths = HashSet::new();
    let mut diff_stream = old_tree.diff_stream(new_tree, &EverythingMatcher);
    while let Some(TreeDiffEntry { path, values }) = diff_stream.next().await {
        let values = values.map_err(index_error)?;
        if values.before.is_absent() {
            let mode = match values.after.as_normal() {
                Some(TreeValue::File {
                    executable: true, ..
                }) => Mode::FILE_EXECUTABLE,
                Some(TreeValue::File { .. } | TreeValue::Symlink(_)) => Mode::FILE,
                _ => continue,
            };
            added_paths.push((BString::from(path.into_internal_string()), mode));
        } else if values.after.is_absent() {
            removed_paths.insert(BString::from(path.into_internal_string()));
        }
    }
    Ok((added_paths, removed_paths))
}

fn index_error(error: impl Display) -> JayError {
    JayError::internal(format!("update git index: {error}"))
}

/// Safe to retry: `reset_head` rebuilds the index from the commit instead of editing the one it read.
fn reset_head_with_lock_retry(
    mut reset_head: impl FnMut() -> Result<(), GitResetHeadError>,
) -> JayResult<()> {
    let deadline = Instant::now() + GIT_LOCK_TIMEOUT;
    loop {
        let result = reset_head();
        let remaining = deadline.saturating_duration_since(Instant::now());
        match result {
            Err(GitResetHeadError::Git(ref source))
                if !remaining.is_zero()
                    && source
                        .downcast_ref::<gix::Error>()
                        .is_some_and(gix::Error::is_retryable) =>
            {
                std::thread::sleep(remaining.min(Duration::from_millis(25)));
            }
            result => {
                return result
                    .map_err(|error| JayError::internal(format!("reset git head: {error}")));
            }
        }
    }
}
