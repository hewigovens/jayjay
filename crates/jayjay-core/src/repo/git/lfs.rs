use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use gix::attrs::StateRef;
use gix::glob::pattern::Case;
use gix::worktree::stack::state::attributes::Source;
use jj_lib::git::get_git_repo;
use jj_lib::repo::{ReadonlyRepo, Repo as _};

use crate::repo::Repo;
use crate::types::*;

/// Keyed on the loaded repo handle: a refresh replaces it even when Git attributes or the index changed without a new jj operation.
#[derive(Default)]
pub(in crate::repo) struct LfsCache {
    repo: Option<Arc<ReadonlyRepo>>,
    attribute_says_lfs: HashMap<String, bool>,
    /// Lazy: a repo whose paths never claim `filter=lfs` never runs `git lfs ls-files`.
    tracked: Option<HashSet<String>>,
}

impl LfsCache {
    fn reset_if_stale(&mut self, repo: &Arc<ReadonlyRepo>) {
        if self
            .repo
            .as_ref()
            .is_some_and(|cached| Arc::ptr_eq(cached, repo))
        {
            return;
        }
        *self = Self {
            repo: Some(repo.clone()),
            ..Self::default()
        };
    }
}

impl Repo {
    /// List files currently tracked by Git LFS in the checked-out tree.
    fn tracked_git_lfs_files(&self) -> JayResult<Vec<String>> {
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

    /// Filter repo-relative paths to those actually stored as Git LFS objects. A
    /// `.gitattributes filter=lfs` line is repo-controlled (a source file could fake it
    /// to hide its diff), so trust only what `git lfs ls-files` reports, not the attribute.
    pub fn git_lfs_paths(&self, paths: &[String]) -> JayResult<Vec<String>> {
        if paths.is_empty() {
            return Ok(vec![]);
        }
        let repo = self.get_repo();
        // Held across the lookups on purpose: a second caller for the same loaded repo waits for the answer instead of repeating them.
        let mut cache = self.lfs_cache.lock().unwrap();
        cache.reset_if_stale(&repo);

        let unknown: Vec<String> = paths
            .iter()
            .filter(|path| !cache.attribute_says_lfs.contains_key(path.as_str()))
            .cloned()
            .collect();
        if !unknown.is_empty() {
            let lfs = self.attribute_lfs_paths(&repo, &unknown)?;
            for path in unknown {
                let is_lfs = lfs.contains(&path);
                cache.attribute_says_lfs.insert(path, is_lfs);
            }
        }

        let candidates: Vec<String> = paths
            .iter()
            .filter(|path| cache.attribute_says_lfs.get(path.as_str()) == Some(&true))
            .cloned()
            .collect();
        if candidates.is_empty() {
            return Ok(vec![]);
        }
        if cache.tracked.is_none() {
            cache.tracked = Some(self.tracked_git_lfs_files()?.into_iter().collect());
        }
        let tracked = cache.tracked.as_ref().expect("tracked paths loaded");
        Ok(candidates
            .into_iter()
            .filter(|path| tracked.contains(path))
            .collect())
    }

    /// Paths whose `.gitattributes` set `filter=lfs`. Repository-controlled, so callers must confirm real LFS registration before acting on the result.
    fn attribute_lfs_paths(
        &self,
        repo: &ReadonlyRepo,
        paths: &[String],
    ) -> JayResult<HashSet<String>> {
        let error = |error: &dyn std::fmt::Display| {
            JayError::internal(format!("read git attributes: {error}"))
        };
        let git_repo = get_git_repo(repo.store()).map_err(|e| error(&e))?;
        let index = git_repo.index_or_empty().map_err(|e| error(&e))?;
        let globals = git_repo
            .attributes_only(
                &gix::index::State::new(git_repo.object_hash()),
                Source::WorktreeThenIdMapping,
            )
            .map_err(|e| error(&e))?
            .detach();
        let case = if git_repo.config_snapshot().boolean("core.ignoreCase") == Some(true) {
            Case::Fold
        } else {
            Case::Sensitive
        };
        let state = globals.state().clone();
        let index_attributes = state.id_mappings_from_index(&index, index.path_backing(), case);
        // Rooted at the workspace so non-colocated workspaces, whose git store is bare, read their checked-out `.gitattributes` too.
        let mut attributes =
            gix::worktree::Stack::new(&self.path, state, case, Vec::new(), index_attributes);
        let mut outcome = attributes.selected_attribute_matches(["filter"]);
        let mut lfs = HashSet::new();
        for path in paths {
            attributes
                .at_entry(path.as_str(), None, &git_repo.objects)
                .map_err(|e| error(&e))?
                .matching_attributes(&mut outcome);
            if outcome
                .iter_selected()
                .any(|m| m.assignment.state == StateRef::from_bytes(b"lfs"))
            {
                lfs.insert(path.clone());
            }
        }
        Ok(lfs)
    }
}
