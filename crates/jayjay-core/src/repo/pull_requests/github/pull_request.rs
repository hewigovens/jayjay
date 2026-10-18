use serde::Deserialize;

use super::super::checks;
use super::status::GhCheckRun;
use crate::types::{ChecksStatus, PrInfo, PrState};

pub(crate) const PULL_REQUEST_FIELDS: &str = "
fragment PullRequestFields on PullRequest {
  number state title url headRefName headRefOid isCrossRepository
  headRepositoryOwner { login }
  headRepository { name }
  commits(last: 1) { nodes { commit { statusCheckRollup { contexts(first: 100) { nodes {
    ... on CheckRun { status conclusion }
    ... on StatusContext { state }
  } } } } } }
}";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GitHubPullRequest {
    pub(crate) number: u32,
    pub(crate) state: GitHubPrState,
    pub(crate) title: String,
    pub(crate) url: String,
    #[serde(default)]
    pub(crate) head_ref_name: String,
    #[serde(default)]
    pub(crate) head_ref_oid: String,
    #[serde(default)]
    pub(crate) is_cross_repository: bool,
    pub(crate) head_repository_owner: Option<GhRepositoryOwner>,
    pub(crate) head_repository: Option<GhRepository>,
    #[serde(default)]
    commits: Nodes<CommitNode>,
}

#[derive(Deserialize)]
pub(crate) struct GhRepositoryOwner {
    pub(crate) login: String,
}

#[derive(Deserialize)]
pub(crate) struct GhRepository {
    pub(crate) name: String,
}

#[derive(Deserialize)]
pub(crate) struct Nodes<T> {
    pub(crate) nodes: Vec<T>,
}

impl<T> Default for Nodes<T> {
    fn default() -> Self {
        Self { nodes: Vec::new() }
    }
}

#[derive(Deserialize)]
struct CommitNode {
    commit: Commit,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Commit {
    status_check_rollup: Option<Rollup>,
}

#[derive(Deserialize)]
struct Rollup {
    contexts: Nodes<GhCheckRun>,
}

#[derive(Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub(crate) enum GitHubPrState {
    Open,
    Closed,
    Merged,
}

impl From<GitHubPrState> for PrState {
    fn from(state: GitHubPrState) -> Self {
        match state {
            GitHubPrState::Open => PrState::Open,
            GitHubPrState::Closed => PrState::Closed,
            GitHubPrState::Merged => PrState::Merged,
        }
    }
}

impl GitHubPullRequest {
    pub(crate) fn is_open(&self) -> bool {
        self.state == GitHubPrState::Open
    }

    fn checks(&self) -> ChecksStatus {
        let contexts = self
            .commits
            .nodes
            .iter()
            .filter_map(|node| node.commit.status_check_rollup.as_ref())
            .flat_map(|rollup| &rollup.contexts.nodes);
        checks::rollup(contexts.map(GhCheckRun::state))
    }

    pub(crate) fn into_pr_info(self) -> PrInfo {
        PrInfo {
            checks: self.checks(),
            number: self.number,
            state: self.state.into(),
            title: self.title,
            url: self.url,
        }
    }
}

#[cfg(test)]
pub(crate) fn pull_request_json(number: u32, state: &str, contexts: &str) -> String {
    format!(
        r#"{{"number":{number},"state":"{state}","title":"CI","url":"https://github.com/o/r/pull/{number}","headRefName":"feat","headRefOid":"abc","isCrossRepository":false,"headRepositoryOwner":{{"login":"o"}},"headRepository":{{"name":"r"}},"commits":{{"nodes":[{{"commit":{{"statusCheckRollup":{{"contexts":{{"nodes":{contexts}}}}}}}}}]}}}}"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rolls_up_check_run_and_status_context_outcomes() {
        for (state, expected_state, contexts, expected) in [
            (
                "MERGED",
                PrState::Merged,
                r#"[{"status":"COMPLETED","conclusion":"SUCCESS"},{"status":"COMPLETED","conclusion":"FAILURE"}]"#,
                ChecksStatus::Failing,
            ),
            (
                "OPEN",
                PrState::Open,
                r#"[{"status":"COMPLETED","conclusion":"SUCCESS"},{"status":"IN_PROGRESS","conclusion":null}]"#,
                ChecksStatus::Pending,
            ),
            (
                "OPEN",
                PrState::Open,
                r#"[{"status":"COMPLETED","conclusion":"SUCCESS"},{"state":"FAILURE"}]"#,
                ChecksStatus::Failing,
            ),
            (
                "OPEN",
                PrState::Open,
                r#"[{"state":"SUCCESS"}]"#,
                ChecksStatus::Passing,
            ),
            ("OPEN", PrState::Open, "[]", ChecksStatus::None),
        ] {
            let json = pull_request_json(42, state, contexts);
            let pr = serde_json::from_str::<GitHubPullRequest>(&json)
                .unwrap()
                .into_pr_info();
            assert_eq!(pr.state, expected_state, "{state}");
            assert_eq!(pr.checks, expected, "{contexts}");
        }
    }

    #[test]
    fn a_commit_without_a_rollup_has_no_checks() {
        let json = pull_request_json(1, "OPEN", "[]").replace(
            r#"{"statusCheckRollup":{"contexts":{"nodes":[]}}}"#,
            r#"{"statusCheckRollup":null}"#,
        );
        let pr: GitHubPullRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(pr.into_pr_info().checks, ChecksStatus::None);
    }
}
