use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct TelemetryConfig {
    /// Anonymous daily ping (app version, OS, arch). No personal data; opt in by setting true.
    pub enabled: bool,
}
