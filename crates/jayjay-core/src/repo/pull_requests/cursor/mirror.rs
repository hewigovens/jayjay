use serde::Deserialize;

use super::super::super::Repo;
use super::super::super::environment::origin_binary;
use crate::repo::hosted_repo::{HostedRepo, RepoHost};

/// Origin rejects pull requests on inbound GitHub mirrors (`mirror.status == "inbound"`).
pub(super) const GITHUB_MIRROR_BLOCK: &str =
    "Origin pull requests aren't available for GitHub-mirrored repositories.";

/// Load Origin repository metadata and reject inbound GitHub mirrors, which cannot host Origin pull requests.
pub(crate) fn pr_creation_info(repo: &Repo) -> Result<OriginRepoInfo, String> {
    let remote = HostedRepo::parse(&repo.git_remote_url().map_err(|error| error.to_string())?)
        .ok_or_else(|| "Origin repository remote is unavailable".to_owned())?;
    if remote.host != RepoHost::Cursor {
        return Err("Origin repository remote is unavailable".to_owned());
    }
    let path = format!("/repos/{}/{}", remote.owner, remote.repo);
    let output = repo
        .command_output(&origin_binary(), &["api", path.as_str()], "origin api")
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        let detail = strip_origin_error(&Repo::stderr_text(&output));
        return Err(if detail.is_empty() {
            "origin api failed while checking repository mirror status".to_owned()
        } else {
            detail
        });
    }
    ensure_repo_can_create_pr(&Repo::stdout_text(&output))
}

fn ensure_repo_can_create_pr(json: &str) -> Result<OriginRepoInfo, String> {
    let response = serde_json::from_str::<OriginRepoInfo>(json)
        .map_err(|error| format!("origin api returned invalid repository JSON: {error}"))?;
    if response.is_inbound_github_mirror() {
        Err(GITHUB_MIRROR_BLOCK.to_owned())
    } else {
        Ok(response)
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct OriginRepoInfo {
    #[serde(default, rename = "defaultBranch")]
    default_branch: String,
    #[serde(default)]
    mirror: Option<OriginMirror>,
}

impl OriginRepoInfo {
    pub(crate) fn default_branch(&self) -> Result<&str, String> {
        let default_branch = self.default_branch.trim();
        if default_branch.is_empty() {
            Err("origin api did not return the repository's default branch".to_owned())
        } else {
            Ok(default_branch)
        }
    }

    fn is_inbound_github_mirror(&self) -> bool {
        self.mirror
            .as_ref()
            .is_some_and(|mirror| mirror.status.eq_ignore_ascii_case("inbound"))
    }
}

#[derive(Debug, Deserialize)]
struct OriginMirror {
    status: String,
}

pub(super) fn create_failure_message(text: &str) -> String {
    if looks_like_github_mirror_error(text) {
        GITHUB_MIRROR_BLOCK.to_owned()
    } else {
        strip_origin_error(text)
    }
}

fn looks_like_github_mirror_error(text: &str) -> bool {
    let text = text.to_ascii_lowercase();
    text.contains("github-mirrored") || text.contains("mirrorstatus=\"inbound\"")
}

fn strip_origin_error(text: &str) -> String {
    text.trim()
        .strip_prefix("Error:")
        .map(str::trim)
        .filter(|stripped| !stripped.is_empty())
        .unwrap_or(text.trim())
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inbound_github_mirror_json_is_blocked() {
        let inbound = r#"{"defaultBranch":"main","name":"jayjay","mirror":{"source":"github","sourceId":"R_kgDORrSFGA","status":"inbound"}}"#;
        assert_eq!(
            ensure_repo_can_create_pr(inbound).unwrap_err(),
            GITHUB_MIRROR_BLOCK
        );
        assert_eq!(
            ensure_repo_can_create_pr(r#"{"mirror":{"status":"INBOUND"}}"#).unwrap_err(),
            GITHUB_MIRROR_BLOCK
        );
    }

    #[test]
    fn standalone_origin_json_is_not_blocked() {
        let standalone = r#"{"id":"r_01","name":"jayjay-origin-smoke","fullName":"hewig/jayjay-origin-smoke","defaultBranch":"develop"}"#;
        assert_eq!(
            ensure_repo_can_create_pr(standalone)
                .unwrap()
                .default_branch()
                .unwrap(),
            "develop"
        );
        assert!(ensure_repo_can_create_pr(r#"{"mirror":null}"#).is_ok());
        assert!(ensure_repo_can_create_pr("{").is_err());
        assert!(ensure_repo_can_create_pr(r#"{"mirror":{}}"#).is_err());
    }

    #[test]
    fn create_failure_rewrites_mirror_cli_error() {
        let stderr = "Error: Cannot create a pull request in hewig/jayjay: Origin pull requests are not available for GitHub-mirrored repos; got mirrorStatus=\"inbound\".";
        assert_eq!(create_failure_message(stderr), GITHUB_MIRROR_BLOCK);
        assert_eq!(
            create_failure_message("Error: ref \"does-not-exist\" does not exist in git-forge"),
            "ref \"does-not-exist\" does not exist in git-forge"
        );
    }
}
