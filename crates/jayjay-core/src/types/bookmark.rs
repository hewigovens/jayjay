use super::change::ShortId;

#[derive(Debug, Clone)]
pub struct BookmarkInfo {
    pub name: String,
    /// Change id at the local target, with its shortest unique prefix length.
    pub change_id: ShortId,
    pub description: String,
    pub is_tracking_remote: bool,
    pub is_deleted: bool,
    pub is_conflicted: bool,
    pub tracked_remotes: Vec<String>,
    pub available_remotes: Vec<String>,
    /// False for synthesized entries from an untracked remote bookmark (e.g. `feature@origin`).
    pub has_local_target: bool,
    /// Position of each tracked remote ref (e.g. `feature@origin`) relative to the local bookmark, so the UI can show whether they are in sync and, if not, where the remote sits. Empty for remote-only (orphan) entries.
    pub remote_targets: Vec<RemoteBookmarkTarget>,
}

/// Where a tracked remote bookmark (`name@remote`) points, relative to the local bookmark of the same name.
#[derive(Debug, Clone)]
pub struct RemoteBookmarkTarget {
    pub remote: String,
    /// Change id at the remote target; empty if the target can't be resolved.
    pub change_id: String,
    /// First line of the remote target's description.
    pub description: String,
    /// Position of the remote ref relative to the local bookmark.
    pub status: RemoteSyncStatus,
    /// Local commits the remote lacks (how far ahead the local bookmark is).
    pub ahead: u32,
    /// Remote commits the local bookmark lacks (how far behind it is).
    pub behind: u32,
}

/// Position of a tracked remote ref relative to its local bookmark.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteSyncStatus {
    /// Remote points at the same commit as the local bookmark.
    Synced,
    /// Local bookmark has commits the remote lacks — push to update the remote.
    Ahead,
    /// Remote has commits the local bookmark lacks — fetch to catch up.
    Behind,
    /// Each side has commits the other lacks.
    Diverged,
}
