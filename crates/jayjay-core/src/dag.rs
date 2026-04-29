//! DAG lane-assignment for the jj log graph.
//!
//! Mirrors `shell/mac/Sources/JayJay/Repo/DAGLayout.swift`. Pure logic — both
//! the SwiftUI shell (via uniffi) and the GPUI shell can render against the
//! same lane assignments.

use std::collections::HashMap;

use crate::types::{EdgeType, GraphEntry};

pub const LANE_WIDTH: f32 = 16.0;
pub const NODE_RADIUS: f32 = 4.0;
pub const ROW_LEADING_PADDING: f32 = 4.0;
pub const ROW_VERTICAL_PADDING: f32 = 8.0;
pub const NODE_CENTER_Y: f32 = 12.0;

/// Pre-computed lane assignment for a sequence of `GraphEntry` rows.
#[derive(Debug, Clone, Default)]
pub struct DagLayout {
    /// Lane index for each commit_id (hex).
    pub lanes: HashMap<String, usize>,
    /// Total active lanes at each row index.
    pub active_lanes_per_row: Vec<usize>,
    /// Lane indices that continue through each row.
    pub active_lane_indices_per_row: Vec<Vec<usize>>,
}

impl DagLayout {
    pub fn compute(entries: &[GraphEntry]) -> Self {
        let mut lanes: HashMap<String, usize> = HashMap::new();
        let mut active: Vec<Option<String>> = Vec::new();
        let mut active_counts: Vec<usize> = Vec::with_capacity(entries.len());
        let mut active_indices: Vec<Vec<usize>> = Vec::with_capacity(entries.len());

        for entry in entries {
            let cid = &entry.change.commit_id;

            if !lanes.contains_key(cid) {
                let lane = match active
                    .iter()
                    .position(|c| c.as_deref() == Some(cid.as_str()))
                {
                    Some(existing) => existing,
                    None => assign_lane(cid, &mut active, None),
                };
                lanes.insert(cid.clone(), lane);
            }

            let my_lane = lanes[cid];
            if my_lane < active.len() {
                active[my_lane] = None;
            }

            for edge in &entry.edges {
                if edge.edge_type == EdgeType::Missing {
                    continue;
                }
                if !lanes.contains_key(&edge.target) {
                    let lane = assign_lane(&edge.target, &mut active, Some(my_lane));
                    lanes.insert(edge.target.clone(), lane);
                }
            }

            active_counts.push(active.len());
            let row_active: Vec<usize> = active
                .iter()
                .enumerate()
                .filter_map(|(i, c)| c.as_ref().map(|_| i))
                .collect();
            active_indices.push(row_active);
        }

        DagLayout {
            lanes,
            active_lanes_per_row: active_counts,
            active_lane_indices_per_row: active_indices,
        }
    }

    pub fn lane(&self, commit_id: &str) -> usize {
        self.lanes.get(commit_id).copied().unwrap_or(0)
    }

    pub fn max_lanes(&self) -> usize {
        self.active_lanes_per_row.iter().copied().max().unwrap_or(1)
    }

    pub fn active_lane_indices(&self, row: usize) -> &[usize] {
        self.active_lane_indices_per_row
            .get(row)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }
}

fn assign_lane(
    commit_id: &str,
    active: &mut Vec<Option<String>>,
    preferring: Option<usize>,
) -> usize {
    if let Some(preferred) = preferring
        && preferred < active.len()
        && active[preferred].is_none()
    {
        active[preferred] = Some(commit_id.to_owned());
        return preferred;
    }
    if let Some(free) = active.iter().position(|c| c.is_none()) {
        active[free] = Some(commit_id.to_owned());
        return free;
    }
    active.push(Some(commit_id.to_owned()));
    active.len() - 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ChangeInfo, GraphEdge};

    fn entry(commit_id: &str, parents: &[&str]) -> GraphEntry {
        GraphEntry {
            change: ChangeInfo {
                change_id: format!("change-{commit_id}"),
                commit_id: commit_id.to_owned(),
                description: String::new(),
                author: String::new(),
                email: String::new(),
                timestamp_millis: 0,
                parents: parents.iter().map(|s| (*s).to_owned()).collect(),
                bookmarks: Vec::new(),
                is_working_copy: false,
                has_conflict: false,
                is_empty: false,
                is_immutable: false,
                is_divergent: false,
            },
            edges: parents
                .iter()
                .map(|p| GraphEdge {
                    target: (*p).to_owned(),
                    edge_type: EdgeType::Direct,
                })
                .collect(),
        }
    }

    #[test]
    fn empty_entries() {
        let layout = DagLayout::compute(&[]);
        assert!(layout.lanes.is_empty());
        assert_eq!(layout.max_lanes(), 1);
    }

    #[test]
    fn linear_chain_on_lane_zero() {
        // C -> B -> A, all on lane 0
        let entries = vec![entry("C", &["B"]), entry("B", &["A"]), entry("A", &[])];
        let layout = DagLayout::compute(&entries);
        assert_eq!(layout.lane("C"), 0);
        assert_eq!(layout.lane("B"), 0);
        assert_eq!(layout.lane("A"), 0);
        assert_eq!(layout.max_lanes(), 1);
    }

    #[test]
    fn fork_uses_two_lanes() {
        // D has two parents B and C, both have parent A.
        // Reverse-topological order: D, B, C, A
        let entries = vec![
            entry("D", &["B", "C"]),
            entry("B", &["A"]),
            entry("C", &["A"]),
            entry("A", &[]),
        ];
        let layout = DagLayout::compute(&entries);
        assert_eq!(layout.lane("D"), 0);
        assert_eq!(layout.lane("B"), 0); // fork's first edge stays on parent's lane
        assert_eq!(layout.lane("C"), 1); // second edge spawns new lane
        assert_eq!(layout.lane("A"), 0); // merged back into B's lane
        assert!(layout.max_lanes() >= 2);
    }

    #[test]
    fn missing_edges_dont_assign_lanes() {
        let mut e = entry("A", &[]);
        e.edges.push(GraphEdge {
            target: "missing-parent".to_owned(),
            edge_type: EdgeType::Missing,
        });
        let layout = DagLayout::compute(&[e]);
        assert!(!layout.lanes.contains_key("missing-parent"));
        assert_eq!(layout.lane("A"), 0);
    }
}
