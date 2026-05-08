use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedRevset {
    pub id: String,
    pub name: String,
    pub expression: String,
}
