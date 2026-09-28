use std::collections::HashMap;

use crate::types::{Overview, OverviewBase, OverviewBaseKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverviewGroup {
    pub base: OverviewBase,
    pub lanes: Vec<u32>,
}

pub fn overview_groups(overview: &Overview) -> Vec<OverviewGroup> {
    let mut groups: Vec<OverviewGroup> = Vec::new();
    let mut group_by_commit: HashMap<&str, usize> = HashMap::new();
    for (index, lane) in overview.lanes.iter().enumerate() {
        let slot = *group_by_commit
            .entry(lane.base.commit_id.id.as_str())
            .or_insert_with(|| {
                groups.push(OverviewGroup {
                    base: lane.base.clone(),
                    lanes: Vec::new(),
                });
                groups.len() - 1
            });
        groups[slot].lanes.push(index as u32);
    }
    groups.sort_by(|a, b| {
        base_rank(a.base.kind)
            .cmp(&base_rank(b.base.kind))
            .then(a.base.behind_trunk.cmp(&b.base.behind_trunk))
            .then(b.base.timestamp_millis.cmp(&a.base.timestamp_millis))
            .then(a.base.commit_id.id.cmp(&b.base.commit_id.id))
    });
    for group in &mut groups {
        group.lanes.sort_by(|&a, &b| {
            let (la, lb) = (&overview.lanes[a as usize], &overview.lanes[b as usize]);
            lb.has_current_workspace()
                .cmp(&la.has_current_workspace())
                .then(lb.latest_timestamp_millis.cmp(&la.latest_timestamp_millis))
                .then(a.cmp(&b))
        });
    }
    groups
}

fn base_rank(kind: OverviewBaseKind) -> u8 {
    match kind {
        OverviewBaseKind::Trunk => 0,
        OverviewBaseKind::Mutable => 1,
        OverviewBaseKind::OlderTrunk => 2,
        OverviewBaseKind::Other => 3,
    }
}
