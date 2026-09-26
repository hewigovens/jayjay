use std::sync::Arc;

use jj_lib::commit::Commit;
use jj_lib::matchers::FilesMatcher;
use jj_lib::repo::ReadonlyRepo;
use jj_lib::rewrite::{CommitWithSelection, restore_tree, squash_commits};

use super::Repo;
use super::mutations::combined_description;
use super::path_operands::{gitignore_pattern, reject_control_chars};
use super::support::block_on_result;
use crate::types::*;

impl Repo {
    /// Restore `paths` in `rev` from `from`'s tree when given (`jj restore --from` semantics, used to pick one parent of a merge), else from the auto-merged parent tree. `rev` is always the change being rewritten; `from` is only ever a content source.
    pub fn restore_files(&self, rev: &str, from: Option<&str>, paths: &[String]) -> CoreResult<()> {
        let _write = self.write_guard()?;
        self.restore(rev, paths, |repo| {
            from.map(|f| {
                self.resolve_commit(repo, f)
                    .and_then(|commit| self.follow_rewrites(repo, commit, f))
            })
            .transpose()
        })
    }

    /// Replace `rev`'s whole tree with `version`'s. Evolog versions are hidden predecessors, so `version` is taken exactly as named instead of following its rewrites to the current version.
    pub fn restore_version(&self, rev: &str, version: &str) -> CoreResult<()> {
        let _write = self.write_guard()?;
        self.restore(rev, &[], |repo| {
            self.resolve_commit(repo, version).map(Some)
        })
    }

    fn restore(
        &self,
        rev: &str,
        paths: &[String],
        resolve_source: impl FnOnce(&Arc<ReadonlyRepo>) -> CoreResult<Option<Commit>>,
    ) -> CoreResult<()> {
        self.refresh_working_copy()?;

        let repo = self.get_repo();
        let commit = self.follow_rewrites(&repo, self.resolve_commit(&repo, rev)?, rev)?;
        let source = resolve_source(&repo)?;
        self.ensure_commit_mutable(&repo, &commit, rev)?;
        let repo_paths = self.parse_repo_paths(paths)?;
        self.rewrite_existing_commit_with_tree(
            repo,
            commit,
            "restore files",
            true,
            "rewrite commit",
            move |repo, commit| {
                let source_tree = match &source {
                    Some(source) => source.tree(),
                    None => self.load_parent_tree(repo, commit, "load parent tree")?,
                };
                if repo_paths.is_empty() {
                    return Ok(source_tree);
                }
                let matcher = FilesMatcher::new(repo_paths.iter().map(|path| path.as_ref()));
                let old_tree = commit.tree();
                let new_tree = restore_tree(
                    &source_tree,
                    &old_tree,
                    "parent".to_owned(),
                    "current".to_owned(),
                    &matcher,
                );
                block_on_result("restore tree", new_tree)
            },
        )
    }

    /// Delete files from disk (working copy only). jj will pick up the deletion on next snapshot.
    pub fn delete_files(&self, paths: &[String]) -> CoreResult<()> {
        let _write = self.write_guard()?;
        for path in paths {
            let abs_path = self.path.join(path);
            if abs_path.exists() {
                std::fs::remove_file(&abs_path)
                    .or_else(|_| std::fs::remove_dir_all(&abs_path))
                    .map_err(|e| CoreError::Internal {
                        message: format!("delete {path}: {e}"),
                    })?;
            }
        }
        self.refresh_working_copy()
    }

    /// Add paths to .gitignore, then stop tracking them (`jj file untrack`).
    pub fn ignore_and_untrack(&self, paths: &[String]) -> CoreResult<()> {
        let _write = self.write_guard()?;
        // Reject control chars first: a newline would inject extra .gitignore patterns.
        reject_control_chars(paths)?;

        let gitignore_path = self.path.join(".gitignore");
        let existing = std::fs::read_to_string(&gitignore_path).unwrap_or_default();
        let mut lines_to_add = Vec::new();
        for path in paths {
            let pattern = gitignore_pattern(path);
            if !existing.lines().any(|line| line.trim() == pattern) {
                lines_to_add.push(pattern);
            }
        }
        if !lines_to_add.is_empty() {
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&gitignore_path)
                .map_err(|e| CoreError::Internal {
                    message: format!("open .gitignore: {e}"),
                })?;
            if !existing.is_empty() && !existing.ends_with('\n') {
                writeln!(file).ok();
            }
            for line in &lines_to_add {
                writeln!(file, "{line}").map_err(|e| CoreError::Internal {
                    message: format!("write .gitignore: {e}"),
                })?;
            }
        }

        self.refresh_working_copy()?;
        self.untrack_paths(&self.parse_repo_paths(paths)?)
    }

    /// `jj squash --from rev --into @ -- paths`: the named files' changes move to the working copy; a source left empty is abandoned and its description joins `@`'s.
    pub fn move_to_working_copy(&self, rev: &str, paths: &[String]) -> CoreResult<()> {
        let _write = self.write_guard()?;
        self.refresh_working_copy()?;
        let repo = self.get_repo();
        let source = self.follow_rewrites(&repo, self.resolve_commit(&repo, rev)?, rev)?;
        self.ensure_commit_mutable(&repo, &source, rev)?;
        let destination = self.working_copy_commit(&repo)?;
        self.ensure_commit_mutable(&repo, &destination, "@")?;
        if source.id() == destination.id() {
            return Err(CoreError::internal("cannot move files from @ to @"));
        }
        let repo_paths = self.parse_repo_paths(paths)?;
        let matcher = FilesMatcher::new(repo_paths.iter().map(|path| path.as_ref()));
        let parent_tree = self.load_parent_tree(&repo, &source, "load parent tree")?;
        let selected_tree = block_on_result(
            "select files",
            restore_tree(
                &source.tree(),
                &parent_tree,
                "source".to_owned(),
                "parent".to_owned(),
                &matcher,
            ),
        )?;
        // jj-lib reads an empty selection of an empty source as "everything" and abandons the source, so refuse it here like split does.
        if selected_tree.tree_ids() == parent_tree.tree_ids() {
            return Err(CoreError::internal(
                "none of the selected files differ from the parent",
            ));
        }
        let selection = CommitWithSelection {
            commit: source.clone(),
            selected_tree,
            parent_tree,
        };
        let mut tx = repo.start_transaction();
        let squashed = block_on_result(
            "move files to working copy",
            squash_commits(tx.repo_mut(), &[selection], &destination, false),
        )?;
        let Some(squashed) = squashed else {
            return Err(CoreError::internal(
                "none of the selected files differ from the parent",
            ));
        };
        let description = if squashed.abandoned_commits.is_empty() {
            destination.description().to_owned()
        } else {
            combined_description(destination.description(), source.description())
        };
        let write = squashed.commit_builder.set_description(description).write();
        block_on_result("write working-copy change", write)?;
        self.commit_transaction_rebase(tx, "move files to working copy")
    }
}
