use jayjay_primitives::{ReviewGroupState, ReviewMarkSource};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct StoredReviewGroup {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) source: Option<ReviewMarkSource>,
    pub(crate) digest: String,
    #[serde(default)]
    pub(crate) state: ReviewGroupState,
}

impl StoredReviewGroup {
    pub(crate) fn agent_owned(&self) -> bool {
        self.source == Some(ReviewMarkSource::Agent)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum ReviewEntryState {
    File,
    Hunks {
        indices: Vec<u32>,
    },
    Groups {
        algorithm_version: u32,
        #[serde(default)]
        groups: Vec<StoredReviewGroup>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        removed_reviewed: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ReviewEntry {
    /// Caller-supplied content identity at mark time. Treated as opaque.
    pub(crate) identity: String,
    pub(crate) state: ReviewEntryState,
    #[serde(default, skip_serializing_if = "ReviewMarkSource::is_user")]
    pub(crate) source: ReviewMarkSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) agent_removed_reviewed: Option<Vec<String>>,
    /// Entry fields written by a newer binary; carried through saves so an older binary cannot strip them.
    #[serde(flatten)]
    pub(crate) extra: serde_json::Map<String, serde_json::Value>,
}

impl ReviewEntry {
    pub(crate) fn new(identity: &str, state: ReviewEntryState) -> Self {
        Self {
            identity: identity.to_string(),
            state,
            source: ReviewMarkSource::User,
            agent_removed_reviewed: None,
            extra: serde_json::Map::new(),
        }
    }

    pub(crate) fn agent_removed(&self) -> &[String] {
        self.agent_removed_reviewed.as_deref().unwrap_or(&[])
    }

    pub(crate) fn owned_by(&self, source: ReviewMarkSource) -> bool {
        let ReviewEntryState::Groups {
            groups,
            removed_reviewed,
            ..
        } = &self.state
        else {
            return self.source == source;
        };
        let agent = source == ReviewMarkSource::Agent;
        groups
            .iter()
            .any(|g| g.state != ReviewGroupState::Unreviewed && g.agent_owned() == agent)
            || if agent {
                !self.agent_removed().is_empty()
            } else {
                removed_reviewed.len() > self.agent_removed().len()
            }
    }

    pub(crate) fn with_group_sources_restored(mut self) -> Self {
        if self.source == ReviewMarkSource::Agent
            && let ReviewEntryState::Groups { groups, .. } = &mut self.state
            && groups.iter().all(|g| g.source.is_none())
        {
            for group in groups
                .iter_mut()
                .filter(|g| g.state != ReviewGroupState::Unreviewed)
            {
                group.source = Some(ReviewMarkSource::Agent);
            }
        }
        self
    }

    pub(crate) fn refresh_source(&mut self) {
        if !matches!(self.state, ReviewEntryState::Groups { .. }) {
            return;
        }
        let agent = self.owned_by(ReviewMarkSource::Agent);
        self.source = if agent {
            ReviewMarkSource::Agent
        } else {
            ReviewMarkSource::User
        };
        if let ReviewEntryState::Groups { groups, .. } = &mut self.state {
            for group in groups.iter_mut().filter(|g| !g.agent_owned()) {
                group.source = agent.then_some(ReviewMarkSource::User);
            }
        }
    }

    pub(crate) fn inherit_group_sources(&mut self, previous: Option<&Self>, touched: &[u32]) {
        let Some(previous) = previous else { return };
        let previous_removed = previous.agent_removed();
        let mut agent_removed = previous_removed.to_vec();
        if let ReviewEntryState::Groups { groups, .. } = &previous.state {
            agent_removed.extend(
                groups
                    .iter()
                    .filter(|g| g.state == ReviewGroupState::Reviewed && g.agent_owned())
                    .map(|g| g.digest.clone()),
            );
        }
        let ReviewEntryState::Groups {
            groups,
            removed_reviewed,
            ..
        } = &mut self.state
        else {
            return;
        };
        for (index, group) in groups.iter_mut().enumerate() {
            let agent = match &previous.state {
                ReviewEntryState::Groups { groups: old, .. }
                    if previous.identity != self.identity =>
                {
                    old.iter()
                        .any(|old| old.digest == group.digest && old.agent_owned())
                        || previous_removed.contains(&group.digest)
                }
                ReviewEntryState::Groups { groups: old, .. } => {
                    old.get(index).is_some_and(StoredReviewGroup::agent_owned)
                }
                _ => previous.source == ReviewMarkSource::Agent,
            };
            group.source =
                (agent && !touched.contains(&(index as u32))).then_some(ReviewMarkSource::Agent);
        }
        self.agent_removed_reviewed = Some(
            removed_reviewed
                .iter()
                .filter_map(|digest| {
                    let index = agent_removed
                        .iter()
                        .position(|candidate| candidate == digest)?;
                    Some(agent_removed.remove(index))
                })
                .collect(),
        );
        self.refresh_source();
    }

    pub(crate) fn file(identity: &str) -> Self {
        Self::new(identity, ReviewEntryState::File)
    }

    pub(crate) fn hunks(identity: &str, mut indices: Vec<u32>) -> Self {
        indices.sort_unstable();
        indices.dedup();
        Self::new(identity, ReviewEntryState::Hunks { indices })
    }

    pub(crate) fn with_extra(mut self, extra: serde_json::Map<String, serde_json::Value>) -> Self {
        self.extra = extra;
        self
    }
}

pub(crate) fn key(change_id: &str, path: &str) -> String {
    format!("{change_id}|{path}")
}
