use jayjay_core::RebaseMode;

#[uniffi::remote(Enum)]
pub enum RebaseMode {
    Source,
    Branch,
}
