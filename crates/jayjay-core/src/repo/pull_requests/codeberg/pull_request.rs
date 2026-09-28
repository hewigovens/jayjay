use serde::Deserialize;

use crate::types::{ChecksStatus, PrInfo, PrState};

#[derive(Deserialize)]
pub(super) struct CodebergPrResponse {
    number: u32,
    state: CodebergPrState,
    title: String,
    #[serde(default)]
    html_url: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    merged: bool,
    head: Option<CodebergPrBranch>,
}

/// Forgejo returns the state lowercase; a merged PR is "closed" + `merged: true`,
/// so there's no "merged" wire value here (into_pr_info handles that).
#[derive(Deserialize)]
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
}

impl CodebergPrResponse {
    pub(super) fn matches(&self, head: &str) -> bool {
        self.head.as_ref().is_some_and(|branch| branch.name == head)
    }

    pub(super) fn head_sha(&self) -> Option<&str> {
        self.head
            .as_ref()
            .map(|branch| branch.sha.as_str())
            .filter(|sha| !sha.is_empty())
    }

    pub(super) fn into_pr_info(self, checks: ChecksStatus) -> PrInfo {
        // A merged PR reports state "closed" + merged=true; surface it as Merged.
        let state = if self.merged {
            PrState::Merged
        } else {
            self.state.into()
        };
        let url = if self.html_url.is_empty() {
            self.url
        } else {
            self.html_url
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
