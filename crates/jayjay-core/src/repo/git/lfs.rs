use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::process::Stdio;
use std::sync::Arc;

use jj_lib::repo::ReadonlyRepo;

use crate::repo::{Repo, subprocess_command};
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
    fn tracked_git_lfs_files(&self) -> CoreResult<Vec<String>> {
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
    pub fn git_lfs_paths(&self, paths: &[String]) -> CoreResult<Vec<String>> {
        if paths.is_empty() {
            return Ok(vec![]);
        }
        let repo = self.get_repo();
        // Held across the subprocesses on purpose: a second caller for the same loaded repo waits for the answer instead of spawning git again.
        let mut cache = self.lfs_cache.lock().unwrap();
        cache.reset_if_stale(&repo);

        let unknown: Vec<String> = paths
            .iter()
            .filter(|path| !cache.attribute_says_lfs.contains_key(path.as_str()))
            .cloned()
            .collect();
        if !unknown.is_empty() {
            let lfs: HashSet<String> = self.check_attr_lfs_paths(&unknown)?.into_iter().collect();
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

    /// Paths whose `.gitattributes` set `filter=lfs`. Repository-controlled, so
    /// callers must confirm real LFS registration before acting on the result.
    fn check_attr_lfs_paths(&self, paths: &[String]) -> CoreResult<Vec<String>> {
        let mut child = subprocess_command("git")
            .current_dir(&self.path)
            .args(["check-attr", "--stdin", "filter"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| CoreError::Internal {
                message: format!("git check-attr: {e}"),
            })?;

        // Feed stdin on a side thread so we can drain stdout concurrently and avoid a pipe deadlock.
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
}

fn parse_git_check_attr_lfs_paths(stdout: &str) -> Vec<String> {
    stdout
        .lines()
        .filter_map(|line| line.strip_suffix(": filter: lfs"))
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::parse_git_check_attr_lfs_paths;

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
}
