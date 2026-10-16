use serde::{Deserialize, Serialize};

use crate::repo::hosted_repo::HostedRepo;
use crate::types::{ChecksStatus, PrInfo, PrState};

#[derive(Serialize)]
pub(crate) struct CreatePull {
    pub head: String,
    pub base: String,
    pub title: String,
    pub body: String,
}

#[derive(Serialize)]
pub(crate) struct EditPull {
    pub base: String,
    pub title: String,
    pub body: String,
}

#[derive(Deserialize)]
pub(crate) struct CodebergPrResponse {
    number: u32,
    state: CodebergPrState,
    #[serde(default)]
    title: String,
    #[serde(default)]
    html_url: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    merged: bool,
    #[serde(default)]
    body: String,
    head: Option<CodebergPrBranch>,
}

#[derive(Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum CodebergPrState {
    Open,
    Closed,
}

impl From<CodebergPrState> for PrState {
    fn from(state: CodebergPrState) -> Self {
        match state {
            CodebergPrState::Open => PrState::Open,
            CodebergPrState::Closed => PrState::Closed,
        }
    }
}

#[derive(Deserialize)]
struct CodebergPrBranch {
    #[serde(rename = "ref", default)]
    name: String,
    #[serde(default)]
    sha: String,
    #[serde(default)]
    repo: Option<CodebergHeadRepo>,
}

#[derive(Deserialize)]
struct CodebergHeadRepo {
    #[serde(default)]
    full_name: String,
}

impl CodebergPrResponse {
    pub(super) fn matches(&self, head: &str) -> bool {
        self.head.as_ref().is_some_and(|branch| branch.name == head)
    }

    // Editing a fork's same-named branch PR, or a closed or merged one, would hijack someone else's review.
    pub(crate) fn is_stack_layer_of(&self, remote: &HostedRepo, head: &str) -> bool {
        self.state == CodebergPrState::Open
            && !self.merged
            && self.head.as_ref().is_some_and(|branch| branch.name == head)
            && self.head_repo_matches(remote)
    }

    fn head_repo_matches(&self, remote: &HostedRepo) -> bool {
        self.head
            .as_ref()
            .and_then(|branch| branch.repo.as_ref())
            .is_some_and(|repo| repo.full_name.eq_ignore_ascii_case(&remote.slug()))
    }

    pub(crate) fn number(&self) -> u32 {
        self.number
    }

    pub(crate) fn body(&self) -> &str {
        &self.body
    }

    pub(super) fn head_sha(&self) -> Option<&str> {
        self.head
            .as_ref()
            .map(|branch| branch.sha.as_str())
            .filter(|sha| !sha.is_empty())
    }

    pub(crate) fn web_url(&self) -> String {
        if self.html_url.is_empty() {
            self.url.clone()
        } else {
            self.html_url.clone()
        }
    }

    pub(super) fn into_pr_info(self, checks: ChecksStatus) -> PrInfo {
        let url = self.web_url();
        let state = if self.merged {
            PrState::Merged
        } else {
            self.state.into()
        };
        PrInfo {
            number: self.number,
            state,
            title: self.title,
            url,
            checks,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_an_open_same_repo_pr_on_the_head_branch_is_a_stack_layer() {
        let prs: Vec<CodebergPrResponse> = serde_json::from_str(
            r#"[
            {"number": 2, "state": "open", "head": {"ref": "feat", "repo": {"full_name": "fork/repo"}}},
            {"number": 3, "state": "closed", "head": {"ref": "feat", "repo": {"full_name": "owner/repo"}}},
            {"number": 4, "state": "closed", "merged": true, "head": {"ref": "feat", "repo": {"full_name": "owner/repo"}}},
            {"number": 5, "state": "open", "head": {"ref": "other", "repo": {"full_name": "owner/repo"}}},
            {"number": 6, "state": "open", "head": {"ref": "feat", "repo": {"full_name": "Owner/Repo"}}},
            {"number": 7, "state": "open", "head": {"ref": "feat"}}
        ]"#,
        )
        .unwrap();
        let remote = HostedRepo::parse("https://codeberg.org/owner/repo").unwrap();

        let layers: Vec<u32> = prs
            .iter()
            .filter(|pr| pr.is_stack_layer_of(&remote, "feat"))
            .map(CodebergPrResponse::number)
            .collect();
        assert_eq!(layers, [6]);
    }
}
