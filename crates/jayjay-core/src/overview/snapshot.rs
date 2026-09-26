use std::collections::HashMap;

use super::OverviewGroup;
use crate::types::{Overview, OverviewBaseKind, WorkspaceInfo};

#[derive(Debug, Clone)]
pub struct OverviewSnapshot {
    pub overview: Overview,
    pub groups: Vec<OverviewGroup>,
    pub workspaces: Vec<WorkspaceInfo>,
}

impl OverviewSnapshot {
    /// Divergent heads share a change id, so those lanes carry the commit id too.
    pub fn lane_ids(&self) -> Vec<String> {
        let mut seen: HashMap<&str, usize> = HashMap::new();
        for lane in &self.overview.lanes {
            *seen.entry(lane.head().change_id.id.as_str()).or_default() += 1;
        }
        self.overview
            .lanes
            .iter()
            .map(|lane| {
                let head = lane.head();
                match seen[head.change_id.id.as_str()] {
                    1 => head.change_id.id.clone(),
                    _ => format!("{}/{}", head.change_id.id, head.commit_id.id),
                }
            })
            .collect()
    }

    pub fn visible_groups(&self, filter: &str) -> Vec<OverviewGroup> {
        self.groups
            .iter()
            .filter_map(|group| {
                let lanes: Vec<u32> = group
                    .lanes
                    .iter()
                    .copied()
                    .filter(|&lane| self.overview.lanes[lane as usize].matches(filter))
                    .collect();
                (!lanes.is_empty()).then(|| OverviewGroup {
                    base: group.base.clone(),
                    lanes,
                })
            })
            .collect()
    }

    pub fn trunk_name(&self) -> &str {
        match self
            .groups
            .iter()
            .find(|group| group.base.kind == OverviewBaseKind::Trunk)
            .map(|group| group.base.bookmarks.as_slice())
        {
            Some([bookmark]) => bookmark,
            _ => "trunk",
        }
    }

    pub fn workspace(&self, name: &str) -> Option<&WorkspaceInfo> {
        self.workspaces
            .iter()
            .find(|workspace| workspace.name == name)
    }
}
