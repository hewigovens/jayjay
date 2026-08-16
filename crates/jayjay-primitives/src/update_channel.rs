use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum UpdateChannel {
    #[default]
    Stable,
    Beta,
}

impl UpdateChannel {
    pub fn identifier(self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::Beta => "beta",
        }
    }

    /// Unknown values fall back to Stable so nothing opts into betas by accident.
    pub fn parse(value: &str) -> Self {
        if value == "beta" { Self::Beta } else { Self::Stable }
    }
}
