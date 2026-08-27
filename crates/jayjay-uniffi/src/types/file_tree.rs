use jayjay_core::FileTreeEntry;

#[uniffi::remote(Record)]
pub struct FileTreeEntry {
    pub name: String,
    pub path: String,
    pub depth: u32,
    pub hunk_index: Option<u32>,
}
