use jayjay_core::{JjConfigEntry, JjConfigSection, JjUserConfig};

use crate::error::JayJayError;

#[uniffi::remote(Record)]
pub struct JjConfigEntry {
    pub key: String,
    pub value: String,
}

#[uniffi::remote(Record)]
pub struct JjConfigSection {
    pub name: String,
    pub entries: Vec<JjConfigEntry>,
}

#[uniffi::remote(Record)]
pub struct JjUserConfig {
    pub path: String,
    pub exists: bool,
    pub sections: Vec<JjConfigSection>,
    pub error: Option<String>,
}

#[uniffi::export]
fn jj_user_config() -> Result<JjUserConfig, JayJayError> {
    Ok(jayjay_core::jj_user_config()?)
}
