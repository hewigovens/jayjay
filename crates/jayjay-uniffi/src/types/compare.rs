use jayjay_core::compare::{BookmarkDiffRequest, CompareDisplay, CompareState, RevsetEndpoint};

#[uniffi::remote(Record)]
pub struct RevsetEndpoint {
    pub rev: String,
    pub label: String,
}

#[uniffi::remote(Record)]
pub struct CompareDisplay {
    pub title: String,
    pub from: String,
    pub to: String,
    pub is_combined_selection: bool,
}

#[uniffi::remote(Record)]
pub struct CompareState {
    pub from_rev: String,
    pub to_rev: String,
    pub source_change_id: Option<String>,
    pub target_change_id: Option<String>,
    pub display: CompareDisplay,
}

#[uniffi::remote(Record)]
pub struct BookmarkDiffRequest {
    pub base: RevsetEndpoint,
    pub head: RevsetEndpoint,
    pub head_change_id: String,
}
