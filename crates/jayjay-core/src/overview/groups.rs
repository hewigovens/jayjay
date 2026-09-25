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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{OverviewChange, OverviewLane, OverviewWorkspace, ShortId};

    fn base(id: &str, kind: OverviewBaseKind, timestamp: i64) -> OverviewBase {
        OverviewBase {
            change_id: ShortId::new(id.to_owned(), 2),
            commit_id: ShortId::new(id.to_owned(), 2),
            description: String::new(),
            timestamp_millis: timestamp,
            kind,
            bookmarks: Vec::new(),
            behind_trunk: 0,
        }
    }

    fn lane(base: OverviewBase, latest: i64, current_workspace: bool) -> OverviewLane {
        OverviewLane {
            changes: vec![OverviewChange {
                change_id: ShortId::new("c".to_owned(), 1),
                commit_id: ShortId::new("c".to_owned(), 1),
                description: String::new(),
                full_description: String::new(),
                timestamp_millis: latest,
                is_empty: false,
                has_conflict: false,
                bookmarks: Vec::new(),
                workspaces: Vec::new(),
            }],
            base,
            workspaces: if current_workspace {
                vec![OverviewWorkspace {
                    name: "default".to_owned(),
                    is_current: true,
                    changes_above: 0,
                }]
            } else {
                Vec::new()
            },
            latest_timestamp_millis: latest,
            attention: Vec::new(),
        }
    }

    #[test]
    fn trunk_group_comes_first_with_the_current_workspace_leading_it() {
        let trunk = base("main", OverviewBaseKind::Trunk, 300);
        let mut older = base("old", OverviewBaseKind::OlderTrunk, 100);
        older.behind_trunk = 1;
        let mut oldest = base("oldest", OverviewBaseKind::OlderTrunk, 200);
        oldest.behind_trunk = 2;
        let overview = Overview {
            lanes: vec![
                lane(older, 5, false),
                lane(trunk.clone(), 10, false),
                lane(trunk.clone(), 20, true),
                lane(trunk, 30, false),
                lane(oldest, 40, false),
            ],
            workspace_count: 1,
        };

        let groups = overview_groups(&overview);

        assert_eq!(groups.len(), 3);
        assert_eq!(groups[0].base.commit_id.id, "main");
        assert_eq!(
            groups[0].lanes,
            [2, 3, 1],
            "current workspace, then by activity"
        );
        assert_eq!(groups[1].lanes, [0]);
        assert_eq!(
            groups[2].base.commit_id.id, "oldest",
            "older trunk bases follow ancestry, not committer time"
        );
    }
}
