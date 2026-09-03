use jayjay_core::MutationEffect;

#[uniffi::remote(Enum)]
pub enum MutationEffect {
    Changed,
    Unchanged,
}
