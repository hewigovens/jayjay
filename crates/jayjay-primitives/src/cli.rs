pub const JAYJAY_CONFIG_COMMAND: &str = "config";
pub const JAYJAY_REVIEW_COMMAND: &str = "review";
pub const JAYJAY_TOOL_COMMAND: &str = "tool";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliCommandOutcome {
    pub exit_code: i32,
    pub message: String,
}

impl CliCommandOutcome {
    pub fn ok(message: impl Into<String>) -> Self {
        Self {
            exit_code: 0,
            message: message.into(),
        }
    }

    pub fn err(message: impl Into<String>) -> Self {
        Self {
            exit_code: 1,
            message: message.into(),
        }
    }

    pub fn is_error(&self) -> bool {
        self.exit_code != 0
    }
}
