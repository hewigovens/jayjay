use jayjay_core as core;
use jayjay_core::overview::OverviewGroup;
use jayjay_core::{
    Overview, OverviewBase, OverviewBaseKind, OverviewChange, OverviewLane, OverviewWorkspace,
};

#[uniffi::remote(Record)]
pub struct OverviewChange {
    pub change_id: core::ShortId,
    pub commit_id: core::ShortId,
    pub description: String,
    pub full_description: String,
    pub timestamp_millis: i64,
    pub is_empty: bool,
    pub has_conflict: bool,
    pub bookmarks: Vec<String>,
    pub workspaces: Vec<String>,
}

#[uniffi::remote(Enum)]
pub enum OverviewBaseKind {
    Trunk,
    OlderTrunk,
    Mutable,
    Other,
}

#[uniffi::remote(Record)]
pub struct OverviewBase {
    pub change_id: core::ShortId,
    pub commit_id: core::ShortId,
    pub description: String,
    pub timestamp_millis: i64,
    pub kind: core::OverviewBaseKind,
    pub bookmarks: Vec<String>,
    pub behind_trunk: u32,
}

#[uniffi::remote(Record)]
pub struct OverviewWorkspace {
    pub name: String,
    pub is_current: bool,
    pub changes_above: u32,
}

#[uniffi::remote(Record)]
pub struct OverviewLane {
    pub changes: Vec<core::OverviewChange>,
    pub base: core::OverviewBase,
    pub workspaces: Vec<core::OverviewWorkspace>,
    pub latest_timestamp_millis: i64,
    pub attention: Vec<String>,
}

#[uniffi::remote(Record)]
pub struct Overview {
    pub lanes: Vec<core::OverviewLane>,
    pub workspace_count: u32,
}

#[uniffi::remote(Record)]
pub struct OverviewGroup {
    pub base: core::OverviewBase,
    pub lanes: Vec<u32>,
}
