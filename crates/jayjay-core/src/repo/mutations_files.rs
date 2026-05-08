use super::Repo;
use super::support::block_on_result;
use crate::types::*;

impl Repo {
    pub fn restore_files(&self, rev: &str, paths: &[String]) -> CoreResult<()> {
        self.refresh_working_copy()?;

        let repo = self.get_repo();
        let commit = self.resolve_commit(&repo, rev)?;
        let is_wc = repo
            .view()
            .get_wc_commit_id(self.workspace_name.as_ref())
            .is_some_and(|id| id == commit.id());

        if is_wc {
            let mut args = vec!["restore", "--from", "@-"];
            args.extend(paths.iter().map(|s| s.as_str()));
            self.run_jj_reload(&args)
        } else {
            let repo_paths = self.parse_repo_paths(paths)?;
            self.rewrite_existing_commit_with_tree(
                repo,
                commit,
                "restore files",
                true,
                "rewrite commit",
                move |repo, commit| {
                    let old_tree = commit.tree();
                    let parent_tree = self.load_parent_tree(repo, commit, "load parent tree")?;
                    let matcher = jj_lib::matchers::FilesMatcher::new(
                        repo_paths.iter().map(|path| path.as_ref()),
                    );
                    let new_tree = jj_lib::rewrite::restore_tree(
                        &parent_tree,
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

    pub fn restore_revision_into_working_copy(&self, rev: &str) -> CoreResult<()> {
        self.run_jj_reload(&["restore", "--from", rev, "--into", "@"])
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
        let gitignore_path = self.path.join(".gitignore");
        let existing = std::fs::read_to_string(&gitignore_path).unwrap_or_default();
        let mut lines_to_add = Vec::new();
        for path in paths {
            if !existing.lines().any(|line| line.trim() == path.as_str()) {
                lines_to_add.push(path.as_str());
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

        let mut args = vec!["file", "untrack"];
        args.extend(paths.iter().map(|s| s.as_str()));
        self.run_jj_reload(&args)
    }

    /// Move files from a change to working copy using `jj squash --from rev --into @`.
    pub fn move_to_working_copy(&self, rev: &str, paths: &[String]) -> CoreResult<()> {
        let mut args = vec!["squash", "--from", rev, "--into", "@"];
        args.extend(paths.iter().map(|s| s.as_str()));
        self.run_jj_reload(&args)
    }
}
