use jayjay_core::{JjConfigEntry, JjConfigSection, JjUserConfig, UserIdentity};

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

#[uniffi::remote(Record)]
pub struct UserIdentity {
    pub name: String,
    pub email: String,
}

#[uniffi::export]
fn jj_user_config() -> Result<JjUserConfig, JayError> {
    jayjay_core::jj_user_config()
}

#[uniffi::export]
fn configured_user_identity() -> Result<Option<UserIdentity>, JayError> {
    UserIdentity::configured()
}

#[uniffi::export]
fn suggested_user_identity() -> Result<UserIdentity, JayError> {
    UserIdentity::suggested()
}

#[uniffi::export]
fn user_identity_config_path() -> Result<String, JayError> {
    UserIdentity::config_path()
}

#[uniffi::export]
fn save_user_identity(identity: UserIdentity) -> Result<(), JayError> {
    identity.save()
}
