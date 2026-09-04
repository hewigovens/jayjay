use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ShortcutModifier {
    #[default]
    Ctrl,
    Super,
}

impl ShortcutModifier {
    pub(crate) fn key(self) -> &'static str {
        match self {
            Self::Ctrl => "ctrl",
            Self::Super => "super",
        }
    }
}
