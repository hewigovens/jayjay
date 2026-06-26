#[derive(Debug, Clone)]
pub struct CommitAuthor {
    pub name: String,
    pub email: String,
    pub timestamp_millis: i64,
}

impl CommitAuthor {
    pub fn new(name: impl Into<String>, email: impl Into<String>, timestamp_millis: i64) -> Self {
        Self {
            name: name.into(),
            email: email.into(),
            timestamp_millis,
        }
    }

    pub fn empty(timestamp_millis: i64) -> Self {
        Self::new("", "", timestamp_millis)
    }
}

/// An id (change-id or commit-id) paired with the length of its shortest unique
/// prefix among visible commits. Shells highlight that prefix and dim the rest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortId {
    pub id: String,
    pub short_len: u32,
}

impl ShortId {
    pub fn new(id: String, short_len: u32) -> Self {
        Self { id, short_len }
    }

    pub fn as_str(&self) -> &str {
        &self.id
    }
}

/// Deref to the id string so a `ShortId` can be used wherever the bare id was —
/// `&short_id` coerces to `&str`, and str methods (`len`, `starts_with`) apply.
impl std::ops::Deref for ShortId {
    type Target = str;
    fn deref(&self) -> &str {
        &self.id
    }
}

impl PartialEq<str> for ShortId {
    fn eq(&self, other: &str) -> bool {
        self.id == other
    }
}

impl PartialEq<String> for ShortId {
    fn eq(&self, other: &String) -> bool {
        &self.id == other
    }
}

#[derive(Debug, Clone)]
pub struct ChangeInfo {
    pub change_id: ShortId,
    pub commit_id: ShortId,
    pub description: String,
    pub author: CommitAuthor,
    pub parents: Vec<String>,
    pub bookmarks: Vec<String>,
    pub remote_bookmarks: Vec<String>,
    pub tags: Vec<String>,
    pub is_working_copy: bool,
    pub has_conflict: bool,
    pub is_empty: bool,
    pub is_immutable: bool,
    pub is_divergent: bool,
}

/// One entry in a change's evolution history (one rewrite operation).
#[derive(Debug, Clone)]
pub struct EvologEntry {
    pub change_id: ShortId,
    pub commit_id: ShortId,
    /// Operation timestamp (when this rewrite happened).
    pub timestamp_millis: i64,
    /// Operation summary, e.g. "snapshot working copy", "describe commit X", "rebase commit X".
    pub operation: String,
    /// Commit description at this point in evolution (often empty for snapshots).
    pub description: String,
}

/// A change with its graph edges for DAG rendering.
#[derive(Debug, Clone)]
pub struct GraphEntry {
    pub change: ChangeInfo,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone)]
pub struct GraphEdge {
    /// Target commit_id (hex) this edge points to.
    pub target: String,
    pub edge_type: EdgeType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeType {
    Direct,
    Indirect,
    Missing,
}

#[derive(Debug, Clone)]
pub struct ChangeDetail {
    pub info: ChangeInfo,
    pub diff: Vec<super::DiffHunk>,
}
