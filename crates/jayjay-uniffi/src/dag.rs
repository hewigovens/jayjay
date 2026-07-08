#[derive(uniffi::Record, Debug, Clone)]
pub struct DagLayoutData {
    pub lanes: std::collections::HashMap<String, u32>,
    pub active_lanes_per_row: Vec<u32>,
    pub active_lane_indices_per_row: Vec<Vec<u32>>,
    pub overflow_rows: Vec<bool>,
    pub display_lane_count: u32,
}

#[uniffi::export]
pub fn compute_dag_layout(entries: Vec<jayjay_core::GraphEntry>) -> DagLayoutData {
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
        overflow_rows: layout.overflow_rows.to_vec(),
        display_lane_count: display_lane_count as u32,
    }
}
