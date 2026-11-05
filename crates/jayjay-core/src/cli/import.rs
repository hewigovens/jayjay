use std::path::Path;

use clap::Parser;

use crate::{JayResult, Repo};

/// Import a GitHub pull request or GitLab merge request into a new jj workspace
#[derive(Debug, PartialEq, Parser)]
#[command(disable_version_flag = true)]
pub(super) struct ImportCommand {
    /// Pull request or merge request URL
    url: String,

    /// Path to the jj repository (default: current directory)
    #[arg(long, default_value = ".", allow_hyphen_values = true)]
    repo: String,

    /// Workspace name (default: pr-<number>)
    #[arg(long, allow_hyphen_values = true)]
    name: Option<String>,

    /// Workspace directory, relative to the repository's parent (default: ../<name>)
    #[arg(long, allow_hyphen_values = true)]
    dest: Option<String>,
}

impl ImportCommand {
    pub(super) fn run(&self) -> JayResult<String> {
        let repo = Repo::open(Path::new(&self.repo))?;
        let sync = repo.sync_token();
        let preview = repo.pull_request_import_preview(&self.url, &sync)?;
        let source = &preview.pull_request;
        let label = format!("{}#{}", source.base_repo, source.number);
        if let Some(existing) = preview.existing_workspace {
            return Ok(format!(
                "{label} is already in workspace {}: {}\n",
                existing.name, existing.dest
            ));
        }
        let name = self.name.as_deref().unwrap_or(&preview.workspace.name);
        let dest = self
            .dest
            .as_deref()
            .or(self.name.as_deref())
            .unwrap_or(&preview.workspace.dest);
        let dest =
            repo.pull_request_import(&self.url, &preview.head_commit_id, name, dest, &sync)?;
        Ok(format!("Imported {label} into workspace {name}: {dest}\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::args;
    use crate::cli::parser::{ParseOutcome, parse};

    fn parse_import(arguments: &[&str]) -> Result<ImportCommand, String> {
        parse("jayjay import", &args(arguments)).map(|outcome| match outcome {
            ParseOutcome::Run(command) => command,
            ParseOutcome::Print(text) => panic!("unexpected print: {text}"),
        })
    }

    #[test]
    fn takes_a_url_with_optional_repo_name_and_destination() {
        assert_eq!(
            parse_import(&["https://github.com/o/r/pull/1"]),
            Ok(ImportCommand {
                url: "https://github.com/o/r/pull/1".to_owned(),
                repo: ".".to_owned(),
                name: None,
                dest: None,
            })
        );
        assert_eq!(
            parse_import(&["u", "--repo", "/tmp/r", "--name", "pr", "--dest", "../pr"]),
            Ok(ImportCommand {
                url: "u".to_owned(),
                repo: "/tmp/r".to_owned(),
                name: Some("pr".to_owned()),
                dest: Some("../pr".to_owned()),
            })
        );
        assert!(
            parse_import(&[])
                .expect_err("missing URL")
                .contains("<URL>")
        );
    }
}
