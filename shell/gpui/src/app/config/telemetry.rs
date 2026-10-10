use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct TelemetryConfig {
    /// Anonymous build and OS statistics. No repository, file, or command data is sent.
    pub enabled: bool,
}

impl TelemetryConfig {
    pub const LABEL: &str = "Share anonymous build and OS stats";
    pub const HINT: &str = "No repository, file, or command data is sent.";
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}
