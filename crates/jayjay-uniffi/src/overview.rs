use std::sync::Arc;

use jayjay_core::overview::{OverviewGroup, OverviewSnapshot};

#[uniffi::export]
fn overview_lane_ids(snapshot: OverviewSnapshot) -> Vec<String> {
    snapshot.lane_ids()
}

#[uniffi::export]
fn overview_trunk_name(snapshot: OverviewSnapshot) -> String {
    snapshot.trunk_name().to_owned()
}

/// Holds the snapshot on the Rust side so each filter keystroke sends only the filter text.
#[derive(uniffi::Object)]
pub struct OverviewFilter(OverviewSnapshot);

#[uniffi::export]
impl OverviewFilter {
    #[uniffi::constructor]
    fn new(snapshot: OverviewSnapshot) -> Arc<Self> {
        Arc::new(Self(snapshot))
    }

    fn visible_groups(&self, filter: String) -> Vec<OverviewGroup> {
        self.0.visible_groups(&filter)
    }
}
