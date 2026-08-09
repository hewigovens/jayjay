#[derive(Debug, Clone)]
pub struct ConflictEditorData {
    pub path: String,
    /// Whether the editor was opened for this workspace's working-copy commit.
    pub is_working_copy: bool,
    /// Snapshot-stable, unlike a commit id, so an open editor survives working-copy snapshots.
    pub change_id: String,
    /// Fingerprint of the conflict's file ids at load; a mismatch at save means the sides changed underneath.
    pub conflict_id: String,
    pub left: String,
    pub base: String,
    pub right: String,
    pub result: String,
    pub marker_length: u32,
    pub side_count: u32,
    pub is_text: bool,
    pub hunks: Vec<MergeEditorHunk>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeHunkSource {
    Left,
    Base,
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeEditorHunk {
    pub index: u32,
    /// Initial ordinal among byte-identical raw blocks; remaining identical conflicts become interchangeable as earlier blocks are resolved.
    pub occurrence: u32,
    pub raw: String,
    pub left: String,
    pub base: String,
    pub right: String,
}
