use jayjay_core::{ShortId, WorkspaceInfo};

#[uniffi::remote(Record)]
pub struct WorkspaceInfo {
    pub name: String,
    pub path: String,
    pub is_path_resolved: bool,
    pub pinnable_path: Option<String>,
    pub is_current: bool,
    pub change_id: ShortId,
    pub description: String,
    pub timestamp: i64,
    pub has_conflict: bool,
    pub files_changed: u32,
}
