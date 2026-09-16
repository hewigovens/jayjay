use jayjay_core::{AiProvider, DiffExcerpt};

#[uniffi::remote(Enum)]
pub enum AiProvider {
    Codex,
    Claude,
    AppleIntelligence,
}

#[uniffi::remote(Record)]
pub struct DiffExcerpt {
    pub stat: String,
    pub diff: String,
}

/// `None` means `DEFAULT_MAX_BYTES`.
#[uniffi::export]
fn diff_excerpt_text(excerpt: DiffExcerpt, max_bytes: Option<u32>) -> String {
    excerpt.text(max_bytes.map_or(DiffExcerpt::DEFAULT_MAX_BYTES, |bytes| bytes as usize))
}

#[uniffi::export]
fn ai_providers(ids: Vec<String>) -> Vec<AiProvider> {
    AiProvider::ordered(&ids)
}

#[uniffi::export]
fn ai_provider_id(provider: AiProvider) -> String {
    provider.id().to_owned()
}

#[uniffi::export]
fn ai_provider_label(provider: AiProvider) -> String {
    provider.label().to_owned()
}

#[uniffi::export]
fn ai_provider_is_installed(provider: AiProvider) -> bool {
    provider.is_installed()
}

#[uniffi::export]
fn generate_commit_message(provider: AiProvider, excerpt: DiffExcerpt) -> Option<String> {
    provider.generate_commit_message(&excerpt.text(DiffExcerpt::DEFAULT_MAX_BYTES))
}

#[uniffi::export]
fn generate_branch_name(provider: AiProvider, description: String) -> Option<String> {
    provider.generate_branch_name(&description)
}
