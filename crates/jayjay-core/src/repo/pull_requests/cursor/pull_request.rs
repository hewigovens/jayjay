use serde::{Deserialize, Serialize};

use crate::types::{ChecksStatus, PrInfo, PrState};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OriginPull {
    /// Origin encodes 64-bit integers as JSON strings.
    #[serde(deserialize_with = "number_string")]
    pub(crate) number: u32,
    state: String,
    #[serde(default)]
    merged: bool,
    title: String,
    pub(crate) web_url: String,
    #[serde(default)]
    head: Option<OriginRef>,
}

#[derive(Deserialize)]
struct OriginRef {
    #[serde(default)]
    sha: String,
}

impl OriginPull {
    /// Unknown states keep the lookup uncertain so they can never trigger PR creation.
    pub(super) fn pr_state(&self) -> Result<PrState, String> {
        match (self.state.as_str(), self.merged) {
            ("open", _) => Ok(PrState::Open),
            ("closed", true) => Ok(PrState::Merged),
            ("closed", false) => Ok(PrState::Closed),
            (state, _) => Err(format!(
                "Cursor returned unsupported pull request state {state:?}"
            )),
        }
    }

    pub(super) fn head_sha(&self) -> Option<&str> {
        self.head
            .as_ref()
            .map(|head| head.sha.as_str())
            .filter(|sha| !sha.is_empty())
    }

    pub(super) fn into_pr_info(self, state: PrState, checks: ChecksStatus) -> PrInfo {
        PrInfo {
            number: self.number,
            state,
            title: self.title,
            url: self.web_url,
            checks,
        }
    }
}

fn number_string<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<u32, D::Error> {
    String::deserialize(deserializer)?
        .parse()
        .map_err(|_| serde::de::Error::custom("invalid pull request number"))
}

#[derive(Serialize)]
pub(super) struct CreatePull<'a> {
    pub(super) title: &'a str,
    pub(super) body: &'a str,
    pub(super) head: &'a str,
    pub(super) base: &'a str,
}

#[derive(Serialize)]
pub(crate) struct UpdatePull<'a> {
    pub(crate) title: &'a str,
    pub(crate) body: &'a str,
    pub(crate) base: &'a str,
}
