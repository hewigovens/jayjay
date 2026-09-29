use jayjay_core::{DiffStats, FileDiffStats};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Default)]
pub struct StatsState {
    pub change: Option<DiffStats>,
    pub per_file: Arc<HashMap<String, FileDiffStats>>,
}
