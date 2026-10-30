use std::sync::Arc;

use jayjay_core::JayError;
use jayjay_core::flock::{FlockMember, FlockStore, FlockVerification};

use crate::repo::JayJaySyncToken;

#[uniffi::remote(Record)]
pub struct FlockMember {
    pub id: String,
    pub login: String,
    pub name: Option<String>,
}

#[uniffi::remote(Enum)]
pub enum FlockVerification {
    SignInRequired,
    NotSponsored { login: String },
    Unlocked { member: FlockMember },
}

#[uniffi::export]
fn flock_member() -> Option<FlockMember> {
    FlockStore::load().member()
}

#[uniffi::export]
fn redeem_flock_code(code: String, name: String) -> Result<Option<FlockMember>, JayError> {
    FlockStore::load().redeem(&code, &name)
}

#[uniffi::export]
fn forget_flock() -> Result<(), JayError> {
    FlockStore::load().forget()
}

#[uniffi::export]
fn verify_flock(
    sponsorable: String,
    sync: Arc<JayJaySyncToken>,
) -> Result<FlockVerification, JayError> {
    FlockStore::load().verify(&sponsorable, &sync.inner)
}

#[derive(uniffi::Object)]
pub struct FlockSignIn {
    inner: jayjay_core::flock::FlockSignIn,
}

#[uniffi::export]
impl FlockSignIn {
    #[uniffi::constructor]
    fn start(sponsorable: String) -> Result<Arc<Self>, JayError> {
        Ok(Arc::new(Self {
            inner: jayjay_core::flock::FlockSignIn::start(sponsorable)?,
        }))
    }

    fn user_code(&self) -> String {
        self.inner.user_code().to_owned()
    }

    fn verification_uri(&self) -> String {
        self.inner.verification_uri().to_owned()
    }

    fn wait(&self, sync: Arc<JayJaySyncToken>) -> Result<FlockVerification, JayError> {
        self.inner.wait(&sync.inner)
    }
}
