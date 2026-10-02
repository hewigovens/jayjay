use serde::Deserialize;

use crate::types::{ChecksStatus, PrInfo, PrState};

#[derive(Deserialize)]
pub(crate) struct GitLabMrResponse {
    /// Per-project number shown in the UI and used in MR URLs (not the global `id`).
    iid: u32,
    state: GitLabMrState,
    title: String,
    #[serde(default)]
    web_url: String,
    #[serde(default)]
    source_branch: String,
    /// Diff head sha; used to look up the pipeline status.
    #[serde(default)]
    sha: String,
    source_project_id: Option<u64>,
    target_project_id: Option<u64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum GitLabMrState {
    Opened,
    Closed,
    Merged,
    Locked,
}

impl From<GitLabMrState> for PrState {
    fn from(state: GitLabMrState) -> Self {
        match state {
            GitLabMrState::Opened | GitLabMrState::Locked => PrState::Open,
            GitLabMrState::Closed => PrState::Closed,
            GitLabMrState::Merged => PrState::Merged,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum MrHeadProject {
    Same,
    Fork(u64),
}

impl GitLabMrResponse {
    pub(super) fn matches(&self, source_branch: &str) -> bool {
        self.source_branch == source_branch
    }

    pub(super) fn is_open(&self) -> bool {
        matches!(self.state, GitLabMrState::Opened)
    }

    pub(crate) fn head_sha(&self) -> Option<&str> {
        (!self.sha.is_empty()).then_some(self.sha.as_str())
    }

    pub(crate) fn head_branch(&self) -> &str {
        &self.source_branch
    }

    pub(crate) fn head_project(&self) -> Option<MrHeadProject> {
        match (self.source_project_id, self.target_project_id) {
            (Some(source), Some(target)) => Some(if source == target {
                MrHeadProject::Same
            } else {
                MrHeadProject::Fork(source)
            }),
            _ => None,
        }
    }

    pub(crate) fn into_pr_info(self, checks: ChecksStatus) -> PrInfo {
        PrInfo {
            number: self.iid,
            state: self.state.into(),
            title: self.title,
            url: self.web_url,
            checks,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_open_mr_with_iid_and_web_url() {
        let json = r#"[
            {
                "iid": 7,
                "id": 9001,
                "state": "opened",
                "title": "feat: add gitlab support",
                "web_url": "https://gitlab.com/hewig/jj-test/-/merge_requests/7",
                "source_branch": "feat/gitlab",
                "sha": "head-sha"
            }
        ]"#;

        let mr = serde_json::from_str::<Vec<GitLabMrResponse>>(json)
            .unwrap()
            .into_iter()
            .find(|mr| mr.matches("feat/gitlab"))
            .unwrap();
        assert!(mr.is_open());
        assert_eq!(mr.head_sha(), Some("head-sha"));

        let pr = mr.into_pr_info(ChecksStatus::None);
        assert_eq!(pr.number, 7);
        assert_eq!(pr.state, PrState::Open);
        assert_eq!(pr.title, "feat: add gitlab support");
        assert_eq!(
            pr.url,
            "https://gitlab.com/hewig/jj-test/-/merge_requests/7"
        );
    }

    #[test]
    fn merged_state_maps_to_merged() {
        let json = r#"{
            "iid": 1, "state": "merged", "title": "t",
            "web_url": "u", "source_branch": "b", "sha": ""
        }"#;
        let mr: GitLabMrResponse = serde_json::from_str(json).unwrap();
        assert!(!mr.is_open());
        assert_eq!(mr.head_sha(), None);
        assert_eq!(mr.into_pr_info(ChecksStatus::None).state, PrState::Merged);
    }

    #[test]
    fn head_project_distinguishes_same_project_from_fork_and_missing_ids() {
        let mr = |source: &str, target: &str| {
            let json = format!(
                r#"{{"iid":1,"state":"opened","title":"t","web_url":"u",
                   "source_branch":"b","sha":"","source_project_id":{source},"target_project_id":{target}}}"#
            );
            serde_json::from_str::<GitLabMrResponse>(&json).unwrap()
        };
        assert_eq!(mr("7765", "7765").head_project(), Some(MrHeadProject::Same));
        assert_eq!(
            mr("83542242", "7765").head_project(),
            Some(MrHeadProject::Fork(83542242))
        );
        assert_eq!(mr("null", "7765").head_project(), None);
        assert_eq!(mr("null", "null").head_project(), None);
    }
}
