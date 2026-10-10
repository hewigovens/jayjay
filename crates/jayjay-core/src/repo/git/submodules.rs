use crate::repo::Repo;
use crate::types::*;

impl Repo {
    pub fn submodule_statuses(&self) -> JayResult<Vec<GitSubmoduleStatus>> {
        let output = self.command_output(
            "git",
            &[
                "status",
                "--porcelain=v2",
                "-z",
                "--ignore-submodules=none",
                "--untracked-files=no",
            ],
            "git status",
        )?;
        self.ensure_success(&output, "git status")?;

        Ok(parse_git_status_submodule_statuses(
            &String::from_utf8_lossy(&output.stdout),
        ))
    }

    pub fn commit_safe_submodule_updates(
        &self,
        message: &str,
        paths: &[String],
    ) -> JayResult<String> {
        let _write = self.write_guard()?;
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

        self.git_import()?;

        if self.has_jj_working_copy_changes()? {
            self.jj_commit(message)?;
            Ok("Committed safe submodule updates and working-copy changes.".to_owned())
        } else {
            Ok("Committed safe submodule updates.".to_owned())
        }
    }

    fn has_jj_working_copy_changes(&self) -> JayResult<bool> {
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
    let mut records = stdout.split('\0');
    let mut statuses = Vec::new();
    while let Some(record) = records.next() {
        let field_count = match record.split_once(' ') {
            Some(("1", _)) => 9,
            Some(("2", _)) => {
                // A rename's original path follows as its own NUL-terminated record.
                records.next();
                10
            }
            _ => continue,
        };
        let fields: Vec<&str> = record.splitn(field_count, ' ').collect();
        if fields.len() != field_count {
            continue;
        }
        let state = fields[2].as_bytes();
        if state.first() != Some(&b'S') {
            continue;
        }
        statuses.push(GitSubmoduleStatus {
            path: fields[field_count - 1].to_owned(),
            has_new_commits: state.get(1) == Some(&b'C'),
            has_modified_content: state.get(2) == Some(&b'M'),
            has_untracked_content: state.get(3) == Some(&b'U'),
        });
    }
    statuses
}

#[cfg(test)]
mod tests {
    use super::parse_git_status_submodule_statuses;

    #[test]
    fn parses_git_status_submodule_statuses() {
        let statuses = parse_git_status_submodule_statuses(
            "1 .M SC.. 160000 160000 160000 2196000605e45d91097147c9c71f26b72af58003 2196000605e45d91097147c9c71f26b72af58003 crypto/tests/wycheproof\0\
             1 .M N... 100644 100644 100644 a660a4976efe880bae7982ee410b9e0dc59ac983 a660a4976efe880bae7982ee410b9e0dc59ac983 README.md\0\
             2 R. S..U 160000 160000 160000 a660a4976efe880bae7982ee410b9e0dc59ac983 a660a4976efe880bae7982ee410b9e0dc59ac983 R100 vendor/my lib\0\
             1 .M SCM. 160000 160000 160000 a660a4976efe880bae7982ee410b9e0dc59ac983 a660a4976efe880bae7982ee410b9e0dc59ac983 old\0",
        );

        let summary: Vec<_> = statuses
            .iter()
            .map(|status| {
                (
                    status.path.as_str(),
                    status.has_new_commits,
                    status.has_modified_content,
                    status.has_untracked_content,
                )
            })
            .collect();
        assert_eq!(
            summary,
            [
                ("crypto/tests/wycheproof", true, false, false),
                ("vendor/my lib", false, false, true),
            ]
        );
    }
}
