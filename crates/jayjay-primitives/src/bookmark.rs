use super::change::ShortId;

#[derive(Debug, Clone)]
pub struct BookmarkInfo {
    pub name: String,
    pub change_id: ShortId,
    pub description: String,
    pub is_tracking_remote: bool,
    pub is_deleted: bool,
    pub is_conflicted: bool,
    pub tracked_remotes: Vec<String>,
    pub available_remotes: Vec<String>,
    /// False for synthesized remote entries, including locally deleted bookmarks still tracked on a remote.
    pub has_local_target: bool,
    /// Empty for remote-only (orphan) entries.
    pub remote_targets: Vec<RemoteBookmarkTarget>,
}

impl BookmarkInfo {
    /// Conflicted state is repo-wide (`list_bookmarks`), not a second list on each change.
    pub fn is_conflicted_name(bookmarks: &[Self], name: &str) -> bool {
        bookmarks
            .iter()
            .any(|bookmark| bookmark.name == name && bookmark.is_conflicted)
    }
}

#[derive(Debug, Clone)]
pub struct RemoteBookmarkTarget {
    pub remote: String,
    /// Empty if the target can't be resolved.
    pub change_id: String,
    /// First line of the remote target's description.
    pub description: String,
    pub status: RemoteSyncStatus,
    pub ahead: u32,
    pub behind: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteSyncStatus {
    Synced,
    /// Local bookmark has commits the remote lacks — push to update the remote.
    Ahead,
    /// Remote has commits the local bookmark lacks — fetch to catch up.
    Behind,
    Diverged,
}
