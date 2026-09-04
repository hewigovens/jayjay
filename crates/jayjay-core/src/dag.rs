//! DAG lane-assignment for the jj log graph.
//!
//! Mirrors `shell/mac/Sources/JayJay/Repo/DAGLayout.swift`. Pure logic — both
//! the SwiftUI shell (via uniffi) and the GPUI shell can render against the
//! same lane assignments.

use std::collections::HashMap;

use crate::types::{EdgeType, GraphEntry};

const COMPACT_LANE_THRESHOLD: usize = 4;
const COMPACT_VISIBLE_LANES: usize = 4;

/// Pre-computed lane assignment for a sequence of `GraphEntry` rows.
#[derive(Debug, Clone, Default)]
pub struct DagLayout {
    /// Lane index for each commit_id (hex).
    pub lanes: HashMap<String, usize>,
    /// Total active lanes at each row index.
    pub active_lanes_per_row: Vec<usize>,
    /// Lane indices active below each row.
    pub active_lane_indices_per_row: Vec<Vec<usize>>,
    /// Lane indices that were active both above and below each row.
    pub pass_through_lane_indices_per_row: Vec<Vec<usize>>,
    /// True when this row references lanes collapsed into the compact overflow lane.
    pub overflow_rows: Vec<bool>,
}

impl DagLayout {
    pub fn compute(entries: &[GraphEntry]) -> Self {
        let mut lanes: HashMap<String, usize> = HashMap::new();
        let mut active: Vec<Option<String>> = Vec::new();
        let mut active_counts: Vec<usize> = Vec::with_capacity(entries.len());
        let mut active_indices: Vec<Vec<usize>> = Vec::with_capacity(entries.len());
        let mut pass_through_indices: Vec<Vec<usize>> = Vec::with_capacity(entries.len());

        for entry in entries {
            let cid = &entry.change.commit_id.id;

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
            pass_through_indices.push(
                active
                    .iter()
                    .enumerate()
                    .filter_map(|(lane, commit)| {
                        (lane != my_lane && commit.is_some()).then_some(lane)
                    })
                    .collect(),
            );
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

        let mut layout = DagLayout {
            lanes,
            active_lanes_per_row: active_counts,
            active_lane_indices_per_row: active_indices,
            pass_through_lane_indices_per_row: pass_through_indices,
            overflow_rows: Vec::new(),
        };
        layout.overflow_rows = compute_overflow_rows(entries, &layout);
        layout
    }

    pub fn lane(&self, commit_id: &str) -> usize {
        self.lanes.get(commit_id).copied().unwrap_or(0)
    }

    fn max_lanes(&self) -> usize {
        self.active_lanes_per_row.iter().copied().max().unwrap_or(1)
    }

    pub fn display_lane_count(&self) -> usize {
        let max_lanes = self.max_lanes();
        if self.uses_compact_lanes() {
            COMPACT_VISIBLE_LANES
        } else {
            max_lanes
        }
    }

    fn uses_compact_lanes(&self) -> bool {
        self.max_lanes() > COMPACT_LANE_THRESHOLD
    }

    pub fn display_lane(&self, lane: usize) -> usize {
        if self.uses_compact_lanes() {
            lane.min(COMPACT_VISIBLE_LANES.saturating_sub(1))
        } else {
            lane
        }
    }

    pub fn active_lane_indices(&self, row: usize) -> &[usize] {
        self.active_lane_indices_per_row
            .get(row)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn pass_through_lane_indices(&self, row: usize) -> &[usize] {
        self.pass_through_lane_indices_per_row
            .get(row)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn row_has_overflow(&self, row: usize) -> bool {
        self.overflow_rows.get(row).copied().unwrap_or(false)
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

fn compute_overflow_rows(entries: &[GraphEntry], layout: &DagLayout) -> Vec<bool> {
    if !layout.uses_compact_lanes() {
        return vec![false; entries.len()];
    }

    entries
        .iter()
        .enumerate()
        .map(|(row, entry)| row_has_hidden_lanes(entry, row, layout))
        .collect()
}

fn row_has_hidden_lanes(entry: &GraphEntry, row: usize, layout: &DagLayout) -> bool {
    let row_lane = layout.lane(&entry.change.commit_id.id);
    if lane_is_compacted(row_lane) {
        return true;
    }

    if layout
        .active_lane_indices(row)
        .iter()
        .copied()
        .any(lane_is_compacted)
    {
        return true;
    }

    if row > 0
        && layout
            .active_lane_indices(row - 1)
            .iter()
            .copied()
            .any(lane_is_compacted)
    {
        return true;
    }

    layout
        .active_lane_indices(row + 1)
        .iter()
        .copied()
        .any(lane_is_compacted)
        || entry
            .edges
            .iter()
            .filter(|edge| edge.edge_type != EdgeType::Missing)
            .map(|edge| layout.lane(&edge.target))
            .any(lane_is_compacted)
}

fn lane_is_compacted(lane: usize) -> bool {
    lane >= COMPACT_VISIBLE_LANES
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ChangeInfo, CommitAuthor, GraphEdge, ShortId};

    fn entry(commit_id: &str, parents: &[&str]) -> GraphEntry {
        GraphEntry {
            change: ChangeInfo {
                change_id: ShortId::new(format!("change-{commit_id}"), 1),
                commit_id: ShortId::new(commit_id.to_owned(), 1),
                description: String::new(),
                author: CommitAuthor::empty(0),
                parents: parents.iter().map(|s| (*s).to_owned()).collect(),
                bookmarks: Vec::new(),
                tags: Vec::new(),
                workspaces: Vec::new(),
                is_working_copy: false,
                has_conflict: false,
                is_empty: false,
                is_immutable: false,
                is_divergent: false,
                new_change: crate::types::NewChangeEligibility {
                    on_top: true,
                    before: true,
                    after: true,
                },
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
        assert_eq!(layout.display_lane_count(), 1);
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
        assert_eq!(layout.display_lane_count(), 1);
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
        assert!(layout.pass_through_lane_indices(0).is_empty());
        assert_eq!(layout.pass_through_lane_indices(1), &[1]);
        assert!(layout.max_lanes() >= 2);
        assert_eq!(layout.display_lane_count(), layout.max_lanes());
    }

    #[test]
    fn four_lane_graph_uses_dynamic_width_and_zero_offsets() {
        let entries = vec![
            entry("merge", &["p0", "p1", "p2", "p3"]),
            entry("p3", &["base"]),
            entry("p2", &["base"]),
            entry("p1", &["base"]),
        ];
        let layout = DagLayout::compute(&entries);

        assert_eq!(layout.max_lanes(), 4);
        assert_eq!(layout.display_lane_count(), 4);
        assert!(!layout.uses_compact_lanes());
        assert_eq!(layout.lane("p3"), 3);
        assert_eq!(layout.display_lane(layout.lane("p3")), 3);
        assert!(!layout.row_has_overflow(0));
        assert!(!layout.row_has_overflow(1));
        assert!(!layout.row_has_overflow(2));
    }

    #[test]
    fn compact_display_lanes_collapse_hidden_lanes_into_stable_overflow_slot() {
        let entries = vec![
            entry("merge", &["p0", "p1", "p2", "p3", "p4", "p5"]),
            entry("p5", &["base"]),
            entry("p4", &["base"]),
            entry("p3", &["base"]),
        ];
        let layout = DagLayout::compute(&entries);

        assert_eq!(layout.max_lanes(), 6);
        assert_eq!(layout.display_lane_count(), COMPACT_VISIBLE_LANES);
        assert!(layout.uses_compact_lanes());
        assert_eq!(layout.display_lane(layout.lane("p0")), 0);
        assert_eq!(layout.display_lane(layout.lane("p3")), 3);
        assert_eq!(layout.lane("p5"), 5);
        assert_eq!(layout.display_lane(layout.lane("p5")), 3);
        assert_eq!(layout.lane("p4"), 4);
        assert_eq!(layout.display_lane(layout.lane("p4")), 3);
        assert!(layout.row_has_overflow(0));
        assert!(layout.row_has_overflow(1));
        assert!(layout.row_has_overflow(2));
    }

    #[test]
    fn compact_overflow_row_tracks_hidden_active_lanes() {
        let entries = vec![
            entry("merge", &["p0", "p1", "p2", "p3", "p4", "p5"]),
            entry("p0", &["base"]),
        ];
        let layout = DagLayout::compute(&entries);

        assert_eq!(layout.max_lanes(), 6);
        assert_eq!(layout.display_lane_count(), COMPACT_VISIBLE_LANES);
        assert_eq!(layout.lane("p0"), 0);
        assert_eq!(layout.display_lane(layout.lane("p0")), 0);
        assert!(layout.row_has_overflow(1));
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
