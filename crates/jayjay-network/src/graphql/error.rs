use serde::Deserialize;
use serde_json::{Map, Value};

#[derive(Debug, Deserialize)]
pub struct GraphQlError {
    #[serde(default)]
    pub message: String,
    #[serde(flatten)]
    pub metadata: Map<String, Value>,
}

impl GraphQlError {
    pub fn kind(&self) -> Option<&str> {
        self.metadata.get("type")?.as_str()
    }
}
