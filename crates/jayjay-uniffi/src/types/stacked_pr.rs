use jayjay_core as core;
use jayjay_core::{
    Stack, StackLayer, StackLayerOutcome, StackedPrResult, SubmitStackLayer, SubmittedLayer,
};

#[uniffi::remote(Record)]
pub struct StackLayer {
    pub change_id: String,
    pub commit_id: String,
    pub title: String,
    pub body: String,
    pub bookmark: String,
    pub base: String,
    pub bookmark_existed: bool,
    pub change_id_short: String,
}

#[uniffi::remote(Record)]
pub struct SubmitStackLayer {
    pub change_id: String,
    pub bookmark: String,
    pub title: String,
    pub body: String,
}

#[uniffi::remote(Record)]
pub struct Stack {
    pub layers: Vec<core::StackLayer>,
    pub base_bookmark: String,
}

#[uniffi::remote(Enum)]
pub enum StackLayerOutcome {
    Created,
    Updated,
    Failed,
}

#[uniffi::remote(Record)]
pub struct SubmittedLayer {
    pub bookmark: String,
    pub base: String,
    pub title: String,
    pub outcome: core::StackLayerOutcome,
    pub pr_number: u32,
    pub pr_url: String,
    pub detail: String,
}

#[uniffi::remote(Record)]
pub struct StackedPrResult {
    pub layers: Vec<core::SubmittedLayer>,
    pub message: String,
    pub open_urls: Vec<String>,
}
