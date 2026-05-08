#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JjCommandRun {
    pub display: String,
    pub stdout: String,
    pub stderr: String,
    pub output: String,
    pub exit_code: i32,
    pub success: bool,
}
