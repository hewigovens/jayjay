use std::io::Write;
use std::process::Stdio;

use crate::repo::{Repo, subprocess_command};
use crate::types::*;

impl Repo {
    /// List files currently tracked by Git LFS in the checked-out tree.
    pub fn tracked_git_lfs_files(&self) -> CoreResult<Vec<String>> {
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

    /// Filter the provided repo-relative paths down to those matched by Git LFS attributes.
    pub fn git_lfs_paths(&self, paths: &[String]) -> CoreResult<Vec<String>> {
        if paths.is_empty() {
            return Ok(vec![]);
        }

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
