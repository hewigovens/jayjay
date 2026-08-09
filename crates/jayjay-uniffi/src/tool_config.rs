#[uniffi::export]
pub fn jj_tool_config() -> String {
    jayjay_core::JJ_TOOL_CONFIG.to_owned()
}
