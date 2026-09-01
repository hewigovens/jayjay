use jj_lib::object_id::ObjectId;

use super::Repo;
use super::path_operands::{fileset_literal, gitignore_pattern, reject_control_chars};
use super::support::block_on_result;
use crate::types::*;

impl Repo {
    /// Restore `paths` in `rev` from `from`'s tree when given (`jj restore --from` semantics, used to pick one parent of a merge), else from the auto-merged parent tree. `rev` is always the change being rewritten; `from` is only ever a content source.
    pub fn restore_files(&self, rev: &str, from: Option<&str>, paths: &[String]) -> CoreResult<()> {
        self.refresh_working_copy()?;

        let repo = self.get_repo();
        let commit = self.follow_rewrites(&repo, self.resolve_commit(&repo, rev)?, rev)?;
        let source = from
            .map(|f| {
                self.resolve_commit(&repo, f)
                    .and_then(|commit| self.follow_rewrites(&repo, commit, f))
            })
            .transpose()?;
        let is_wc = repo
            .view()
            .get_wc_commit_id(self.workspace_name.as_ref())
            .is_some_and(|id| id == commit.id());

        if is_wc {
            // Resolving the source up front turns it into a fixed hex operand, so an option- or revset-shaped string can never become extra CLI syntax.
            let from_arg = source.map_or_else(|| "@-".to_owned(), |c| c.id().hex());
            let operands: Vec<String> = paths.iter().map(|p| fileset_literal(p)).collect();
            let mut args = vec!["restore", "--from", from_arg.as_str(), "--"];
            args.extend(operands.iter().map(String::as_str));
            self.run_jj_reload(&args)
        } else {
            // The working-copy branch above shells out to jj, which enforces immutability itself; this direct jj-lib rewrite must refuse immutable targets on its own.
            self.ensure_commit_mutable(&repo, &commit, rev)?;
            let repo_paths = self.parse_repo_paths(paths)?;
            self.rewrite_existing_commit_with_tree(
                repo,
                commit,
                "restore files",
                true,
                "rewrite commit",
                move |repo, commit| {
                    let old_tree = commit.tree();
                    let source_tree = match &source {
                        Some(source) => source.tree(),
                        None => self.load_parent_tree(repo, commit, "load parent tree")?,
                    };
                    let matcher = jj_lib::matchers::FilesMatcher::new(
                        repo_paths.iter().map(|path| path.as_ref()),
                    );
                    let new_tree = jj_lib::rewrite::restore_tree(
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
    }

    /// Delete files from disk (working copy only). jj will pick up the deletion on next snapshot.
    pub fn delete_files(&self, paths: &[String]) -> CoreResult<()> {
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

    /// Add paths to .gitignore and untrack them via `jj file untrack`.
    pub fn ignore_and_untrack(&self, paths: &[String]) -> CoreResult<()> {
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

        let operands: Vec<String> = paths.iter().map(|p| fileset_literal(p)).collect();
        let mut args = vec!["file", "untrack", "--"];
        args.extend(operands.iter().map(String::as_str));
        self.run_jj_reload(&args)
    }

    /// Move files from a change to working copy using `jj squash --from rev --into @`.
    pub fn move_to_working_copy(&self, rev: &str, paths: &[String]) -> CoreResult<()> {
        let operands: Vec<String> = paths.iter().map(|p| fileset_literal(p)).collect();
        let mut args = vec!["squash", "--from", rev, "--into", "@", "--"];
        args.extend(operands.iter().map(String::as_str));
        self.run_jj_reload(&args)
    }
}
