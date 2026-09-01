use jayjay_primitives::CliCommandOutcome;

#[uniffi::remote(Record)]
pub struct CliCommandOutcome {
    pub exit_code: i32,
    pub message: String,
}
