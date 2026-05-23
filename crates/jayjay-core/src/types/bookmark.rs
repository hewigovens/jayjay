#[derive(Debug, Clone)]
pub struct BookmarkInfo {
    pub name: String,
    pub change_id: String,
    pub description: String,
    pub is_tracking_remote: bool,
    pub is_deleted: bool,
    pub is_conflicted: bool,
    pub tracked_remotes: Vec<String>,
    pub available_remotes: Vec<String>,
    /// False for synthesized entries from an untracked remote bookmark (e.g. `feature@origin`).
    pub has_local_target: bool,
}
