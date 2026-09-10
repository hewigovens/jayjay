#[derive(uniffi::Record, Debug, Clone)]
pub struct DagLayoutData {
    lanes: std::collections::HashMap<String, u32>,
    active_lanes_per_row: Vec<u32>,
    active_lane_indices_per_row: Vec<Vec<u32>>,
    pass_through_lane_indices_per_row: Vec<Vec<u32>>,
    missing_ancestry_rows: Vec<bool>,
    overflow_rows: Vec<bool>,
    display_lane_count: u32,
}

#[uniffi::export]
fn compute_dag_layout(entries: Vec<jayjay_core::GraphEntry>) -> DagLayoutData {
    let layout = jayjay_core::dag::DagLayout::compute(&entries);
    let display_lane_count = layout.display_lane_count();
    DagLayoutData {
        lanes: layout
            .lanes
            .into_iter()
            .map(|(k, v)| (k, v as u32))
            .collect(),
        active_lanes_per_row: layout
            .active_lanes_per_row
            .iter()
            .map(|&v| v as u32)
            .collect(),
        active_lane_indices_per_row: layout
            .active_lane_indices_per_row
            .iter()
            .map(|row| row.iter().map(|&v| v as u32).collect())
            .collect(),
        pass_through_lane_indices_per_row: layout
            .pass_through_lane_indices_per_row
            .iter()
            .map(|row| row.iter().map(|&v| v as u32).collect())
            .collect(),
        missing_ancestry_rows: layout.missing_ancestry_rows,
        overflow_rows: layout.overflow_rows.to_vec(),
        display_lane_count: display_lane_count as u32,
    }
}

#[uniffi::export]
fn descendant_commit_ids(entries: Vec<jayjay_core::GraphEntry>, commit_id: &str) -> Vec<String> {
    jayjay_core::dag::descendant_commit_ids(&entries, commit_id)
}

#[uniffi::export]
fn can_rebase_onto(
    entries: Vec<jayjay_core::GraphEntry>,
    source_commit_id: &str,
    target_commit_id: &str,
) -> bool {
    jayjay_core::dag::can_rebase_onto(&entries, source_commit_id, target_commit_id)
}
