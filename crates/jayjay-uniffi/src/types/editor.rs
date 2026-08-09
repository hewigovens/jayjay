use jayjay_core as core;
use jayjay_core::{ConflictEditorData, FileEditorData, MergeEditorHunk, MergeHunkSource};

#[uniffi::remote(Record)]
pub struct ConflictEditorData {
    pub path: String,
    pub is_working_copy: bool,
    pub change_id: String,
    pub conflict_id: String,
    pub left: String,
    pub base: String,
    pub right: String,
    pub result: String,
    pub marker_length: u32,
    pub side_count: u32,
    pub is_text: bool,
    pub hunks: Vec<core::MergeEditorHunk>,
}

#[uniffi::remote(Enum)]
pub enum MergeHunkSource {
    Left,
    Base,
    Right,
}

#[uniffi::remote(Record)]
pub struct MergeEditorHunk {
    pub index: u32,
    pub occurrence: u32,
    pub raw: String,
    pub left: String,
    pub base: String,
    pub right: String,
}

#[uniffi::remote(Record)]
pub struct FileEditorData {
    pub path: String,
    pub change_id: String,
    pub file_id: String,
    pub content: String,
}
