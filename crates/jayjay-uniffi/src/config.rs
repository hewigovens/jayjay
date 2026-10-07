use jayjay_core::{JjConfigEntry, JjConfigSection, JjUserConfig};

use jayjay_core::JayError;

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
fn jj_user_config() -> Result<JjUserConfig, JayError> {
    jayjay_core::jj_user_config()
}
