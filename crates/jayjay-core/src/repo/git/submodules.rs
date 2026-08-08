use crate::repo::Repo;
use crate::types::*;

impl Repo {
    pub fn submodule_statuses(&self) -> CoreResult<Vec<GitSubmoduleStatus>> {
        let output = self.command_output(
            "git",
            &[
                "status",
                "--porcelain=v2",
                "--ignore-submodules=none",
                "--untracked-files=no",
            ],
            "git status",
        )?;
        self.ensure_success(&output, "git status")?;

        Ok(parse_git_status_submodule_statuses(&Self::stdout_text(
            &output,
        )))
    }

    pub fn commit_safe_submodule_updates(
        &self,
        message: &str,
        paths: &[String],
    ) -> CoreResult<String> {
        if paths.is_empty() {
            return Ok("No safe submodule updates to commit.".to_owned());
        }

        let mut add_args = vec!["add", "--"];
        add_args.extend(paths.iter().map(String::as_str));
        let output = self.command_output("git", &add_args, "git add submodule updates")?;
        self.ensure_success(&output, "git add submodule updates")?;

        let mut commit_args = vec!["commit", "-m", message, "--only", "--"];
        commit_args.extend(paths.iter().map(String::as_str));
        let output = self.command_output("git", &commit_args, "git commit submodule updates")?;
        self.ensure_success(&output, "git commit submodule updates")?;

        self.run_jj_reload(&["git", "import"])?;

        if self.has_jj_working_copy_changes()? {
            self.jj_commit(message)?;
            Ok("Committed safe submodule updates and working-copy changes.".to_owned())
        } else {
            Ok("Committed safe submodule updates.".to_owned())
        }
    }

    fn has_jj_working_copy_changes(&self) -> CoreResult<bool> {
        self.refresh_working_copy()?;
        let detail = self.show_summary("@")?;
        if !detail.diff.is_empty() {
            return Ok(true);
        }
        let conflicts = self.resolve_list("@").unwrap_or_default();
        Ok(!conflicts.is_empty())
    }
}

fn parse_git_status_submodule_statuses(stdout: &str) -> Vec<GitSubmoduleStatus> {
    stdout
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            if parts.next()? != "1" {
                return None;
            }

            let _xy = parts.next()?;
            let submodule_state = parts.next()?;
            if !submodule_state.starts_with('S') {
                return None;
            }

            let path = parts.last()?.to_owned();
            let state: Vec<char> = submodule_state.chars().collect();
            Some(GitSubmoduleStatus {
                path,
                has_new_commits: state.get(1) == Some(&'C'),
                has_modified_content: state.get(2) == Some(&'M'),
                has_untracked_content: state.get(3) == Some(&'U'),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::parse_git_status_submodule_statuses;

    #[test]
    fn parses_git_status_submodule_statuses() {
        let statuses = parse_git_status_submodule_statuses(
            "1 .M SC.. 160000 160000 160000 2196000605e45d91097147c9c71f26b72af58003 2196000605e45d91097147c9c71f26b72af58003 crypto/tests/wycheproof\n\
             1 .M S..U 160000 160000 160000 a660a4976efe880bae7982ee410b9e0dc59ac983 a660a4976efe880bae7982ee410b9e0dc59ac983 vendor/secp256k1-zkp\n",
        );

        assert_eq!(statuses.len(), 2);
        assert_eq!(statuses[0].path, "crypto/tests/wycheproof");
        assert!(statuses[0].has_new_commits);
        assert!(!statuses[0].has_modified_content);
        assert!(!statuses[0].has_untracked_content);
        assert_eq!(statuses[1].path, "vendor/secp256k1-zkp");
        assert!(!statuses[1].has_new_commits);
        assert!(!statuses[1].has_modified_content);
        assert!(statuses[1].has_untracked_content);
    }
}
